use tokio;
use std::time::Duration;


// joint_states -> getting joint position feedback (feedback position)
async fn joint_states() {
    let interval = Duration::from_micros(10000); // 100 Hz
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        println!("Running feedback...");
    }
}


// joint_commands -> sending joint position commands (desired position)
async fn joint_commands() {
    let interval = Duration::from_micros(20000); // 50 Hz 
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        println!("Running set...");
    }
}


// control loop
fn control_loop() {

    let joint_states = tokio::spawn(joint_states());
    let joint_commands = tokio::spawn(joint_commands());

    tokio::join!(joint_states, joint_commands);
}


// main control loop
#[tokio::main]
async fn main() {
    println!("Runtime loop experiments");

    control_loop();
}

