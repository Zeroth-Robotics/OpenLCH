use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use ort::{Environment, Session, SessionBuilder, Value};
use ndarray::{Array, CowArray, IxDyn};

#[cfg(feature = "milkv")]
mod milkv_model {
    use super::*;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_float, c_int};

    #[link(name = "cviwrapper")]
    extern "C" {
        fn init_model(model_path: *const c_char) -> c_int;
        fn forward(input_data: *const c_float, output_data: *mut c_float) -> c_int;
        fn cleanup();
        fn get_input_size() -> usize;
        fn get_output_size() -> usize;
    }

    pub struct Model {
        _private: (), // Prevent direct construction
    }

    impl Model {
        pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
            println!("Loading MilkV model from: {:?}", model_path.as_ref());
            let c_model_path = CString::new(model_path.as_ref().to_str().unwrap())?;
            let result = unsafe { init_model(c_model_path.as_ptr()) };
            if result != 0 {
                anyhow::bail!("Failed to initialize MilkV model");
            }
            Ok(Model { _private: () })
        }

        pub fn infer(&self, input: &[f32]) -> Result<Vec<f32>> {
            let input_size = unsafe { get_input_size() };
            let output_size = unsafe { get_output_size() };

            if input.len() != input_size / std::mem::size_of::<f32>() {
                anyhow::bail!("Input size mismatch");
            }

            let mut output = vec![0.0f32; output_size / std::mem::size_of::<f32>()];

            let result = unsafe { forward(input.as_ptr(), output.as_mut_ptr()) };

            if result != 0 {
                anyhow::bail!("Forward pass failed");
            }

            Ok(output)
        }
    }

    impl Drop for Model {
        fn drop(&mut self) {
            unsafe { cleanup() };
        }
    }
}

#[cfg(not(feature = "milkv"))]
mod onnx_model {
    use super::*;
    use ort::{Environment, Session, SessionBuilder, Value};
    use ndarray::{Array, CowArray, IxDyn};

    pub struct Model {
        session: Arc<Session>,
        environment: Arc<Environment>,
    }

    impl Model {
        pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
            println!("Loading ONNX model from: {:?}", model_path.as_ref());
            let environment = Environment::builder().build()?;
            let environment = Arc::new(environment);
            
            let session = SessionBuilder::new(&environment)?
                .with_model_from_file(model_path)?;
            let session = Arc::new(session);

            Ok(Model { session, environment })
        }

        pub fn infer(&self, input: &[f32]) -> Result<Vec<f32>> {
            let input_shape: Vec<usize> = self.session.inputs[0]
                .dimensions()
                .map(|d| d.unwrap_or(1))
                .collect();

            let array = Array::from_shape_vec(IxDyn(&input_shape), input.to_vec())?;
            let input_tensor: CowArray<f32, IxDyn> = CowArray::from(array);
            
            let inputs = vec![Value::from_array(self.session.allocator(), &input_tensor)?];
            
            let outputs: Vec<Value> = self.session.run(inputs)?;
            
            let output: ort::tensor::OrtOwnedTensor<f32, _> = outputs[0].try_extract()?;
            let output_vec = output.view().to_owned().as_slice().unwrap().to_vec();
            
            Ok(output_vec)
        }
    }
}

#[cfg(feature = "milkv")]
pub use milkv_model::Model;

#[cfg(not(feature = "milkv"))]
pub use onnx_model::Model;
