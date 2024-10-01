use anyhow::Result;
use candle_core::{Device, Tensor};
use candle_onnx::onnx::ModelProto;
use std::collections::HashMap;
use std::path::Path;

pub struct Model {
    proto: ModelProto,
    device: Device,
}

impl Model {
    pub fn new<P: AsRef<Path>>(model_path: P, device: &Device) -> Result<Self> {
        println!("Loading model from: {:?}", model_path.as_ref());
        let proto = candle_onnx::read_file(model_path)?;
        Ok(Model {
            proto,
            device: device.clone(),
        })
    }

    pub fn infer(&self, input: &[f32]) -> Result<Vec<f32>> {
        let input_tensor = Tensor::from_vec(input.to_vec(), (1, input.len()), &self.device)?;

        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), input_tensor);

        let outputs = candle_onnx::simple_eval(&self.proto, inputs)?;

        // Assuming the output tensor is named "output"
        let output = outputs.get("output").expect("Output 'output' not found");
        let output_vec = output.to_vec1::<f32>()?;

        Ok(output_vec)
    }
}
