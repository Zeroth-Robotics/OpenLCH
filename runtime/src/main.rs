mod controller;
mod robot;

use crate::controller::*;
use anyhow::Result;
use robot::Robot;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = PathBuf::from("config.toml");
    let robot = Robot::new(config_path)?;
    let controller = StandingControllerPID::new(robot);
    controller.run().await
}
