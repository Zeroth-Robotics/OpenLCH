use crate::model::Controller;
use crate::robot::Robot;
use anyhow::Result;
use onnxruntime::ndarray::Array1;
use std::time::Instant;
use tokio::time::{sleep, Duration};

pub struct StandingControllerPID {
    robot: Robot,
    controller: Controller,
}

impl StandingControllerPID {
    pub fn new(robot: Robot, controller: Controller) -> Self {
        Self { robot, controller }
    }

    pub async fn get_state(&self) -> Result<Array1<f32>> {
        // ### === TODO: DENYS === ###
        // let state = self.robot.joint_states().await;
        // let imu = self.robot.imu_state().await;
        // let action = self.model.infer(state, imu).await?;
        Ok(Array1::zeros(10)) // Placeholder
    }

    // ### === TODO: DENYS === ###
    pub async fn get_feedback(&self) -> Result<()> {
        let time = Instant::now();
        println!("feedback");
        println!("Start time: {:?}", time);
        Ok(())
    }

    // ### === TODO: DENYS === ###
    pub async fn send_command(&self) -> Result<()> {
        let time = Instant::now();
        println!("command");
        println!("Start time: {:?}", time);
        Ok(())
    }

    pub async fn stand(&mut self) -> Result<()> {
        let desired_positions = self.robot.get_default_standing_positions();
        let mut commands = Vec::new();

        for (joint_name, &position) in desired_positions {
            if let Some(joint_config) = self.find_joint_config(joint_name) {
                commands.push((joint_config.id, position));
            }
        }

        for (joint_id, position) in commands {
            self.robot.set_joint_command(joint_id, position, 0.0);
        }

        Ok(())
    }

    fn find_joint_config(&self, joint_name: &str) -> Option<&crate::robot::JointConfig> {
        for limb in [&self.robot.config.legs, &self.robot.config.arms] {
            for side_joints in limb.values() {
                if let Some(joint_config) = side_joints.get(joint_name) {
                    return Some(joint_config);
                }
            }
        }
        None
    }

    pub async fn run(&mut self) -> Result<()> {
        // get joint states
        // get imu state
        // send command

        println!("Starting StandingControllerPID");
        loop {
            let feedback_future = self.get_feedback();
            let command_future = self.send_command();

            tokio::try_join!(feedback_future, command_future)?;

            sleep(Duration::from_millis(100)).await;
        }
    }
}

// pub struct StandingControllerPPO {
//     model: OnnxInfer,
//     robot: Robot,
//     config: Config,
// }

// impl StandingControllerPPO {
//     pub fn new(model: OnnxInfer, robot: Robot, config: Config) -> Self {
//         Self {
//             model,
//             robot,
//             config,
//         }
//     }

//     pub fn get_state(&self) -> Result<()> {
//         let state = self.robot.joint_states();
//         let imu = self.robot.imu_state();
//         let action = self.model.infer(state, imu)?;
//         Ok(())
//     }

//     pub fn send_commands(&self) -> Result<()> {
//         Ok(())
//     }

//     pub fn run(&self) -> Result<()> {
//         Ok(())
//     }

// }
