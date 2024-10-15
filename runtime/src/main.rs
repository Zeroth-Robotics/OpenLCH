mod controller;
mod model;

use anyhow::{Context, Result};
use runtime::hal;
use std::path::PathBuf;
use std::env;
use runtime::model::Model;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize robot
    let robot = robot::Robot::new().context("Failed to initialize robot")?;

    // load model
    let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    let model = Model::new(model_path).context("Failed to load model")?;
    let model_arc = Arc::new(model);

    // run controller
    controller::run_controller(model_arc, robot).await.context("Controller encountered an error")
}
