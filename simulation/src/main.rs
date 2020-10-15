use ggez;
use ggez::{event, GameResult};
#[macro_use]
extern crate serde_derive;
use crate::gamestate::GameState;
use crate::config::read_config_from_file;
mod gamestate;
mod food;
mod randomwalker;
mod entity;
mod collisionobjectdata;
mod random_helper;
mod config;
mod brain;
mod wall;
mod record;


fn main() -> GameResult {

    let configs = read_config_from_file().unwrap();

    // Setup metadata
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Cooperative simulation", "Krisztián Varga")
        // Set up the window
        .window_setup(ggez::conf::WindowSetup::default().title("Cooperative Simulation"))
        // Setting the size of the window
        .window_mode(ggez::conf::WindowMode::default().dimensions(
            configs.0["screen_size_x"].as_f64().unwrap() as f32,
            configs.0["screen_size_y"].as_f64().unwrap() as f32))
        .build()?;

    // New instance of the simulation
    let state = &mut GameState::new(configs);
    // Run the simulation
    event::run(ctx, events_loop, state)
}