use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};
use std::thread;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};

fn clamp(value: u16, min: u16, max: u16) -> u16 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn main() -> Result<()> {
    // Initialize the servo
    let servo = Servo::new()?;

    // Constants
    const SERVO_ID: u8 = 1;
    const MOVE_TIME: u16 = 20; // 20ms for 50Hz

    servo.enable_readout()?;

    // Read the input CSV file
    let file = File::open("input_data.csv")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().skip(1); // Skip header

    let mut output_data = Vec::new();

    let start_time = Instant::now();
    let interval = Duration::from_millis(20); // 50Hz interval

    let mut previous_pos = 1024i16; // Initialize previous position

    // Initialize ServoMultipleWriteCommand
    let mut cmd = ServoMultipleWriteCommand {
        only_write_positions: 0,
        ids: [0; MAX_SERVOS],
        positions: [0; MAX_SERVOS],
        times: [0; MAX_SERVOS],
        speeds: [0; MAX_SERVOS],
    };

    // Read initial servo positions
    let initial_data = servo.read_continuous()?;
    for (i, servo_info) in initial_data.servo.iter().enumerate() {
        cmd.ids[i] = i as u8 + 1;
        cmd.positions[i] = servo_info.current_location;
        cmd.times[i] = MOVE_TIME;
        cmd.speeds[i] = 0; // We'll update this for the test servo
    }

    let mut previous_target: Option<i16> = None;
    let mut previous_speed: Option<u16> = None;

    while let Some(Ok(line)) = lines.next() {
        let loop_start = Instant::now();

        let values: Vec<&str> = line.split(',').collect();
        if values.len() != 3 {
            eprintln!("Invalid line format: {}", line);
            continue;
        }

        let target_pos_real: i16 = values[0].parse()?;
        let target_pos_sim: f32 = values[1].parse()?;
        let pos_sim: f32 = values[2].parse()?;

        // Calculate speed and clamp it
        let speed = clamp(
            ((target_pos_real - previous_pos).abs() as f32 / (MOVE_TIME as f32 / 1000.0)) as u16,
            100,
            4096
        );

        // Update the command for the test servo
        let servo_index = SERVO_ID as usize - 1;
        cmd.positions[servo_index] = target_pos_real;
        cmd.speeds[servo_index + 1] = speed;
        cmd.speeds[servo_index + 2] = speed;

        cmd.positions[servo_index + 1] = target_pos_real + 1024;
        cmd.positions[servo_index + 2] = 1024 * 3 - target_pos_real;

        // Measure time to set positions
        let write_start = Instant::now();
        servo.write_multiple(&cmd)?;
        let write_duration = write_start.elapsed();
        println!("Time to set all positions: {:?}", write_duration);

        // Read current position immediately after writing
        let current_data = servo.read_continuous()?;
        let current_pos = current_data.servo[servo_index].current_location;

        println!("Current position: {}, speed: {}", current_pos, speed);
        
        // Store data from previous cycle
        if let (Some(prev_target), Some(prev_speed)) = (previous_target, previous_speed) {
            output_data.push((prev_target, target_pos_sim, pos_sim, current_pos, prev_speed));
        }

        // Update previous values for next iteration
        previous_target = Some(target_pos_real);
        previous_speed = Some(speed);
        previous_pos = current_pos;

        // Sleep for the remaining time to maintain 50Hz
        let elapsed = loop_start.elapsed();
        if elapsed < interval {
            thread::sleep(interval - elapsed);
        }
    }

    // Handle the last data point
    if let (Some(prev_target), Some(prev_speed)) = (previous_target, previous_speed) {
        let final_data = servo.read_continuous()?;
        let final_pos = final_data.servo[SERVO_ID as usize - 1].current_location;
        output_data.push((prev_target, 0.0, 0.0, final_pos, prev_speed));
    }

    // Save output to a new CSV file
    let mut output_file = File::create("output_data.csv")?;
    writeln!(output_file, "target_pos_real,target_pos_sim,pos_sim,pos_real,speed")?;
    for (target_real, target_sim, pos_sim, pos_real, speed) in output_data {
        writeln!(output_file, "{},{:.6},{:.6},{},{}", target_real, target_sim, pos_sim, pos_real, speed)?;
    }

    println!("Test completed successfully. Data saved to output_data.csv");
    println!("Total runtime: {:?}", start_time.elapsed());
    servo.disable_readout()?;
    Ok(())
}
