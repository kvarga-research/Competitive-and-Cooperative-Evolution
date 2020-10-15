use ggez::{event, graphics, Context, GameResult, input};
use ggez::event::KeyCode;
use nalgebra::{zero, Isometry2, Point2, Vector2};
use ncollide2d::pipeline::object::{CollisionGroups, GeometricQueryType, CollisionObject};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, Segment, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::collisionobjectdata::CollisionObjectData;
use crate::entity::Entity;
use crate::food::Food;
use crate::random_helper::RandomHelper;
use crate::randomwalker::RandomWalker;
use crate::wall::Wall;
use crate::record::{GamestateRecord, EventRecord, AverageRecord, Writer};

pub struct GameState {
    foods: Vec<Food>,
    millis_per_update: u64,
    // Tracking the last time we updated so that we can limit our update rate.
    last_update: Instant,
    start: Instant,
    herbivores: Vec<RandomWalker>,
    carnivores_1: Vec<RandomWalker>,
    carnivores_2: Vec<RandomWalker>,
    world: CollisionWorld<f32, CollisionObjectData>,
    random: RandomHelper,
    walls: Vec<Wall>,
    best_herbivore_score: i32,
    best_carnivore_score: i32,
    food_nutrition: i32,
    herbivore_nutrition: i32,
    threshold_herbivore_score: i32,
    sharing_percentage_1: f32,
    sharing_percentage_2: f32,
    start_recording: u64,
    recording_duration: u64,
    record_all_details: bool,
    recording: bool,
    simulation_writer: Writer,
    event_writer: Writer,
    average_writer: Writer,
    counter: u64,
    show_details: bool,
}

