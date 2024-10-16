mod controller;
mod model;
mod robot;

use crate::robot::Robot;
use crate::model::Model;
use anyhow::{Context, Result};
// use runtime::hal; 
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    // // load model
    // let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    // let model = Model::new(model_path).context("Failed to load model")?;
    // let model = Arc::new(model);

    // // initialize robot
    // let robot = Robot::new()?;
    // let robot = Arc::new(Mutex::new(robot));
    // robot.lock().unwrap().initialize().await?;

    // // run controller
    // controller::run(model, robot).await.context("Controller encountered an error")

    Ok(())
}