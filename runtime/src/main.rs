mod controller;
mod model;

use anyhow::{Context, Result};
use controller::Robot;
use model::Model;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize robot
    let robot = Arc::new(Robot::new().context("Failed to initialize robot")?);

    // load model
    let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    let model = Arc::new(Model::new(model_path).context("Failed to load model")?);

    // run controller
    controller::run(model, robot)
        .await
        .context("Controller encountered an error")
}
