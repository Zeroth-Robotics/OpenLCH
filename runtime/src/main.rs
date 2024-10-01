// src/main.rs

mod HAL;
mod controller;
mod model;
mod robot;

use anyhow::{Context, Result};
use candle_core::Device;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting robot initialization");

    let config_path = PathBuf::from("config/stompymicro.toml");
    println!("Loading config from: {:?}", config_path);

    let robot = robot::Robot::new(config_path).context("Failed to initialize robot")?;

    println!("Robot initialized. Printing configuration:");
    robot.print_config();

    println!("Initializing controller"); // Onnx or PID

    // Onnx
    // let model_path = PathBuf::from("path/to/model.onnx");
    // let device = Device::Cpu; // or Device::Cuda(0) for GPU
    // let model = model::Model::new(model_path, &device)?;
    // let controller: Arc<dyn controller::Controller> =
    //     Arc::new(controller::MLController::new(model));

    // PID
    let controller: Arc<dyn controller::Controller + Send + Sync> =
        Arc::new(controller::PIDController {});
    println!("Creating StandingController");
    let mut standing_controller = controller::StandingController::new(robot, controller);

    println!("Starting controller");
    let iterations = Some(10); // Run for 10 iterations, None for infinite
    standing_controller
        .run(iterations)
        .await
        .context("Controller run failed")?;

    println!("Controller finished running");
    Ok(())
}
