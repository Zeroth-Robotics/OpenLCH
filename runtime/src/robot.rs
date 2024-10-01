use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
struct PID {
    p: f32,
    i: f32,
    d: f32,
}

#[derive(Deserialize, Clone)]
struct Limits {
    lower: f32,
    upper: f32,
}

#[derive(Deserialize, Clone)]
struct MotorConfig {
    id: usize,
    pid: PID,
    limits: Limits,
}

#[derive(Deserialize)]
struct Config {
    robot: RobotConfig,
}

#[derive(Deserialize)]
struct RobotConfig {
    legs: LegsConfig,
    arms: ArmsConfig,
}

#[derive(Deserialize)]
struct LegsConfig {
    left: HashMap<String, MotorConfig>,
    right: HashMap<String, MotorConfig>,
}

#[derive(Deserialize)]
struct ArmsConfig {
    left: HashMap<String, MotorConfig>,
    right: HashMap<String, MotorConfig>,
}

pub struct Servo {
    id: usize,
    pid: PID,
    limits: Limits,
    current_position: f32,
    current_velocity: f32,
}

pub struct Leg {
    servos: Vec<Servo>,
}

pub struct Arm {
    servos: Vec<Servo>,
}

pub struct Robot {
    left_leg: Leg,
    right_leg: Leg,
    left_arm: Arm,
    right_arm: Arm,
}

impl Robot {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = Self::load_config(config_path)?;

        let left_leg = Leg {
            servos: config
                .robot
                .legs
                .left
                .values()
                .map(|m| Servo {
                    id: m.id,
                    pid: m.pid.clone(),
                    limits: m.limits.clone(),
                    current_position: 0.0,
                    current_velocity: 0.0,
                })
                .collect(),
        };

        let right_leg = Leg {
            servos: config
                .robot
                .legs
                .right
                .values()
                .map(|m| Servo {
                    id: m.id,
                    pid: m.pid.clone(),
                    limits: m.limits.clone(),
                    current_position: 0.0,
                    current_velocity: 0.0,
                })
                .collect(),
        };

        let left_arm = Arm {
            servos: config
                .robot
                .arms
                .left
                .values()
                .map(|m| Servo {
                    id: m.id,
                    pid: m.pid.clone(),
                    limits: m.limits.clone(),
                    current_position: 0.0,
                    current_velocity: 0.0,
                })
                .collect(),
        };

        let right_arm = Arm {
            servos: config
                .robot
                .arms
                .right
                .values()
                .map(|m| Servo {
                    id: m.id,
                    pid: m.pid.clone(),
                    limits: m.limits.clone(),
                    current_position: 0.0,
                    current_velocity: 0.0,
                })
                .collect(),
        };

        Ok(Robot {
            left_leg,
            right_leg,
            left_arm,
            right_arm,
        })
    }

    fn load_config<P: AsRef<Path>>(filename: P) -> Result<Config> {
        let contents = fs::read_to_string(&filename)
            .with_context(|| format!("Failed to read config file: {:?}", filename.as_ref()))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", filename.as_ref()))
    }

    // ### === TODO: DENYS === ###
    pub fn get_joint_state(&self, joint_id: usize) -> Option<(f32, f32)> {
        // ### === TODO: DENYS === ###
        // for servo in self.all_servos() {
        //     if servo.id == joint_id {
        //         return Some((servo.current_position, servo.current_velocity));
        //     }
        // }
        None
    }

    // ### === TODO: DENYS === ###
    pub fn set_joint_command(&mut self, joint_id: usize, position: f32, velocity: f32) {
        // for servo in self.all_servos_mut() {
        //     if servo.id == joint_id {
        //         servo.current_position = position;
        //         servo.current_velocity = velocity;
        //     }
        // }
    }

    fn all_servos(&self) -> Vec<&Servo> {
        self.left_leg
            .servos
            .iter()
            .chain(self.right_leg.servos.iter())
            .chain(self.left_arm.servos.iter())
            .chain(self.right_arm.servos.iter())
            .collect()
    }

    fn all_servos_mut(&mut self) -> Vec<&mut Servo> {
        self.left_leg
            .servos
            .iter_mut()
            .chain(self.right_leg.servos.iter_mut())
            .chain(self.left_arm.servos.iter_mut())
            .chain(self.right_arm.servos.iter_mut())
            .collect()
    }
}
