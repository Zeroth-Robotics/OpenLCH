use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct PID {
    pub p: f32,
    pub i: f32,
    pub d: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Limits {
    pub lower: f32,
    pub upper: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JointConfig {
    pub id: usize,
    pub pid: PID,
    pub limits: Limits,
}

#[derive(Deserialize, Debug)]
pub struct RobotConfig {
    pub name: String,
    pub height: f32,
    pub weight: i32,
    pub legs: HashMap<String, HashMap<String, JointConfig>>,
    pub arms: HashMap<String, HashMap<String, JointConfig>>,
    pub parameters: ParametersConfig,
    pub default_standing_positions: HashMap<String, f32>,
}

#[derive(Deserialize, Debug)]
pub struct ParametersConfig {
    pub stiffness: HashMap<String, f32>,
    pub damping: HashMap<String, f32>,
    pub effort: HashMap<String, f32>,
    pub velocity: HashMap<String, f32>,
    pub friction: HashMap<String, f32>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub robot: RobotConfig,
}

pub struct Joint {
    pub config: JointConfig,
    pub current_position: f32,
    pub current_velocity: f32,
}

pub struct Robot {
    pub config: RobotConfig,
    pub joints: HashMap<usize, Joint>,
}

impl Robot {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = Self::load_config(config_path)?;
        let joints = Self::create_joints(&config.robot);

        Ok(Robot {
            config: config.robot,
            joints,
        })
    }

    fn load_config<P: AsRef<Path>>(filename: P) -> Result<Config> {
        let contents = fs::read_to_string(&filename)
            .with_context(|| format!("Failed to read config file: {:?}", filename.as_ref()))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", filename.as_ref()))
    }

    fn create_joints(config: &RobotConfig) -> HashMap<usize, Joint> {
        let mut joints = HashMap::new();

        for leg_joints in config.legs.values() {
            for joint_config in leg_joints.values() {
                joints.insert(
                    joint_config.id,
                    Joint {
                        config: joint_config.clone(),
                        current_position: 0.0,
                        current_velocity: 0.0,
                    },
                );
            }
        }

        for arm_joints in config.arms.values() {
            for joint_config in arm_joints.values() {
                joints.insert(
                    joint_config.id,
                    Joint {
                        config: joint_config.clone(),
                        current_position: 0.0,
                        current_velocity: 0.0,
                    },
                );
            }
        }

        joints
    }

    pub fn get_joint_state(&self, joint_id: usize) -> Option<(f32, f32)> {
        self.joints
            .get(&joint_id)
            .map(|joint| (joint.current_position, joint.current_velocity))
    }

    pub fn set_joint_command(&mut self, joint_id: usize, position: f32, velocity: f32) {
        if let Some(joint) = self.joints.get_mut(&joint_id) {
            joint.current_position = position;
            joint.current_velocity = velocity;
        }
    }

    pub fn get_default_standing_positions(&self) -> &HashMap<String, f32> {
        &self.config.default_standing_positions
    }

    pub fn print_config(&self) {
        println!("Robot Configuration:");
        println!("Name: {}", self.config.name);
        println!("Height: {} m", self.config.height);
        println!("Weight: {} kg", self.config.weight);

        for (limb_type, joints) in [("Leg", &self.config.legs), ("Arm", &self.config.arms)] {
            for (side, side_joints) in joints {
                println!("{} {} Configuration:", side, limb_type);
                for (joint_name, joint_config) in side_joints {
                    println!("  Joint: {}", joint_name);
                    println!("    ID: {}", joint_config.id);
                    println!(
                        "    PID: p={}, i={}, d={}",
                        joint_config.pid.p, joint_config.pid.i, joint_config.pid.d
                    );
                    println!(
                        "    Limits: lower={}, upper={}",
                        joint_config.limits.lower, joint_config.limits.upper
                    );
                }
            }
        }

        println!("Parameters:");
        println!("  Stiffness: {:?}", self.config.parameters.stiffness);
        println!("  Damping: {:?}", self.config.parameters.damping);
        println!("  Effort: {:?}", self.config.parameters.effort);
        println!("  Velocity: {:?}", self.config.parameters.velocity);
        println!("  Friction: {:?}", self.config.parameters.friction);

        println!("Default Standing Positions:");
        for (joint, position) in &self.config.default_standing_positions {
            println!("  {}: {}", joint, position);
        }
    }
}
