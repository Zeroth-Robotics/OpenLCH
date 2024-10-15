use anyhow::Result;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};
use ndarray::Array1;
use runtime::model::Model;
use ort::tensor::InputTensor;
use ort::OrtOwnedTensor;

pub struct Robot {
    servo: Servo,
    x_vel: Array1<f32>,
    y_vel: Array1<f32>,
    rot: Array1<f32>,
    t: Array1<f32>,
    dof_pos: Array1<f32>,
    dof_vel: Array1<f32>,
    prev_actions: Array1<f32>,
    imu_ang_vel: Array1<f32>,
    imu_euler_xyz: Array1<f32>,
    buffer: Array1<f32>,
    prev_dof_pos: Array1<f32>,
}

impl Robot {
    pub fn new() -> Result<Self> {
        let servo = Servo::new()?;

        Ok(Self {
            servo,
            x_vel: Array1::<f32>::zeros(1),
            y_vel: Array1::<f32>::zeros(1),
            rot: Array1::<f32>::zeros(1),
            t: Array1::<f32>::zeros(1),
            dof_pos: Array1::<f32>::zeros(10),
            dof_vel: Array1::<f32>::zeros(10),
            prev_actions: Array1::<f32>::zeros(10),
            imu_ang_vel: Array1::<f32>::zeros(3),
            imu_euler_xyz: Array1::<f32>::zeros(3),
            buffer: Array1::<f32>::zeros(574),
            prev_dof_pos: Array1::<f32>::zeros(10),
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.servo.enable_readout()
    }

    pub async fn get_joint_states(&mut self) -> Result<[f32; 16]> {
        let servo_data = self.servo.read_continuous()?;
        let joint_states: [f32; 16] = servo_data.servo.iter()
            .take(16)
            .map(|s| s.target_location as f32)
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap_or([0.0; 16]);

        // Update dof_pos
        self.dof_pos.assign(&Array1::from_vec(joint_states[0..10].to_vec()));
        
        // calculate dof_vel
        let new_dof_vel = (&self.dof_pos - &self.prev_dof_pos) / 0.02; // 20ms interval (50HZ)
        self.dof_vel.assign(&new_dof_vel);
        self.prev_dof_pos.assign(&self.dof_pos);

        // set x_vel, y_vel, rot, and IMU-related fields to 0
        self.x_vel.fill(0.0);
        self.y_vel.fill(0.0);
        self.rot.fill(0.0);
        self.imu_ang_vel.fill(0.0);
        self.imu_euler_xyz.fill(0.0);

        // increment time
        self.t[0] += 0.02;

        Ok(joint_states)
    }

    pub async fn model_inference(&mut self, model: &Model) -> Result<[f32; 16]> {
        // Prepare the inputs
        let inputs = ort::inputs![
            "x_vel.1" => self.x_vel.clone(),
            "y_vel.1" => self.y_vel.clone(),
            "rot.1" => self.rot.clone(),
            "t.1" => self.t.clone(),
            "dof_pos.1" => self.dof_pos.clone(),
            "dof_vel.1" => self.dof_vel.clone(),
            "prev_actions.1" => self.prev_actions.clone(),
            "imu_ang_vel.1" => self.imu_ang_vel.clone(),
            "imu_euler_xyz.1" => self.imu_euler_xyz.clone(),
            "buffer.1" => self.buffer.clone(),
        ]?;

        // Run the model inference
        let outputs = model.session.run(inputs)?;

        // Extract outputs
        let actions = outputs[1].try_extract::<f32>()?;
        let next_buffer = outputs[2].try_extract::<f32>()?;

        // update `prev_actions` and `buffer`
        self.prev_actions.assign(&actions);
        self.buffer.assign(&next_buffer);

        // get the desired joint positions
        let desired_joint_positions = outputs[0].try_extract::<f32>()?;

        // convert the desired joint positions to an array of length 16
        let desired_positions_array: [f32; 16] = desired_joint_positions
            .as_slice()
            .unwrap()
            .try_into()
            .unwrap_or([0.0; 16]);

        Ok(desired_positions_array)
    }

    pub async fn send_joint_commands(&self, positions: &[f32; 16]) -> Result<()> {
        let mut cmd = ServoMultipleWriteCommand {
            ids: [0; MAX_SERVOS],
            positions: [0; MAX_SERVOS],
            times: [0; MAX_SERVOS],
            speeds: [0; MAX_SERVOS],
            only_write_positions: 0,
        };

        for i in 0..16 {
            cmd.ids[i] = (i + 1) as u8;
            cmd.positions[i] = positions[i] as i16;
            cmd.times[i] = 20;
        }

        self.servo.write_multiple(&cmd)?;

        println!("Command sent to move all servos to position {} with time {} ms and speed {}, send_only_positions: {}", positions[0] as i16, 20, 0, 0);
        Ok(())
    }
}