impl GameState {
    // Set up the initial state of the simulation
    pub fn new(configs: (Value, Value)) -> Self {
        let config = configs.0;
        let wall_config = configs.1;

        let food_amount = config["food_amount"].as_i64().unwrap() as i32;
        let herbivore_amount = config["herbivore_amount"].as_i64().unwrap() as i32;
        let carnivore_amount_1 = config["carnivore_amount_1"].as_i64().unwrap() as i32;
        let carnivore_amount_2 = config["carnivore_amount_2"].as_i64().unwrap() as i32;
        let updates_per_second = config["updates_per_second"].as_f64().unwrap() as f32;
        let screen_size_x = config["screen_size_x"].as_f64().unwrap() as f32;
        let screen_size_y = config["screen_size_y"].as_f64().unwrap() as f32;
        let seed = config["seed"].as_u64().unwrap() as u64;
        let herbivore_speed = config["herbivore_speed"].as_f64().unwrap() as f32;
        let carnivore_speed = config["carnivore_speed"].as_f64().unwrap() as f32;
        let initial_herbivore_health = config["initial_herbivore_health"].as_i64().unwrap() as i32;
        let initial_carnivore_health = config["initial_carnivore_health"].as_i64().unwrap() as i32;
        let food_nutrition = config["food_nutrition"].as_i64().unwrap() as i32;
        let herbivore_nutrition = config["herbivore_nutrition"].as_i64().unwrap() as i32;
        let threshold_herbivore_score = config["threshold_herbivore_score"].as_i64().unwrap() as i32;
        let sharing_percentage_1 = config["sharing_percentage_1"].as_i64().unwrap() as i32;
        let sharing_percentage_2 = config["sharing_percentage_2"].as_i64().unwrap() as i32;
        let share_range = config["share_range"].as_f64().unwrap() as f32;
        let mutation_rate = config["mutation_rate"].as_f64().unwrap() as f32;
        let herbivore_size = config["herbivore_size"].as_f64().unwrap() as f32;
        let carnivore_size = config["carnivore_size"].as_f64().unwrap() as f32;
        let thinking_time = config["thinking_time"].as_i64().unwrap() as i32;
        let view_range = config["view_range"].as_f64().unwrap() as f32;
        let start_recording = config["start_recording"].as_u64().unwrap() as u64;
        let recording_duration = config["recording_duration"].as_u64().unwrap() as u64;
        let record_all_details = config["record_all_details"].as_bool().unwrap();

        let mut food_group = CollisionGroups::new();
        let mut herbivore_group = CollisionGroups::new();
        let mut carnivore_group = CollisionGroups::new();
        let mut wall_group = CollisionGroups::new();
        let mut carnivore_environment_group = CollisionGroups::new();
        //GROUPS:
        //FOOD: 1
        //herbivore: 2
        //CARNIVORE: 3
        //herbivore RAY: 4
        //WALL: 5
        //CARNIVORE RAY: 6
        //CARNIVORE ENVIRONMENT: 7
        food_group.set_membership(&[1]);
        food_group.set_whitelist(&[2, 4]);
        food_group.set_blacklist(&[1, 3, 5, 6, 7]);
        herbivore_group.set_membership(&[2]);
        herbivore_group.set_whitelist(&[1, 3, 5, 6]);
        herbivore_group.set_blacklist(&[2, 4, 7]);
        carnivore_group.set_membership(&[3]);
        carnivore_group.set_whitelist(&[2, 4, 5, 6, 7]);
        carnivore_group.set_blacklist(&[1, 3]);
        wall_group.set_membership(&[5]);
        wall_group.set_whitelist(&[2, 3, 4, 6]);
        wall_group.set_blacklist(&[1, 5, 7]);
        carnivore_environment_group.set_membership(&[7]);
        carnivore_environment_group.set_whitelist(&[3]);
        carnivore_environment_group.set_blacklist(&[1, 2, 4, 5, 6, 7]);
        let query = GeometricQueryType::Proximity(0.01);
        let mut world = CollisionWorld::new(0.01);
        let ball = ShapeHandle::new(Ball::new(herbivore_size / 2.0));
        let omni_polygon = ShapeHandle::new(RandomWalker::create_polygon(herbivore_size, 0.0, 0.0));
        let carni_polygon =
            ShapeHandle::new(RandomWalker::create_polygon(carnivore_size, 0.0, 0.0));
        let carni_env_circle = ShapeHandle::new(Ball::new(share_range));

        let mut random_helper = RandomHelper::new(screen_size_x, screen_size_y, seed);
        let mut foods = Vec::new();
        let mut herbivores = Vec::new();
        let mut carnivores_1 = Vec::new();
        let mut carnivores_2 = Vec::new();
        let mut walls = Vec::new();
        for _ in 0..food_amount {
            let (pos_x, pos_y) = random_helper.random_coordinate();
            let coll_data = CollisionObjectData::new(Entity::FOOD, -2, None);
            foods.push(Food::new(
                world
                    .add(
                        Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                        ball.clone(),
                        food_group,
                        query,
                        coll_data,
                    )
                    .0,
                    herbivore_size / 2.0
            ))
        }
        for i in 0..herbivore_amount {
            let (pos_x, pos_y) = random_helper.random_coordinate();
            let coll_data = CollisionObjectData::new(Entity::HERBIVORE, i, None);
            herbivores.push(RandomWalker::new(
                world
                    .add(
                        Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                        omni_polygon.clone(),
                        herbivore_group,
                        query,
                        coll_data,
                    )
                    .0,
                None,
                i,
                herbivore_size,
                herbivore_speed,
                initial_herbivore_health,
                Entity::HERBIVORE,
                thinking_time,
                view_range,
                mutation_rate,
                seed + i as u64 + 3333,
                [1.0, 0.5, 0.0, 1.0],
                [1.0, 0.5, 1.0, 1.0],
            ))
        }
        for i in 0..carnivore_amount_1 {
            let (pos_x, pos_y) = random_helper.random_coordinate();
            let env_coll_data =
                CollisionObjectData::new(Entity::OTHER, i, None);
            let env_handle = Some(world
                .add(
                    Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                    carni_env_circle.clone(),
                    carnivore_environment_group,
                    query,
                    env_coll_data,
                )
                .0);
            let coll_data =
                CollisionObjectData::new(Entity::CARNIVORE, i, env_handle);
            carnivores_1.push(RandomWalker::new(
                world
                    .add(
                        Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                        carni_polygon.clone(),
                        carnivore_group,
                        query,
                        coll_data,
                    )
                    .0,
                env_handle,
                i,
                carnivore_size,
                carnivore_speed,
                initial_carnivore_health,
                Entity::CARNIVORE,
                thinking_time,
                view_range,
                mutation_rate,
                seed + i as u64 + 5555,
                [1.0, 0.5, 0.5, 1.0],
                [1.0, 1.0, 0.0, 1.0],
            ))
        }
        for i in 0..carnivore_amount_2 {
            let (pos_x, pos_y) = random_helper.random_coordinate();
            let env_coll_data =
                CollisionObjectData::new(Entity::OTHER, i + carnivore_amount_1, None);
            let env_handle = Some(world
                .add(
                    Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                    carni_env_circle.clone(),
                    carnivore_environment_group,
                    query,
                    env_coll_data,
                )
                .0);
            let coll_data =
                CollisionObjectData::new(Entity::CARNIVORE, i + carnivore_amount_1, env_handle);
            carnivores_2.push(RandomWalker::new(
                world
                    .add(
                        Isometry2::new(Vector2::new(pos_x, pos_y), zero()),
                        carni_polygon.clone(),
                        carnivore_group,
                        query,
                        coll_data,
                    )
                    .0,
                env_handle,
                i + carnivore_amount_1,
                carnivore_size,
                carnivore_speed,
                initial_carnivore_health,
                Entity::CARNIVORE,
                thinking_time,
                view_range,
                mutation_rate,
                seed + carnivore_amount_1 as u64 + i as u64 + 5555,
                [0.0, 0.3, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
            ))
        }
        for wall in wall_config["walls"].as_array().unwrap() {
            let x1 = wall["x1"].as_f64().unwrap() as f32;
            let y1 = wall["y1"].as_f64().unwrap() as f32;
            let x2 = wall["x2"].as_f64().unwrap() as f32;
            let y2 = wall["y2"].as_f64().unwrap() as f32;
            let first = Point2::new(x1 * screen_size_x, y1 * screen_size_y);
            let second = Point2::new(x2 * screen_size_x, y2 * screen_size_y);
            let shape = ShapeHandle::new(Segment::new(first, second));
            let coll_data = CollisionObjectData::new(Entity::WALL, -1, None);
            world.add(
                Isometry2::new(Vector2::new(0.0, 0.0), zero()),
                shape,
                wall_group,
                query,
                coll_data.clone(),
            );
            walls.push(Wall::new(first, second));
        }

        GameState {
            last_update: Instant::now(),
            millis_per_update: (1.0 / updates_per_second * 1000.0) as u64,
            foods: foods,
            world: world,
            random: random_helper,
            walls: walls,
            best_herbivore_score: 0,
            herbivores: herbivores,
            best_carnivore_score: 0,
            carnivores_1: carnivores_1,
            carnivores_2: carnivores_2,
            food_nutrition: food_nutrition,
            herbivore_nutrition: herbivore_nutrition,
            threshold_herbivore_score: threshold_herbivore_score,
            sharing_percentage_1: sharing_percentage_1 as f32 / 100.0,
            sharing_percentage_2: sharing_percentage_2 as f32 / 100.0,
            start: Instant::now(),
            start_recording: start_recording,
            recording_duration: recording_duration,
            record_all_details: record_all_details,
            recording: false,
            simulation_writer: Writer::new("simulation.csv"),
            event_writer: Writer::new("event.csv"),
            average_writer: Writer::new("average.csv"),
            counter: 0,
            show_details: true,
        }
    }

    fn handle_carnivore_herbivore_event(&self, carnivore: &CollisionObject<f32, CollisionObjectData>, herbivore: &CollisionObject<f32, CollisionObjectData>, sharing_percentage: f32) -> (bool, i32) {
        let mut is_herbivore = false;
        let mut hunt_counter = 0;
        if herbivore.data().entity_type == Entity::HERBIVORE {
            is_herbivore = true;
            herbivore.data().eaten.set(true);
            let added_nutrition = self.herbivore_nutrition as f32 * (herbivore.data().score.get() as f32 / self.threshold_herbivore_score as f32);
            carnivore.data()
                .energy
                .set(carnivore.data().energy.get() + (added_nutrition * (1.0 - sharing_percentage)) as i32);
            carnivore.data()
                .fitness
                .set(carnivore.data().fitness.get() + (added_nutrition * (1.0 - sharing_percentage)) as i32);
            if let Some(handle) = carnivore.data().env_handle {
                let interaction_results = self.world.interactions_with(handle, true);
                if let Some(friends) = interaction_results {
                    let mut friends_collection = Vec::new();
                    for (_, (c1, c2, _)) in friends.enumerate() {
                        friends_collection.push((c1, c2));
                    }
                    let friend_count = friends_collection.len();
                    hunt_counter = friend_count + 1;
                    let share = added_nutrition * sharing_percentage / friend_count as f32;
                    for (c1, c2) in friends_collection {
                        let friend = self.world.collision_object(c1).unwrap();
                        if friend.data().id != carnivore.data().id {
                            friend.data()
                            .energy
                            .set(friend.data().energy.get() + share as i32);
                            friend.data()
                            .fitness
                            .set(friend.data().fitness.get() + share as i32);
                        }
                        let friend = self.world.collision_object(c2).unwrap();
                        if friend.data().id != carnivore.data().id {
                            friend.data()
                                .energy
                                .set(friend.data().energy.get() + share as i32);
                            friend.data()
                                .fitness
                                .set(friend.data().fitness.get() + share as i32);
                        }
                    }
                    if friend_count == 0 {
                        carnivore.data()
                            .energy
                            .set(carnivore.data().energy.get() + (added_nutrition * sharing_percentage) as i32);
                        carnivore.data()
                            .fitness
                            .set(carnivore.data().fitness.get() + (added_nutrition * sharing_percentage) as i32);
                    }
                }
            }
        }
        return (is_herbivore, hunt_counter as i32);
    }

    fn handle_proximity_event(&mut self) {
        for event in self.world.proximity_events() {
            if event.new_status == Proximity::Intersecting {
                let mut hunt_happened = false;
                let mut hunt_count = 0;
                let co1 = self.world.collision_object(event.collider1).unwrap();
                let co2 = self.world.collision_object(event.collider2).unwrap();
                match co1.data().entity_type {
                    Entity::FOOD => {
                        co1.data().eaten.set(true);
                        co2.data()
                            .energy
                            .set(co2.data().energy.get() + self.food_nutrition);
                        co2.data()
                            .fitness
                            .set(co2.data().fitness.get() + self.food_nutrition);
                    }
                    Entity::WALL => co2.data().eaten.set(true),
                    Entity::CARNIVORE => {
                        let mut sharing_percentage = self.sharing_percentage_1;
                        if co1.data().id >= self.carnivores_1.len() as i32 {
                            sharing_percentage = self.sharing_percentage_2;
                        }
                        let result = self.handle_carnivore_herbivore_event(co1, co2, sharing_percentage);
                        hunt_happened = result.0;
                        hunt_count = result.1;
                    } 
                    _ => (),
                }
                match co2.data().entity_type {
                    Entity::FOOD => {
                        co2.data().eaten.set(true);
                        co1.data()
                            .energy
                            .set(co1.data().energy.get() + self.food_nutrition);
                        co1.data()
                            .fitness
                            .set(co1.data().fitness.get() + self.food_nutrition);
                    }
                    Entity::WALL => co1.data().eaten.set(true),
                    Entity::CARNIVORE => {
                        let mut sharing_percentage = self.sharing_percentage_1;
                        if co2.data().id >= self.carnivores_1.len() as i32 {
                            sharing_percentage = self.sharing_percentage_2;
                        }
                        let result = self.handle_carnivore_herbivore_event(co2, co1, sharing_percentage);
                        hunt_happened = result.0;
                        hunt_count = result.1;
                    }
                    _ => (),
                }
                if self.recording && co1.data().entity_type != Entity::OTHER && co2.data().entity_type != Entity::OTHER {
                    let pos_x;
                    let pos_y;
                    if co1.data().entity_type != Entity::WALL {
                        pos_x = co1.position().translation.x;
                        pos_y = co1.position().translation.y;
                    } else {
                        pos_x = co2.position().translation.x;
                        pos_y = co2.position().translation.y;
                    }
                    let record = EventRecord::new(
                        self.counter,
                        co1.data().id,
                        co2.data().id,
                        co1.data().entity_type.to_string(),
                        co2.data().entity_type.to_string(),
                        pos_x as u64,
                        pos_y as u64,
                        co1.data().score.get() as u64,
                        co2.data().score.get() as u64,
                    );
                    self.event_writer.write_event_record(record).unwrap();
                    if hunt_happened {
                        let mut omni = co1;
                        if co2.data().entity_type == Entity::HERBIVORE {
                            omni = co2;
                        }
                        let record = EventRecord::new(
                            self.counter,
                            -3,
                            hunt_count,
                            Entity::OTHER.to_string(),
                            Entity::OTHER.to_string(),
                            omni.position().translation.x as u64,
                            omni.position().translation.y as u64,
                            0,
                            0,
                        );
                        self.event_writer.write_event_record(record).unwrap();
                    }
                }
            }
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Check to see if enough time has elapsed since our last update based on
        // the update rate we defined.
        let now = Instant::now();
        if now - self.last_update >= Duration::from_millis(self.millis_per_update) {
            if input::keyboard::is_key_pressed(_ctx, KeyCode::H) {
                self.show_details = false;
            }
            if input::keyboard::is_key_pressed(_ctx, KeyCode::S) {
                self.show_details = true;
            }
            self.counter += 1;
            if self.counter % 1000 == 0 {
                println!("Timestep: {}", self.counter);
            }
            if now - self.start >= Duration::from_secs(self.start_recording * 60 - 1)
                && now - self.start
                    <= Duration::from_secs(self.start_recording * 60 + 1) || self.start_recording == 0 && self.counter == 1
            {
                println!("Recording started!");
                println!("Timestep: {}", self.counter);
            }
            if now - self.start >= Duration::from_secs((self.start_recording + self.recording_duration) * 60 - 1)
            && now - self.start
            <= Duration::from_secs((self.start_recording + self.recording_duration) * 60 + 1)
            {
                println!("Recording stopped!");
                println!("Timestep: {}", self.counter);
            }
            if now - self.start >= Duration::from_secs(self.start_recording * 60)
                && now - self.start
                    <= Duration::from_secs((self.start_recording + self.recording_duration) * 60)
            {
                self.recording = true;
            } else {
                self.recording = false;
            }

            
            let mut omni_health_avg = 0.0;
            let mut omni_score_avg = 0.0;
            let mut carn1_score_avg = 0.0;
            let mut carn1_health_avg = 0.0;
            let mut carn2_score_avg = 0.0;
            let mut carn2_health_avg = 0.0;
            let mut top_omni_health_avg = 0.0;
            let mut top_omni_score_avg = 0.0;
            let mut top_carn1_score_avg = 0.0;
            let mut top_carn1_health_avg = 0.0;
            let mut top_carn2_score_avg = 0.0;
            let mut top_carn2_health_avg = 0.0;

            let threshold = self.herbivores.len() / 10;
            for i in 0..self.herbivores.len() {
                let new_x;
                let new_y;
                {
                    let new_pos = self.world.collision_object(
                        self.herbivores[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_handle()
                    ).unwrap().position();
                    let (x, y) = self.random.random_coordinate();
                    new_x = (new_pos.translation.x * 2.0 + x) / 3.0;
                    new_y = (new_pos.translation.y * 2.0 + y) / 3.0;
                }
                
                let mut brain = self.herbivores[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                let wall_brain = self.herbivores[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.wall_network = wall_brain.wall_network;
                let carnivore_brain = self.herbivores[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.carnivore_network = carnivore_brain.carnivore_network;
                let walker = & mut self.herbivores[i];
                omni_health_avg += walker.get_health() as f32;
                omni_score_avg += walker.get_score() as f32;
                if walker.is_dead(&self.world) {
                    if i < threshold {
                        brain = walker.get_brain();
                        walker.respawn(&mut self.world, new_x, new_y, false, brain);
                    } else {
                        walker.respawn(&mut self.world, new_x, new_y, true, brain);
                    }
                    
                } else {
                    walker.update(&mut self.world);
                }
                if i < threshold {
                    top_omni_health_avg += walker.get_health() as f32;
                    top_omni_score_avg += walker.get_score() as f32;
                }
                if self.recording && self.record_all_details {
                    let pos = self.world.collision_object(walker.get_handle()).unwrap().position();
                    let record = GamestateRecord::new(
                        self.counter,
                        pos.translation.x,
                        pos.translation.y,
                        walker.get_id() as u64,
                        walker.get_health(),
                        walker.get_score(),
                        "HERBIVORE"
                    );
                    self.simulation_writer.write_gamestate_record(record).unwrap();
                }
            }
            top_omni_health_avg /= threshold as f32;
            top_omni_score_avg /= threshold as f32;
            let threshold = self.carnivores_1.len() / 10;
            for i in 0..self.carnivores_1.len() {
                let new_x;
                let new_y;
                {
                    let new_pos = self.world.collision_object(
                        self.carnivores_1[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_handle()
                    ).unwrap().position();
                    let (x, y) = self.random.random_coordinate();
                    new_x = (new_pos.translation.x * 2.0 + x) / 3.0;
                    new_y = (new_pos.translation.y * 2.0 + y) / 3.0;
                }
                let mut brain = self.carnivores_1[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                let wall_brain = self.carnivores_1[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.wall_network = wall_brain.wall_network;
                let carnivore_brain = self.carnivores_1[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.carnivore_network = carnivore_brain.carnivore_network;
                let walker = &mut self.carnivores_1[i];
                carn1_health_avg += walker.get_health() as f32;
                carn1_score_avg += walker.get_score() as f32;
                if walker.is_dead(&self.world) {
                    if i < threshold {
                        brain = walker.get_brain();
                        walker.respawn(&mut self.world, new_x, new_y, false, brain);
                    } else {
                        walker.respawn(&mut self.world, new_x, new_y, true, brain);
                    }
                } else {
                    walker.update(&mut self.world);
                }
                if i < threshold {
                    top_carn1_health_avg += walker.get_health() as f32;
                    top_carn1_score_avg += walker.get_score() as f32;
                }
                if self.recording && self.record_all_details {
                    let pos = self.world.collision_object(walker.get_handle()).unwrap().position();
                    let record = GamestateRecord::new(
                        self.counter,
                        pos.translation.x,
                        pos.translation.y,
                        walker.get_id() as u64,
                        walker.get_health(),
                        walker.get_score(),
                        "CARNIVORE"
                    );
                    self.simulation_writer.write_gamestate_record(record).unwrap();
                }
            }
            top_carn1_health_avg /= threshold as f32;
            top_carn1_score_avg /= threshold as f32;
            let threshold = self.carnivores_2.len() / 10;
            for i in 0..self.carnivores_2.len() {
                let new_x;
                let new_y;
                {
                    let new_pos = self.world.collision_object(
                        self.carnivores_2[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_handle()
                    ).unwrap().position();
                    let (x, y) = self.random.random_coordinate();
                    new_x = (new_pos.translation.x * 2.0 + x) / 3.0;
                    new_y = (new_pos.translation.y * 2.0 + y) / 3.0;
                }
                let mut brain = self.carnivores_2[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                let wall_brain = self.carnivores_2[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.wall_network = wall_brain.wall_network;
                let carnivore_brain = self.carnivores_2[self.random.random_between(0.0, threshold as f32 + 0.1) as usize].get_brain();
                brain.carnivore_network = carnivore_brain.carnivore_network;
                let walker = &mut self.carnivores_2[i];
                carn2_health_avg += walker.get_health() as f32;
                carn2_score_avg += walker.get_score() as f32;
                if walker.is_dead(&self.world) {
                    if i < threshold {
                        brain = walker.get_brain();
                        walker.respawn(&mut self.world, new_x, new_y, false, brain);
                    } else {
                        walker.respawn(&mut self.world, new_x, new_y, true, brain);
                    }
                } else {
                    walker.update(&mut self.world);
                }
                if i < threshold {
                    top_carn2_health_avg += walker.get_health() as f32;
                    top_carn2_score_avg += walker.get_score() as f32;
                }
                if self.recording && self.record_all_details {
                    let pos = self.world.collision_object(walker.get_handle()).unwrap().position();
                    let record = GamestateRecord::new(
                        self.counter,
                        pos.translation.x,
                        pos.translation.y,
                        walker.get_id() as u64,
                        walker.get_health(),
                        walker.get_score(),
                        "CARNIVORE"
                    );
                    self.simulation_writer.write_gamestate_record(record).unwrap();
                }
            }
            top_carn2_health_avg /= threshold as f32;
            top_carn2_score_avg /= threshold as f32;
            omni_health_avg /= self.herbivores.len() as f32;
            omni_score_avg /= self.herbivores.len() as f32;
            carn1_health_avg /= self.carnivores_1.len() as f32;
            carn1_score_avg /= self.carnivores_1.len() as f32;
            carn2_health_avg /= self.carnivores_2.len() as f32;
            carn2_score_avg /= self.carnivores_2.len() as f32;
            self.herbivores
                .sort_by(|a, b| b.get_score().cmp(&a.get_score()));
            self.carnivores_1
                .sort_by(|a, b| b.get_score().cmp(&a.get_score()));
            self.carnivores_2
                .sort_by(|a, b| b.get_score().cmp(&a.get_score()));
            
            self.best_herbivore_score = 0;
            self.best_carnivore_score = 0;
            if self.herbivores.len() > 0 {
                self.best_herbivore_score = self.herbivores[0].get_score();
            }
            if self.carnivores_1.len() > 0 {
                self.best_carnivore_score = self.carnivores_1[0].get_score();
            }
            if self.carnivores_2.len() > 0 {
                self.best_carnivore_score = std::cmp::max(self.best_carnivore_score, self.carnivores_2[0].get_score());
            }

            self.world.update();
            self.handle_proximity_event();
            for food in self.foods.iter_mut() {
                food.update(&mut self.world, &mut self.random);
            }
            self.world.update();

            let record = AverageRecord::new(
                self.counter,
                omni_score_avg,
                carn1_score_avg,
                carn2_score_avg,
                omni_health_avg,
                carn1_health_avg,
                carn2_health_avg,
                top_omni_score_avg,
                top_carn1_score_avg,
                top_carn2_score_avg,
                top_omni_health_avg,
                top_carn1_health_avg,
                top_carn2_health_avg,
            );
            self.average_writer.write_average_record(record).unwrap();

            // If we updated, we set our last_update to be now
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        for wall in self.walls.iter_mut() {
            wall.draw(ctx)?;
        }
        for food in self.foods.iter_mut() {
            food.draw(ctx, &mut self.world)?;
        }
        let mut i = 0;
        let mut top = true;
        let threshold = self.herbivores.len() / 10;
        for walker in self.herbivores.iter_mut() {
            if i >= threshold {top = false}
            i += 1;
            walker.draw(ctx, &mut self.world, top, self.show_details)?;
        }
        top = true;
        i = 0;
        let threshold = self.carnivores_1.len() / 10;
        for walker in self.carnivores_1.iter_mut() {
            if i >= threshold {top = false}
            i += 1;
            walker.draw(ctx, &mut self.world, top, self.show_details)?;
        }
        top = true;
        i = 0;
        let threshold = self.carnivores_2.len() / 10;
        for walker in self.carnivores_2.iter_mut() {
            if i >= threshold {top = false}
            i += 1;
            walker.draw(ctx, &mut self.world, top, self.show_details)?;
        }
        // Drawing the best score
        let omni_score = graphics::Text::new((
            "BEST HERBIVORE: ".to_owned() + &self.best_herbivore_score.to_string(),
            graphics::Font::default(),
            24.0,
        ));
        let carn_score = graphics::Text::new((
            "BEST CARNIVORE: ".to_owned() + &self.best_carnivore_score.to_string(),
            graphics::Font::default(),
            24.0,
        ));
        if self.show_details {
            graphics::draw(
                ctx,
                &omni_score,
                (
                    ggez::nalgebra::Point2::new(10.0, 10.0),
                    0.0,
                    graphics::WHITE,
                ),
            )?;
            graphics::draw(
                ctx,
                &carn_score,
                (
                    ggez::nalgebra::Point2::new(10.0, 35.0),
                    0.0,
                    graphics::WHITE,
                ),
            )?;
        }
        // Display the new frame
        graphics::present(ctx)?;
        // Yield the current thread until the next update
        ggez::timer::yield_now();
        // Return success.
        Ok(())
    }
}
