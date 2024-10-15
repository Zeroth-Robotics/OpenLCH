use crate::model::Model;
use crate::robot::Robot;
use anyhow::{Context, Result};
use ndarray::Array1;
use ort::Environment;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;




// ===== Test Convert Joint Positions and Velocities int ===== //




// ===== Test Model Inference ===== //
#[tokio::test]
async fn test_inference() -> Result<()> {
    println!("TEST INFERENCE");

    // load model
    println!("loading model");
    let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    let model = Model::new(model_path).context("Failed to load model")?;
    let model = Arc::new(model);

    // initialize robot
    println!("Initializing robot...");
    let mut robot = Robot::new().context("Failed to create Robot instance")?;
    robot.initialize().await.context("Failed to initialize Robot")?;

    // initialize input tensors (example values)
    let x_vel = Array1::<f32>::zeros(1);
    let y_vel = Array1::<f32>::zeros(1);
    let rot = Array1::<f32>::zeros(1);
    let mut t = Array1::<f32>::zeros(1);
    let dof_pos = Array1::<f32>::zeros(10);
    let dof_vel = Array1::<f32>::zeros(10);
    let mut prev_actions = Array1::<f32>::zeros(10);
    let imu_ang_vel = Array1::<f32>::zeros(3);
    let imu_euler_xyz = Array1::<f32>::zeros(3);
    let mut buffer = Array1::<f32>::zeros(574);

    // combine arrays into a single input vector
    let input: Vec<f32> = [
        &x_vel.to_vec()[..],
        &y_vel.to_vec()[..],
        &rot.to_vec()[..],
        &t.to_vec()[..],
        &dof_pos.to_vec()[..],
        &dof_vel.to_vec()[..],
        &prev_actions.to_vec()[..],
        &imu_ang_vel.to_vec()[..],
        &imu_euler_xyz.to_vec()[..],
        &buffer.to_vec()[..],
    ].concat();

    println!("Input: {:?}", input);

    // run inference
    let output = model.infer(&input).context("Failed to run model inference")?;

    println!("Output: {:?}", output);

    Ok(())
}
