use tokio;
use std::time::Duration;
use rand::Rng;

// tests
fn generate_random_joint_positions() -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..16).map(|_| rng.gen_range(0.0..std::f32::consts::TAU)).collect()
}

async fn stream_test_joint_positions() -> Vec<f32> {
    let interval = Duration::from_millis(10); // 100 Hz
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        let positions = generate_random_joint_positions();
        println!("Joint positions: {:?}", positions);

    }
}


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
async fn controller() {

    // stream out fake test data
    let test = tokio::spawn(stream_test_joint_positions());


    // joint states
    let joint_states = tokio::spawn(joint_states());
    let joint_commands = tokio::spawn(joint_commands());

    tokio::join!(test, joint_states, joint_commands);
}

// main control loop
#[tokio::main]
async fn main() {
    println!("Runtime loop experiments");

    controller().await;
}

