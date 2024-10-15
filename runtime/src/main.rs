mod controller;
mod model;

use anyhow::Result;
use controller::Robot;
use model::Model;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize robot
    let robot = Arc::new(Robot::new()?);

    // load model
    let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    let model = Arc::new(Model::new(model_path)?);

    // run controller
    controller::run(model, robot).await
}
