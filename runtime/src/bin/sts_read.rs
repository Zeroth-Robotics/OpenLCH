use anyhow::{bail, Result};
use runtime::hal::{Servo, ServoRegister};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const LOOP_RATE: f64 = 50.0; // Hz

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let servo_id = match args.get(1) {
        Some(arg) => arg
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid servo ID"))?,
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

        match read_servo_info(&servo, servo_id) {
            Ok(info) => {
                println!(
                    "Position: {}, Speed: {}, Load: {}, Current: {} mA",
                    info.position, info.speed, info.load, info.current
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

struct ServoInfo {
    position: u16,
    speed: i16,
    load: i16,
    current: f32,
}

fn read_servo_info(servo: &Servo, id: u8) -> Result<ServoInfo> {
    let position = servo.read(id, ServoRegister::CurrentLocation, 2)?;
    let speed = servo.read(id, ServoRegister::CurrentSpeed, 2)?;
    let load = servo.read(id, ServoRegister::CurrentLoad, 2)?;
    let current = servo.read(id, ServoRegister::CurrentCurrent, 2)?;

    Ok(ServoInfo {
        position: u16::from_le_bytes([position[0], position[1]]),
        speed: i16::from_le_bytes([speed[0], speed[1]]),
        load: i16::from_le_bytes([load[0], load[1]]),
        current: u16::from_le_bytes([current[0], current[1]]) as f32 * 6.5 / 100.0,
    })
}
