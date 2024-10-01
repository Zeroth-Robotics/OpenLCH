use anyhow::{Context, Result};
use ndarray::{Array1, Array2};
use onnxruntime::{environment::Environment, session::Session, tensor::OrtOwnedTensor};
use std::path::Path;

pub struct Model {
    session: Session,
}

impl Model {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        println!("Loading model from: {:?}", model_path.as_ref());
        let environment = Environment::builder()
            .with_name("robot_control_env")
            .build()?;

        let session = environment
            .new_session_builder()?
            .with_model_from_file(model_path)
            .context("Failed to load the ONNX model")?;

        println!("Model loaded successfully");
        println!("Input count: {}", session.inputs.len());
        for (i, input) in session.inputs.iter().enumerate() {
            println!("Input {}: {:?}", i, input);
        }
        println!("Output count: {}", session.outputs.len());
        for (i, output) in session.outputs.iter().enumerate() {
            println!("Output {}: {:?}", i, output);
        }

        Ok(Model { session })
    }

    pub fn infer(&self, input: Array1<f32>) -> Result<Array1<f32>> {
        println!("Running inference with input shape: {:?}", input.shape());
        let input_shape = vec![1, input.len()];
        let input_tensor = input.into_shape(input_shape)?;

        let outputs: Vec<OrtOwnedTensor<f32, _>> = self
            .session
            .run(vec![input_tensor.view()])
            .context("Failed to run inference")?;

        let output = outputs[0].view().to_owned().into_shape(outputs[0].len())?;
        println!("Inference completed. Output shape: {:?}", output.shape());
        Ok(output)
    }
}
