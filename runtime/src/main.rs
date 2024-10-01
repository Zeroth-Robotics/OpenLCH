mod controller;
mod robot;

use crate::controller::*;
use anyhow::Result;
use robot::Robot;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = PathBuf::from("config/stompymicro.toml");
    let robot = Robot::new(config_path)?;

    println!("Robot initialized. Printing configuration:");
    robot.print_config();

    let mut controller = StandingControllerPID::new(robot);
    controller.run().await
}
