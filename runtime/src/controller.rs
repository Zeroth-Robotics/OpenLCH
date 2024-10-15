use anyhow::Result;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};
use tokio::time::{sleep, interval, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;
use runtime::hal::Model;
use ndarray::Array1;

pub struct Robot{

    servo: Servo,
}

impl Robot{
    pub fn new() -> Result<Self> {
        let servo = Servo::new()?;

        Ok(Self { servo })
    }

    pub async fn run(&self, model: Arc<Model>) -> Result<()> {
        let mut control_interval = interval(Duration::from_millis(20));

        loop {
            control_interval.tick().await;

            // get joint states
            let current_joint_states = self.get_joint_states().await?;
            
            // get desired joint positions (inferenced from model)
            let desired_joint_positions = self.model_inference(&current_joint_states)
            
            // send joint commands
            self.send_joint_commands(desired_joint_positions).await?;
        }
    }

    async fn get_joint_states(&self) -> Result<[f32; 16]> {
        let servo_data = servo.read_continuous()?;
        let joint_states = servo_data.servo.iter().map(|s| s.target_location as f32).collect();
        Ok(joint_states);
    }

    async fn model_inference(&self, model: &Model, joint_states: &[f32; 16]) -> Result<([f32; 16])> {

        //  
        // x_vel: Array1<f32>,
        // y_vel: Array1<f32>,
        // rot: Array1<f32>,
        // t: Array1<f32>,
        // dof_pos: Array1<f32>,
        // dof_vel: Array1<f32>,
        // prev_actions: Array1<f32>,
        // imu_ang_vel: Array1<f32>,
        // imu_euler_xyz: Array1<f32>,
        // buffer: Array1<f32>,


        let model_output = model.infer(&joint_states)?;

        // TODO
        //
        // get model output and convert to desired joint positions
        //
        let desired_joint_positions = model_output.iter().map(|x| x as i16).collect();


        Ok(desired_joint_positions)   
    }

    async fn send_joint_commands(robot: &Robot, position: i16, time: u16, speed: u16, send_only_positions: u8) -> Result<()> {
        let mut interval = interval(Duration::from_millis(20)); // 50Hz
        loop {
            interval.tick().await;
            let mut cmd = ServoMultipleWriteCommand {
                ids: [0; MAX_SERVOS],
                positions: [0; MAX_SERVOS],
                times: [0; MAX_SERVOS],
                speeds: [0; MAX_SERVOS],
                only_write_positions: send_only_positions,
            };

            for i in 0..MAX_SERVOS {
                cmd.ids[i] = (i + 1) as u8;
                cmd.positions[i] = position;
                cmd.times[i] = time;
                cmd.speeds[i] = speed;
            }

            robot.servo.write_multiple(&cmd)?;

            println!("Command sent to move all servos to position {} with time {} ms and speed {}, send_only_positions: {}", position, time, speed, send_only_positions);
        }
    }
}

#[tokio::main]
pub async fn main(model: &Model) -> Result<()> {

    let robot = Robot::new()?;
    robot.servo.enable_readout()?; 
    
    robot.run();
}



