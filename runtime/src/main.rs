mod controller;
mod robot;

use anyhow::Result;
use robot::Robot;
use crate::controller::*;

#[tokio::main]
async fn main() -> Result<()> {
    let robot = Robot::new();
    let controller = StandingControllerPID::new(robot);
    controller.run().await
}


