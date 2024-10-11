mod controller;
mod model;
mod robot;
use anyhow::{Context, Result};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting robot initialization");

    let config_path = PathBuf::from("config/stompymicro.toml");
    println!("Loading config from: {:?}", config_path);

    let robot = robot::Robot::new(config_path).context("Failed to initialize robot")?;

    println!("Robot initialized. Printing configuration:");

    robot.print_config();

    println!("Initializing controller...");

    controller::main()?;
    
    Ok(())
}
