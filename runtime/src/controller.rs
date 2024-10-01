use crate::model::Model;
use crate::robot::Robot;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

pub trait Controller {
    fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>>;
}

pub struct PIDController {
    // PID parameters
}

impl Controller for PIDController {
    fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>> {
        // PID control logic
        println!("Computing action with PID controller");
        Ok(vec![0.0; state.len()])
    }
}

pub struct MLController {
    model: Model,
}

impl MLController {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
}

impl Controller for MLController {
    fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>> {
        // ML control logic
        println!("Computing action with ML controller");
        self.model.infer(state)
    }
}

pub struct StandingController {
    robot: Robot,
    controller: Arc<dyn Controller + Send + Sync>,
}

impl StandingController {
    pub fn new(robot: Robot, controller: Arc<dyn Controller + Send + Sync>) -> Self {
        Self { robot, controller }
    }

    pub async fn get_state(&self) -> Result<Vec<f32>> {
        // ### === TODO: DENYS === ###
        // let state = self.robot.joint_states().await;
        // let imu = self.robot.imu_state().await;
        // let action = self.model.infer(state, imu).await?;
        Ok(vec![0.0; 10])
    }

    pub async fn send_command(&self, command: &[f32]) -> Result<()> {
        // ### === TODO: DENYS === ###
        let time = Instant::now();
        println!("Sending command: {:?}", command);
        println!("Start time: {:?}", time);
        Ok(())
    }

    pub async fn stand(&mut self) -> Result<()> {
        let desired_positions = self.robot.get_default_standing_positions();
        let mut commands = Vec::new();

        for (joint_name, &position) in desired_positions {
            if let Some(joint_config) = self.robot.find_joint_config(joint_name) {
                commands.push((joint_config.id, position));
            }
        }

        for (joint_id, position) in commands {
            self.robot.set_joint_command(joint_id, position, 0.0);
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Starting StandingController");
        for i in 0..5 {
            // Run for 5 iterations only
            println!("Controller iteration {}", i);
            let state = self.get_state().await?;
            let command = self.controller.compute_action(&state)?;
            self.send_command(&command).await?;

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!("StandingController finished");
        Ok(())
    }
}
