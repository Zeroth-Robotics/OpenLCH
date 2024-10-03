use anyhow::{Result, bail};
use ctrlc;
use runtime::hal::Servo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::env;

const LOOP_RATE: f64 = 50.0; // Hz

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let servo_id = match args.get(1) {
        Some(arg) => arg.parse().map_err(|_| anyhow::anyhow!("Invalid servo ID"))?,
        None => bail!("Servo ID must be specified as a command-line argument"),
    };

    let servo = Arc::new(Servo::new()?);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nInterrupt signal received. Stopping...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Starting to read servo {} at {} Hz", servo_id, LOOP_RATE);
    println!("Press Ctrl+C to stop");

    let loop_duration = Duration::from_secs_f64(1.0 / LOOP_RATE);

    servo.enable_readout()?;

    while running.load(Ordering::SeqCst) {
        let start = Instant::now();

        match servo.read_info(servo_id) {
            Ok(info) => {
                println!(
                    "Position: {}, Speed: {}, Load: {}, Current: {} mA",
                    info.current_location,
                    info.current_speed,
                    info.current_load,
                    info.current_current as f32 * 6.5 / 100.0
                );
            }
            Err(e) => {
                eprintln!("Error reading servo info: {}", e);
            }
        }

        let elapsed = start.elapsed();
        if elapsed < loop_duration {
            thread::sleep(loop_duration - elapsed);
        } else {
            eprintln!("Warning: Loop took longer than the desired rate");
        }
    }

    println!("Exiting...");
    Ok(())
}
