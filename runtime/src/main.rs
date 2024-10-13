mod controller;
mod model;
mod robot;
use anyhow::Result;
use rand::Rng;
use half::f16;
use std::path::PathBuf;


fn main() -> Result<()> {

    println!("Initializing robot...");

    // let robot = robot::Robot::new(config_path).context("Failed to initialize robot")?;
    // println!("Robot initialized. Printing configuration:");
    // robot.print_config();

    println!("Robot initialized.");

    println!("Loading model...");
    let model_path = PathBuf::from("/root/models/ppo_walking.cvimodel"); // PATH IN MILK-V
    let model = model::Model::new(model_path)?;

    println!("Model loaded.");

    // generate 615 random float32 values with reduced precision
    let mut rng = rand::thread_rng();
    let input: Vec<f32> = (0..615).map(|_| {
        let f32_value: f32 = rng.gen();
        // Convert to f16 and back to f32 to simulate float16 precision
        f16::from_f32(f32_value).to_f32()
    }).collect();

     // run inference
     let output = model.infer(&input)?;

     println!("Inference completed. Output size: {}", output.len());
     println!("First 5 output values: {:?}", &output[..5.min(output.len())]);


    println!("Controller started...");
    // controller::run(); // TODO pass model or other controller parameters
    
    Ok(())
}
