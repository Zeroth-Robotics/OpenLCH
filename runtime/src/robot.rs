use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct PID {
    p: f32,
    i: f32,
    d: f32,
}

#[derive(Deserialize, Clone)]
pub struct Limits {
    lower: f32,
    upper: f32,
}

#[derive(Deserialize, Clone)]
pub struct ServoConfig {
    id: usize,
    pid: PID,
    limits: Limits,
}

#[derive(Deserialize)]
pub struct Config {
    robot: RobotConfig,
}

#[derive(Deserialize)]
pub struct RobotConfig {
    name: String,
    height: f32,
    weight: i32,
    legs: LegsConfig,
    arms: ArmsConfig,
    parameters: ParametersConfig,
}

#[derive(Deserialize)]
pub struct LegsConfig {
    left: HashMap<String, ServoConfig>,
    right: HashMap<String, ServoConfig>,
}

#[derive(Deserialize)]
pub struct ArmsConfig {
    left: HashMap<String, ServoConfig>,
    right: HashMap<String, ServoConfig>,
}

#[derive(Deserialize)]
pub struct ParametersConfig {
    stiffness: HashMap<String, f32>,
    damping: HashMap<String, f32>,
    effort: HashMap<String, f32>,
    velocity: HashMap<String, f32>,
    friction: HashMap<String, f32>,
}

pub struct Servo {
    config: ServoConfig,
    current_position: f32,
    current_velocity: f32,
}

pub struct Robot {
    config: RobotConfig,
    legs: HashMap<String, HashMap<String, Servo>>,
    arms: HashMap<String, HashMap<String, Servo>>,
}

impl Robot {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = Self::load_config(config_path)?;

        let create_servos =
            |servo_configs: &HashMap<String, ServoConfig>| -> HashMap<String, Servo> {
                servo_configs
                    .iter()
                    .map(|(name, config)| {
                        (
                            name.clone(),
                            Servo {
                                config: config.clone(),
                                current_position: 0.0,
                                current_velocity: 0.0,
                            },
                        )
                    })
                    .collect()
            };

        let legs = HashMap::from([
            ("left".to_string(), create_servos(&config.robot.legs.left)),
            ("right".to_string(), create_servos(&config.robot.legs.right)),
        ]);

        let arms = HashMap::from([
            ("left".to_string(), create_servos(&config.robot.arms.left)),
            ("right".to_string(), create_servos(&config.robot.arms.right)),
        ]);

        Ok(Robot {
            config: config.robot,
            legs,
            arms,
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
        self.legs
            .values()
            .chain(self.arms.values())
            .flat_map(|limb| limb.values())
            .collect()
    }

    fn all_servos_mut(&mut self) -> Vec<&mut Servo> {
        self.legs
            .values_mut()
            .chain(self.arms.values_mut())
            .flat_map(|limb| limb.values_mut())
            .collect()
    }

    pub fn print_config(&self) {
        println!("Robot Configuration:");
        println!("Name: {}", self.config.name);
        println!("Height: {} m", self.config.height);
        println!("Weight: {} kg", self.config.weight);

        let print_servos = |servos: &HashMap<String, Servo>, limb_type: &str, side: &str| {
            println!("{} {} Configuration:", side, limb_type);
            let mut sorted_servos: Vec<_> = servos.iter().collect();
            sorted_servos.sort_by_key(|(_, servo)| servo.config.id);
            for (name, servo) in sorted_servos {
                println!("  Servo: {}", name);
                println!("    ID: {}", servo.config.id);
                println!("    Current Position: {}", servo.current_position);
                println!("    Current Velocity: {}", servo.current_velocity);
                println!(
                    "    PID: p={}, i={}, d={}",
                    servo.config.pid.p, servo.config.pid.i, servo.config.pid.d
                );
                println!(
                    "    Limits: lower={}, upper={}",
                    servo.config.limits.lower, servo.config.limits.upper
                );
            }
        };

        for (side, servos) in &self.legs {
            print_servos(servos, "Leg", side);
        }

        for (side, servos) in &self.arms {
            print_servos(servos, "Arm", side);
        }

        println!("Parameters:");
        println!("  Stiffness: {:?}", self.config.parameters.stiffness);
        println!("  Damping: {:?}", self.config.parameters.damping);
        println!("  Effort: {:?}", self.config.parameters.effort);
        println!("  Velocity: {:?}", self.config.parameters.velocity);
        println!("  Friction: {:?}", self.config.parameters.friction);
    }
}
