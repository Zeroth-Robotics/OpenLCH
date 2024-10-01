mod controller;
mod robot;

use crate::controller::*;
use anyhow::Result;
use robot::Robot;

#[tokio::main]
async fn main() -> Result<()> {
    let robot = Robot::new();
    let controller = StandingControllerPID::new(robot);
    controller.run().await
}
