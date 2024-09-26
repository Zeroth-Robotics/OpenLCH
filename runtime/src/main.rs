use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let fbk = tokio::spawn(async_fbk());
    let set = tokio::spawn(async_set());

    tokio::join!(fbk, set);
}

async fn async_fbk() {
    let interval = Duration::from_micros(10000); // 100 Hz
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        println!("Running feedback...");
    }
}

async fn async_set() {
    let interval = Duration::from_micros(20000); // 50 Hz 
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        println!("Running set...");
    }
}
