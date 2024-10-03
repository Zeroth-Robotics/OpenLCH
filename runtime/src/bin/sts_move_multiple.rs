use anyhow::Result;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};
use std::env;

fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        println!("Usage: {} <position> <time> <speed> <send_only_positions>", args[0]);
        println!("  position: target position (0-4095)");
        println!("  time: movement time in milliseconds");
        println!("  speed: movement speed (0-4095)");
        println!("  send_only_positions: send only positions (0 or 1)");
        std::process::exit(1);
    }

    let position: i16 = args[1].parse()?;
    let time: u16 = args[2].parse()?;
    let speed: u16 = args[3].parse()?;
    let send_only_positions: u8 = args[4].parse()?;
    // Initialize the servo
    let servo = Servo::new()?;

    // Enable servo readout
    servo.enable_readout()?;

    // Prepare the command for all servos
    let mut cmd = ServoMultipleWriteCommand {
        ids: [0; MAX_SERVOS],
        positions: [0; MAX_SERVOS],
        times: [0; MAX_SERVOS],
        speeds: [0; MAX_SERVOS],
        only_write_positions: send_only_positions,
    };

    // Fill the command structure for all servos
    for i in 0..MAX_SERVOS {
        cmd.ids[i] = (i + 1) as u8;
        cmd.positions[i] = position;
        cmd.times[i] = time;
        cmd.speeds[i] = speed;
    }

    // Send the command to move all servos
    servo.write_multiple(&cmd)?;

    println!("Command sent to move all servos to position {} with time {} ms and speed {}, send_only_positions: {}", position, time, speed, send_only_positions);

    // Wait for the movement to complete
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Read and print the current positions of all servos
    let servo_data = servo.read_continuous()?;
    for (i, servo_info) in servo_data.servo.iter().enumerate() {
        println!("Servo {}: Current position = {}", i + 1, servo_info.current_location);
    }

    Ok(())
}
