use anyhow::Result;
use tokio::time::{interval, Duration};
use std::sync::{Arc, Mutex};
use crate::model::Model;
use crate::robot::Robot;

// pub async fn run(model: Arc<Model>, robot: Arc<Mutex<Robot>>) -> Result<()> {
//     let mut control_interval = interval(Duration::from_millis(20)); // 50HZ, for the sake of example

//     loop {
//         control_interval.tick().await;

//         let current_joint_states = robot.lock().unwrap().get_joint_states().await?;
//         let desired_joint_positions = robot.lock().unwrap().model_inference(&model).await?;
//         robot.lock().unwrap().send_joint_commands(&desired_joint_positions).await?;
//     }
// }
