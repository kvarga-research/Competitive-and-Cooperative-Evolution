use ggez::{graphics, Context, GameResult};
use nalgebra::{Point2, Vector2, Translation2, Isometry2};
use ncollide2d::shape::ConvexPolygon;
use ncollide2d::query::Ray;
use ncollide2d::world::CollisionWorld;
use ncollide2d::pipeline::object::{CollisionObjectSlabHandle, CollisionGroups};
use nalgebra::base::Matrix;
use nalgebra::geometry::UnitComplex;

use crate::collisionobjectdata::CollisionObjectData;
use crate::brain::{Brain, BrainNetwork};
use crate::entity::Entity;


pub struct RandomWalker {
    id: i32,
    size: f32,
    handle: CollisionObjectSlabHandle,
    env_handle: Option<CollisionObjectSlabHandle>,
    brain: Brain,
    rays: Vec<(Point2<f32>, Vector2<f32>)>,
    thinking: i32,
    last_trans: Translation2<f32>,
    initial_health: i32,
    health: i32,
    score: i32,
    entity: Entity,
    speed: f32,
    thinking_time: i32,
    facing: i8,
    color: [f32; 4],
    top_color: [f32; 4],
}
impl RandomWalker {
    pub fn new(handle: CollisionObjectSlabHandle, env_handle: Option<CollisionObjectSlabHandle>, id: i32, size: f32, speed: f32, health: i32, entity: Entity,
        thinking_time: i32, view_range: f32, mutation_rate: f32, seed: u64, color: [f32; 4], top_color: [f32; 4],
    ) -> Self {
        RandomWalker{
            id: id,
            size: size,
            handle: handle,
            env_handle: env_handle,
            facing: 1,
            brain: Brain::new(view_range, mutation_rate, seed),
            rays: vec![(Point2::new(1.0, 1.0), Matrix::x());8],
            thinking: thinking_time,
            last_trans: Translation2::new(0.0, 0.0),
            initial_health: health,
            health: health,
            score: 0,
            entity: entity,
            speed: speed,
            thinking_time: thinking_time,
            color: color,
            top_color: top_color,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_handle(&self) -> CollisionObjectSlabHandle {
        self.handle
    }

    pub fn get_health(&self) -> i32 {
        self.health
    }

    pub fn create_polygon(size: f32, pos_x: f32, pos_y: f32) -> ConvexPolygon<f32> {
        let points = RandomWalker::polygon_points(size, pos_x, pos_y);
        ConvexPolygon::try_new(points).expect("Convex hull computation failed.")
    }

    pub fn polygon_points(size: f32, pos_x: f32, pos_y: f32) -> Vec<Point2<f32>> {
        let arm_size = size / 3.0;
        let points = vec![
            Point2::new(pos_x, pos_y - arm_size),
            Point2::new(pos_x + arm_size, pos_y),
            Point2::new(pos_x, pos_y + arm_size),
            Point2::new(pos_x - arm_size, pos_y),
        ];
        points
    }

    fn get_ray_interferences<'a>(&'a mut self, new_pos: Isometry2<f32>,
        world: &'a CollisionWorld<f32, CollisionObjectData>,
    ) -> Vec<Option<(Entity, f32)>>
    {
        let mut closest_objects = Vec::new();
        let points = self.get_sensor_points();
        let mut ray_group = CollisionGroups::new();
        if self.entity == Entity::HERBIVORE {
            ray_group.set_membership(&[4]);
            ray_group.set_whitelist(&[1, 3, 5]);
            ray_group.set_blacklist(&[2, 4, 6, 7]);
        }
        else {
            ray_group.set_membership(&[6]);
            ray_group.set_whitelist(&[2, 3, 5]);
            ray_group.set_blacklist(&[1, 4, 6, 7]);
        }
        self.rays = Vec::new();
        let mut was_thinking = false;
        for i in 0..points.len() {
            let origin = new_pos.transform_point(&points[i]);
            let dir = Matrix::normalize(&(origin.coords - new_pos.translation.vector));
            self.rays.push((origin, dir));
            if self.thinking >= self.thinking_time {
                let ray = Ray::new(origin + 0.1 * dir, dir); // 0.1 is needed for them to not detect themselves
                let closest_object = world.first_interference_with_ray(&ray, self.brain.max_sensor_distance(), &ray_group);
                if let Some(obj) = closest_object {
                    closest_objects.push(Some((obj.co.data().entity_type, obj.inter.toi)));
                }
                else {
                    closest_objects.push(None);
                }
                was_thinking = true;
            }
        }
        if was_thinking {
            self.thinking = 0;
        }
        closest_objects
    }

    pub fn update(&mut self, world: &mut CollisionWorld<f32, CollisionObjectData>) {
        let mut new_pos = world.collision_object(self.handle).unwrap().position().clone();
        let translation;
        let detected_objects = self.get_ray_interferences(new_pos, world);
        if detected_objects.len() > 0 {
            self.facing = self.brain.get_new_direction(detected_objects, self.entity, self.facing);
            let vertical: f32;
            let horizontal: f32;
            match self.facing {
                0 => {
                    vertical = - 1.0;
                    horizontal = 0.0;
                }
                1 => {
                    vertical = - 1.0;
                    horizontal = 1.0;
                }
                2 => {
                    vertical = 0.0;
                    horizontal = 1.0;
                }
                3 => {
                    vertical = 1.0;
                    horizontal = 1.0;
                }
                4 => {
                    vertical = 1.0;
                    horizontal = 0.0;
                }
                5 => {
                    vertical = 1.0;
                    horizontal = - 1.0;
                }
                6 => {
                    vertical = 0.0;
                    horizontal = - 1.0;
                }
                7 => {
                    vertical = - 1.0;
                    horizontal = - 1.0;
                }
                _ => {
                    vertical = 0.0;
                    horizontal = 0.0;
                }
            };
            let length = ((vertical * vertical) + (horizontal * horizontal)).sqrt();
            translation = Translation2::new(horizontal / length * self.speed, vertical / length * self.speed);
            self.last_trans = translation;
        }
        else {
            translation = self.last_trans;
        }
        new_pos.append_translation_mut(&translation);
        {
            if let Some(env) = self.env_handle {
                let env_object = world.get_mut(env).unwrap();
                env_object.set_position(new_pos);
            }
        }
        let randomwalker_object = world.get_mut(self.handle).unwrap();
        self.thinking += 1;
        randomwalker_object.set_position(new_pos);
        self.health = randomwalker_object.data().energy.get();
        if self.health > 2500 {
            self.health = 2500;
            randomwalker_object.data().energy.set(self.health);
        }
        else {
            randomwalker_object.data().energy.set(self.health - 1);
        }
        self.score += 1;
        self.score += randomwalker_object.data().fitness.get();
        randomwalker_object.data().fitness.set(0);
        randomwalker_object.data().score.set(self.score);
    }
    pub fn draw(&self, ctx: &mut Context, world: &mut CollisionWorld<f32, CollisionObjectData>, top: bool, show_details: bool) -> GameResult<()> {
        
        let randomwalker_object = world.collision_object(self.handle).unwrap();
        let randomwalker_object_pos = randomwalker_object.position();
        let pos = randomwalker_object_pos.transform_point(&Point2::new(0.0, 0.0));
        if show_details {
            // Drawing the rays
            let mut drawable_rays = Vec::new();
            drawable_rays.push(&self.rays[(self.facing as usize + 6) % 8]);
            drawable_rays.push(&self.rays[(self.facing as usize + 7) % 8]);
            drawable_rays.push(&self.rays[(self.facing as usize) % 8]);
            drawable_rays.push(&self.rays[(self.facing as usize + 1) % 8]);
            drawable_rays.push(&self.rays[(self.facing as usize + 2) % 8]);
            for (ori, dir) in drawable_rays.iter() {
                let draw_ori = RandomWalker::convert_point(ori.clone());
                let draw_dir = RandomWalker::convert_point(ori.clone() + dir.clone() * 30.0);
                let line = graphics::Mesh::new_line(
                    ctx,
                    &[draw_ori, draw_dir],
                    1.0,
                    [1.0, 1.0, 1.0, 1.0].into(),
                )?;
                let drawpar = graphics::DrawParam::new();
                graphics::draw(ctx, &line, drawpar)?;
            }
            // Drawing the healthbar
            let start = RandomWalker::convert_point(pos) + ggez::nalgebra::Vector2::new(-50.0, -50.0);
            let end = start + ggez::nalgebra::Vector2::new(self.health as f32 / 20.0, 0.0);
            let line = graphics::Mesh::new_line(
                ctx,
                &[start, end],
                10.0,
                [1.0, 0.0, 0.0, 1.0].into(),
            )?;
            let drawpar = graphics::DrawParam::new();
            graphics::draw(ctx, &line, drawpar)?;
            // Drawing the score
            let score = graphics::Text::new((self.score.to_string(), graphics::Font::default(), 24.0));
            graphics::draw(ctx, &score, (start + ggez::nalgebra::Vector2::new(0.0, 15.0), 0.0, graphics::WHITE))?;
        }
        let mut color = self.color;
        if top {color = self.top_color}
        // Drawing the polygon
        let polygon = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::stroke(5.0),
            &RandomWalker::convert_points(RandomWalker::polygon_points(self.size, pos.coords.x, pos.coords.y)),
            color.into(),
        )?;
        let drawparams = graphics::DrawParam::new()
            .offset(RandomWalker::convert_point(pos))
            .rotation(randomwalker_object_pos.rotation.angle());
        graphics::draw(ctx, &polygon, drawparams)
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }
    
