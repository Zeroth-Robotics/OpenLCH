use serde::Deserialize; // deserialize from toml
use std::fs;
use toml;

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
    left: Vec<MotorConfig>,
    right: Vec<MotorConfig>,
}

#[derive(Deserialize)]
struct ArmsConfig {
    left: Vec<MotorConfig>,
    right: Vec<MotorConfig>,
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
    pub fn new() -> Self {
        let config = Self::load_config("config.toml");

        let left_leg = Leg {
            servos: config
                .robot
                .legs
                .left
                .iter()
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
                .iter()
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
                .iter()
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
                .iter()
                .map(|m| Servo {
                    id: m.id,
                    pid: m.pid.clone(),
                    limits: m.limits.clone(),
                    current_position: 0.0,
                    current_velocity: 0.0,
                })
                .collect(),
        };

        Robot {
            left_leg,
            right_leg,
            left_arm,
            right_arm,
        }
    }

    fn load_config(filename: &str) -> Config {
        let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
        toml::from_str(&contents).expect("Failed to parse the config file")
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
