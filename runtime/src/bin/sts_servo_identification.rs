use anyhow::Result;
use std::env;
use std::thread;
use std::time::Duration;
use runtime::hal::Servo;

fn main() -> Result<()> {
    // Get the number of cycles from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number_of_cycles>", args[0]);
        std::process::exit(1);
    }
    let cycles: u32 = args[1].parse().expect("Invalid number of cycles");

    // Initialize the servo
    let servo = Servo::new()?;

    // Disable readout
    servo.disable_readout()?;

    // Constants
    const SERVO_ID: u8 = 10;
    const START_POS: i16 = 1024;
    const END_POS: i16 = 2048;
    const MOVE_TIME: u16 = 2500; // 5 seconds in milliseconds

    // Calculate speed
    let speed = ((END_POS - START_POS).abs() as u16) / 5;

    for cycle in 1..=cycles {
        println!("Cycle {}/{}", cycle, cycles);

        // Move servo from 1024 to 2048
        println!("Moving servo {} from {} to {}", SERVO_ID, START_POS, END_POS);
        servo.move_servo(SERVO_ID, END_POS, MOVE_TIME, speed)?;
        thread::sleep(Duration::from_millis(MOVE_TIME as u64));

        // Move servo back to 1024
        println!("Moving servo {} from {} to {}", SERVO_ID, END_POS, START_POS);
        servo.move_servo(SERVO_ID, START_POS, MOVE_TIME, speed)?;
        thread::sleep(Duration::from_millis(MOVE_TIME as u64));
    }


    servo.enable_readout()?;
    println!("Test completed successfully.");
    Ok(())
}
