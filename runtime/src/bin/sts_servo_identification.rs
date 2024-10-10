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
    const SERVO_ID: u8 = 10;
    const MOVE_TIME: u16 = 20; // 20ms for 50Hz

    servo.enable_readout()?;

    // Initialize ServoMultipleWriteCommand
    let mut cmd = ServoMultipleWriteCommand {
        only_write_positions: 0,
        ids: [0; MAX_SERVOS],
        positions: [0; MAX_SERVOS],
        times: [0; MAX_SERVOS],
        speeds: [0; MAX_SERVOS],
    };

    // Set starting positions for all servos
    for i in 0..MAX_SERVOS {
        cmd.ids[i] = i as u8 + 1;
        cmd.times[i] = 0;
        cmd.speeds[i] = 0;
        
        match i % 3 {
            0 => cmd.positions[i] = 1024, // First servo in each group
            1 => cmd.positions[i] = 2048, // Second servo in each group
            2 => cmd.positions[i] = 2048, // Third servo in each group
            _ => unreachable!(),
        }
    }

    // Move all servos to starting positions
    for _ in 0..3 {
        servo.write_multiple(&cmd)?;
    }

    // Wait for 1 second
    thread::sleep(Duration::from_secs(1));

    // Read the input CSV file
    let file = File::open("input_data.csv")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().skip(1); // Skip header

    let mut output_data = Vec::new();

    let start_time = Instant::now();
    let interval = Duration::from_millis(20); // 50Hz interval

    let mut previous_pos = 1024i16; // Initialize previous position
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
        let speed = 0;

        // Update the command for all servos
        for i in 0..MAX_SERVOS {
            cmd.speeds[i] = speed;
            cmd.times[i] = MOVE_TIME;
            
            match i % 3 {
                0 => cmd.positions[i] = target_pos_real,
                1 => cmd.positions[i] = target_pos_real + 1024,
                2 => cmd.positions[i] = 1024 * 3 - target_pos_real,
                _ => unreachable!(),
            }
        }

        // Measure time to set positions
        let write_start = Instant::now();
        servo.write_multiple(&cmd)?;
        let write_duration = write_start.elapsed();
        println!("Time to set all positions: {:?}", write_duration);

        // Read current position immediately after writing
        let current_data = servo.read_continuous()?;

        // Store data from previous cycle
        if let (Some(prev_target), Some(prev_speed)) = (previous_target, previous_speed) {
            let mut servo_positions = Vec::new();
            for i in 0..MAX_SERVOS {
                servo_positions.push(current_data.servo[i].current_location);
            }
            output_data.push((prev_target, target_pos_sim, pos_sim, servo_positions, prev_speed));
        }

        // Update previous values for next iteration
        previous_target = Some(target_pos_real);
        previous_speed = Some(speed);
        previous_pos = current_data.servo[SERVO_ID as usize - 1].current_location;

        // Sleep for the remaining time to maintain 50Hz
        let elapsed = loop_start.elapsed();
        if elapsed < interval {
            thread::sleep(interval - elapsed);
        }
    }

    // Handle the last data point
    if let (Some(prev_target), Some(prev_speed)) = (previous_target, previous_speed) {
        let final_data = servo.read_continuous()?;
        let mut servo_positions = Vec::new();
        for i in 0..MAX_SERVOS {
            servo_positions.push(final_data.servo[i].current_location);
        }
        output_data.push((prev_target, 0.0, 0.0, servo_positions, prev_speed));
    }

    // Save output to a new CSV file
    let mut output_file = File::create("output_data.csv")?;
    writeln!(output_file, "target_pos_real,target_pos_sim,pos_sim,{},speed", (1..=MAX_SERVOS).map(|i| format!("pos_real_{}", i)).collect::<Vec<_>>().join(","))?;
    for (target_real, target_sim, pos_sim, pos_reals, speed) in output_data {
        writeln!(output_file, "{},{:.6},{:.6},{},{}", target_real, target_sim, pos_sim, pos_reals.iter().map(|&p| p.to_string()).collect::<Vec<_>>().join(","), speed)?;
    }

    println!("Test completed successfully. Data saved to output_data.csv");
    println!("Total runtime: {:?}", start_time.elapsed());
    servo.disable_readout()?;
    Ok(())
}