use ggez::{graphics, Context, GameResult};
use nalgebra::{Point2, Isometry2};
use ncollide2d::world::CollisionWorld;
use ncollide2d::pipeline::object::CollisionObjectSlabHandle;

use crate::collisionobjectdata::CollisionObjectData;
use crate::random_helper::RandomHelper;
use crate::randomwalker::RandomWalker;


pub struct Food {
    handle: CollisionObjectSlabHandle,
    size: f32,
}

impl Food {
    pub fn new(handle: CollisionObjectSlabHandle, size: f32) -> Self {
        Food {
            handle: handle,
            size: size,
        }
    }

    pub fn update(&mut self, world: &mut CollisionWorld<f32, CollisionObjectData>, random: &mut RandomHelper) {
        let food_object = world.get_mut(self.handle).unwrap();
        let food_data = food_object.data();
        if food_data.eaten.get() {
            let new_pos = random.random_coordinate();
            food_data.eaten.set(false);
            food_object.set_position(Isometry2::translation(new_pos.0, new_pos.1));
        }
    }

    pub fn draw(&self, ctx: &mut Context, world: &mut CollisionWorld<f32, CollisionObjectData>) -> GameResult<()> {
        let color = [0.5, 1.0, 0.0, 1.0].into();
        let food_object = world.collision_object(self.handle).unwrap();
        let food_pos = food_object.position();
        let pos = food_pos.transform_point(&Point2::origin());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::nalgebra::Point2::new(0.0, 0.0),
            self.size,
            1.0,
            color,
        )?;
        let drawparams = graphics::DrawParam::new()
            .dest(RandomWalker::convert_point(pos));
        graphics::draw(ctx, &circle, drawparams)?;
        Ok(())
    }
}