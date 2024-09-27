
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


// load PPO model
// get joint positions from sim
// run inference
// send commands to sim
// repeat

fn main() {
    println!("sim2sim");
}




