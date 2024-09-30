// load configuration and run controller

mod config;
mod controller;
mod robot;
mod utils;


use anyhow::Result;


#[tokio::main]
async fn main() -> Result<()> {

    // robot config
    let config = Config::load("config.toml");

    // PPO controller
    let mut controller = WalkController::new(config)?;

    let mut robot = Robot::new();

    WalkController::run(&controller).await
}


