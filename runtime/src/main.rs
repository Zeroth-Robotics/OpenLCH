mod controller;
mod model;
mod robot;
use anyhow::Result;
use rand::Rng;
use half::f16;
use std::path::PathBuf;


fn main() -> Result<()> {

    println!("Initializing robot...");

    // let robot = robot::Robot::new(config_path).context("Failed to initialize robot")?;
    // println!("Robot initialized. Printing configuration:");
    // robot.print_config();

    println!("Robot initialized.");

    println!("Loading model...");

    let model_path = PathBuf::from("/root/standing.cvi");
    let model = model::Model::new(model_path)?;

    println!("Model loaded.");

    println!("Controller started...");
    controller::run(); // TODO, pass model or other controller parameters
    
    Ok(())
}
