// src/main.rs

mod HAL;
mod controller;
mod model;
mod robot;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use rand::Rng;
use half::f16;

fn main() -> Result<()> {
    println!("Starting model test");

    // Initialize the model
    let model_path = PathBuf::from("/root/standing.cvimodel");
    println!("Loading model from: {:?}", model_path);

    let model = model::Model::new(model_path).context("Failed to initialize model")?;

    // Generate 615 random float32 values with reduced precision
    let mut rng = rand::thread_rng();
    let input: Vec<f32> = (0..615).map(|_| {
        let f32_value: f32 = rng.gen();
        // Convert to f16 and back to f32 to simulate float16 precision
        f16::from_f32(f32_value).to_f32()
    }).collect();

    println!("Running inference with random input");

    // Run inference
    let output = model.infer(&input).context("Failed to run inference")?;

    println!("Inference completed. Output size: {}", output.len());
    println!("First 5 output values: {:?}", &output[..5.min(output.len())]);

    Ok(())
}
