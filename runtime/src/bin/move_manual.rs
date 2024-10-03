use anyhow::{bail, Result};
use runtime::hal::Servo;
use std::env;
use std::thread;
use std::time::Duration;

const MOVE_TIME: u16 = 1000; // Time to reach position in milliseconds
const MOVE_SPEED: u16 = 0; // 0 means maximum speed

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        bail!("Usage: {} <servo_id> <target_position>", args[0]);
    }

    let servo_id: u8 = args[1].parse()?;
    let target_position: i16 = args[2].parse()?;

    println!("Initializing servo...");
    let servo = Servo::new()?;

    println!(
        "Moving servo {} to position {}...",
        servo_id, target_position
    );
    servo.move_servo(servo_id, target_position, MOVE_TIME, MOVE_SPEED)?;

    // Wait for the move to complete
    thread::sleep(Duration::from_millis(MOVE_TIME as u64));

    // Read and print the final position
    let info = servo.read_info(servo_id)?;
    println!("Move complete. Final position: {}", info.current_location);

    Ok(())
}
