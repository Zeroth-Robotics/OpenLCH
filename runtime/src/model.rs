use anyhow::{Context, Result};
use ndarray::{Array1, Array2};
use onnxruntime::{environment::Environment, session::Session, tensor::OrtOwnedTensor};
use std::path::Path;

pub struct Model {
    session: Session,
}

impl Model {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let environment = Environment::builder()
            .with_name("robot_control_env")
            .build()?;

        let session = environment
            .new_session_builder()?
            .with_model_from_file(model_path)
            .context("Failed to load the ONNX model")?;

        Ok(Model { session })
    }

    pub fn infer(&self, input: Array1<f32>) -> Result<Array1<f32>> {
        let input_shape = vec![1, input.len()];
        let input_tensor = input.into_shape(input_shape)?;

        let outputs: Vec<OrtOwnedTensor<f32, _>> = self
            .session
            .run(vec![input_tensor.view()])
            .context("Failed to run inference")?;

        let output = outputs[0].view().to_owned().into_shape(outputs[0].len())?;
        Ok(output)
    }
}
