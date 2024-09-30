// model inference

use ndarray::{Array1, ArrayView1};
use onnxruntime::{environment::Environment, session::Session, tensor::OrtOwnedTensor};
use std::time::Instant;
use clap::Parser;

// onnx inference session
struct OnnxInfer {
    session: Session,
    input_name: String,
}

impl OnnxInfer {
    fn new(onnx_model_path: &str, input_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let environment = Environment::builder().build()?;
        let session = environment.new_session_builder()?
            .with_model_from_file(onnx_model_path)?;
        
        Ok(OnnxInfer {
            session,
            input_name: input_name.to_string(),
        })
    }

    fn infer(&self, inputs: ArrayView1<f32>) -> Result<Array1<f32>, Box<dyn std::error::Error>> {
        let input_tensor = inputs.into_dyn().into_tensor();
        let outputs: Vec<OrtOwnedTensor<f32, _>> = self.session.run(vec![input_tensor])?;
        Ok(Array1::from_vec(outputs[0].view().as_slice().unwrap().to_vec()))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    onnx_model_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let oi = OnnxInfer::new(&args.onnx_model_path, "obs")?;

    let inputs = Array1::linspace(0., 54., 55);
    let mut times = Vec::new();

    for _ in 0..1000 {
        let start = Instant::now();
        let output = oi.infer(inputs.view())?;
        println!("{:?}", output);
        times.push(start.elapsed().as_secs_f64());
    }

    let average_time = times.iter().sum::<f64>() / times.len() as f64;
    println!("Average time: {} seconds", average_time);
    println!("Average fps: {}", 1.0 / average_time);

    Ok(())
}

