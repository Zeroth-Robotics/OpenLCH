use crate::robot::Robot;
use anyhow::Result;
use tokio::time::{sleep, Duration};

pub struct StandingControllerPID {
    robot: Robot,
}

// pub struct StandingControllerPPO {
//     model: OnnxInfer,
//     robot: Robot,
//     config: Config,
// }

impl StandingControllerPID {
    pub fn new(robot: Robot) -> Self {
        Self { robot }
    }

    pub async fn get_state(&self) -> Result<()> {
        // ### === TODO: DENYS === ###
        // let state = self.robot.joint_states().await;
        // let imu = self.robot.imu_state().await;
        // let action = self.model.infer(state, imu).await?;
        Ok(())
    }

    // ### === TODO: DENYS === ###
    pub async fn get_feedback(&self) -> Result<()> {
        println!("feedback");
        Ok(())
    }

    // ### === TODO: DENYS === ###
    pub async fn send_command(&self) -> Result<()> {
        println!("command");
        Ok(())
    }

    pub async fn stand(&mut self) -> Result<()> {
        let desired_positions = self.default_standing_positions();

        for (joint_id, position) in desired_positions {
            self.robot.set_joint_command(joint_id, position, 0.0);
        }

        Ok(())
    }

    // EXAMPLE VALUES
    fn default_standing_positions(&self) -> Vec<(usize, f32)> {
        vec![
            // Left leg
            (1, 0.0), // hip_roll
            (2, 0.0), // hip_yaw
            (3, 0.0), // hip_pitch
            (4, 0.0), // knee_pitch
            (5, 0.0), // ankle_pitch
            // Right leg
            (6, 0.0), // hip_roll
            (7, 0.0), // hip_yaw
            (8, 0.0), // hip_pitch
            (9, 0.0), // knee_pitch
            (10, 0.0), // ankle_pitch

                      // Left arm (if needed)
                      // (11, 0.0), // shoulder_yaw
                      // (12, 0.0), // shoulder_pitch
                      // (13, 0.0), // elbow_pitch
                      // Right arm (if needed)
                      // (14, 0.0), // shoulder_yaw
                      // (15, 0.0), // shoulder_pitch
                      // (16, 0.0), // elbow_pitch
        ]
    }

    pub async fn run(&mut self) -> Result<()> {
        // get joint states
        // get imu state
        // send command

        println!("Starting StandingControllerPID");
        loop {
            self.get_feedback().await?;
            self.send_command().await?;
            sleep(Duration::from_millis(100)).await;
        }
    }
}

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
