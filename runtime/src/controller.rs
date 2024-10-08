use crate::model::Model;
use crate::robot::Robot;
use crate::hal::Servo;
use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;
use std::sync::Arc;
use std::time::Instant;

#[async_trait]
pub trait Controller: Send + Sync {
    async fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>>;
}

pub struct PIDController {
    // PID parameters
}

#[async_trait]
impl Controller for PIDController {
    async fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>> {
        // PID control logic
        println!("Computing action with PID controller");

        // Define a small factor for tiny changes (e.g., 1%)
        let small_change_factor = 0.01;

        // Create a random vector with tiny changes relative to the state
        let mut rng = rand::thread_rng();
        let random_vec: Vec<f32> = state
            .iter()
            .map(|&value| {
                let random_change = rng.gen_range(-small_change_factor..small_change_factor);
                value + random_change
            })
            .collect();

        Ok(random_vec)
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

#[async_trait]
impl Controller for MLController {
    async fn compute_action(&self, state: &[f32]) -> Result<Vec<f32>> {
        // ML control logic
        println!("Computing action with ML controller");
        self.model.infer(state)
    }
}

pub struct StandingController {
    robot: Robot,
    controller: Arc<dyn Controller>,
    servo: Servo, // Add Servo to the struct
}

impl StandingController {
    pub fn new(robot: Robot, controller: Arc<dyn Controller>, servo: Servo) -> Self {
        Self { robot, controller, servo }
    }

    pub async fn get_state(&self) -> Result<Vec<f32>> {
        // Fetch servo data from hal.rs
        let servo_data = self.servo.read_continuous()?;
        // Convert servo positions to f32 and collect into a vector
        let positions: Vec<f32> = servo_data
            .servo
            .iter()
            .map(|s| s.current_location as f32)
            .collect();
        Ok(positions)
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

    pub async fn run(&mut self, iterations: Option<u32>) -> Result<()> {
        println!("Starting StandingController");

        let mut i = 0;
        loop {
            // Check if a limit is set and if the iteration count has reached it
            if let Some(max_iterations) = iterations {
                if i >= max_iterations {
                    break;
                }
            }

            println!("Controller iteration {}", i);
            let state = self.get_state().await?;
            let command = self.controller.compute_action(&state).await?;
            self.send_command(&command).await?;

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            i += 1;
        }

        println!("StandingController finished");
        Ok(())
    }
}