    pub fn is_dead(& mut self, world: & CollisionWorld<f32, CollisionObjectData>) -> bool {
        let randomwalker_object = world.collision_object(self.handle).unwrap();
        let eaten = randomwalker_object.data().eaten.get();
        let out_of_energy = randomwalker_object.data().energy.get() <= 0;
        eaten || out_of_energy
    }

    pub fn respawn(&mut self, world: &mut CollisionWorld<f32, CollisionObjectData>, x: f32, y: f32, mutate: bool,
        networks: BrainNetwork
    ) {
        let randomwalker_object = world.get_mut(self.handle).unwrap();
        randomwalker_object.data().eaten.set(false);
        self.health = self.initial_health;
        randomwalker_object.data().energy.set(self.initial_health);
        randomwalker_object.set_position(Isometry2::from_parts(Translation2::new(x, y), UnitComplex::new(0.0)));
        self.brain.set_networks(networks.clone());
        if mutate {
            self.brain.mutate();
        }
        self.thinking = self.thinking_time;
        self.score = 0;
    }

    pub fn get_brain(&self) -> BrainNetwork {
        self.brain.get_networks()
    }

    fn get_sensor_points(&self) -> Vec<Point2<f32>> {
        let points = RandomWalker::polygon_points(self.size, 0.0, 0.0);
        let mut sensor_points = Vec::new();
        for i in 0..points.len() {
            let x1 = points[i].coords.x.clone();
            let y1 = points[i].coords.y.clone();
            let x2 = points[(i + 1) % points.len()].coords.x.clone();
            let y2 = points[(i + 1) % points.len()].coords.y.clone();

            sensor_points.push(points[i]);
            sensor_points.push(Point2::new((x1 + x2) / 2.0, (y1 + y2) / 2.0));
        }
        sensor_points
    }

    // ggez and ncollide use different versions of nalgebra
    pub fn convert_point(point: Point2<f32>) -> ggez::nalgebra::Point2<f32> {
        ggez::nalgebra::Point2::new(point.coords.x, point.coords.y)
    }

    pub fn convert_points(points: Vec<Point2<f32>>) -> Vec<ggez::nalgebra::Point2<f32>> {
        let mut old_lib_points: Vec<ggez::nalgebra::Point2<f32>> = Vec::new();
        for point in points.iter() {
            old_lib_points.push(RandomWalker::convert_point(point.clone()));
        }
        old_lib_points
    }
}