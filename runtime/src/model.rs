use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

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
    use ort::{tensor::InputTensor, tensor::OrtOwnedTensor, Environment, GraphOptimizationLevel, Session};
    use ndarray::{Array1, Array, IxDyn};
    use std::sync::Arc;

    pub struct Model {
        session: Arc<Session>,
    }

    impl Model {
        pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
            println!("Loading ONNX model from: {:?}", model_path.as_ref());
            let environment = Environment::builder().build()?;
            let session = Session::builder()?
                .with_environment(&environment)?
                .with_optimization_level(GraphOptimizationLevel::Level3)?
                .with_intra_threads(4)?
                .with_model_from_file(model_path)?;
            let session = Arc::new(session);

            Ok(Model { session })
        }

        pub fn infer(
            &self,
            x_vel: &Array1<f32>,
            y_vel: &Array1<f32>,
            rot: &Array1<f32>,
            t: &Array1<f32>,
            dof_pos: &Array1<f32>,
            dof_vel: &Array1<f32>,
            prev_actions: &Array1<f32>,
            imu_ang_vel: &Array1<f32>,
            imu_euler_xyz: &Array1<f32>,
            buffer: &Array1<f32>,
        ) -> Result<Vec<OrtOwnedTensor<f32, IxDyn>>> {
            let inputs = [
                ("x_vel.1", x_vel),
                ("y_vel.1", y_vel),
                ("rot.1", rot),
                ("t.1", t),
                ("dof_pos.1", dof_pos),
                ("dof_vel.1", dof_vel),
                ("prev_actions.1", prev_actions),
                ("imu_ang_vel.1", imu_ang_vel),
                ("imu_euler_xyz.1", imu_euler_xyz),
                ("buffer.1", buffer),
            ];

            let mut ort_inputs = Vec::new();
            for (name, array) in &inputs {
                let input_tensor = InputTensor::from_array(array.clone());
                ort_inputs.push((*name, input_tensor));
            }

            let outputs = self.session.run(ort_inputs)?;

            Ok(outputs)
        }
    }
}

#[cfg(feature = "milkv")]
pub use milkv_model::Model;

#[cfg(not(feature = "milkv"))]
pub use onnx_model::Model;
