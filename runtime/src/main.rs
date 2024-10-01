mod HAL;
mod controller;
mod model;
mod robot;

use crate::controller::*;
use crate::model::Controller;
use anyhow::Result;
use robot::Robot;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting robot initialization");
    let config_path = PathBuf::from("config/stompymicro.toml");
    let robot = Robot::new(config_path)?;

    println!("Robot initialized. Printing configuration:");
    robot.print_config();

    // PPO
    let model_path = PathBuf::from("path/model.onnx");
    let controller = Controller::new_ppo(model_path)?;

    // Handwritten
    // let controller = Controller::new_handwritten();

    println!("Creating StandingControllerPID");
    let mut standing_controller = StandingControllerPID::new(robot, controller);
    standing_controller.run().await
}
