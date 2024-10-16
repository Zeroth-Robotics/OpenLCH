use anyhow::{Context, Result};
use clap::Parser;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(short, long, default_value_t = false)]
    looped: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct CaptureData {
    name: String,
    cap: Vec<CapFrame>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CapFrame {
    pos: std::collections::HashMap<String, i32>,
    delay: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let servo = Servo::new()?;
    let capture_data = load_capture_file(&args.file)?;

    println!("Replaying capture: {}", capture_data.name);

    let mut current_positions = vec![0; MAX_SERVOS];
    let servo_data = servo.read_continuous()?;
    servo_data.servo.iter().enumerate().for_each(|(i, s)| {
        current_positions[i] = s.current_location as i32;
    });

    loop {
        for frame in &capture_data.cap {
            let start_time = Instant::now();
            let steps = (frame.delay as f64 / 20.0).ceil() as u64;
            let target_positions = frame_to_positions(frame);

            for step in 0..steps {
                let progress = step as f64 / steps as f64;
                let interpolated_positions: Vec<i32> = current_positions
                    .iter()
                    .zip(target_positions.iter())
                    .map(|(&current, &target)| {
                        (current as f64 + (target as f64 - current as f64) * progress) as i32
                    })
                    .collect();

                let mut cmd = ServoMultipleWriteCommand {
                    only_write_positions: 1,
                    ids: [0; MAX_SERVOS],
                    positions: [0; MAX_SERVOS],
                    times: [0; MAX_SERVOS],
                    speeds: [0; MAX_SERVOS],
                };

                for (servo_id, &pos) in interpolated_positions.iter().enumerate() {
                    cmd.ids[servo_id] = servo_id as u8 + 1;
                    cmd.positions[servo_id] = pos as i16;
                }

                servo.write_multiple(&cmd)?;

                std::thread::sleep(Duration::from_millis(20));
            }

            current_positions = target_positions;
            let elapsed = start_time.elapsed();
            if elapsed < Duration::from_millis(frame.delay) {
                std::thread::sleep(Duration::from_millis(frame.delay) - elapsed);
            }
        }

        if !args.looped {
            break;
        }
    }

    Ok(())
}

fn load_capture_file(path: &PathBuf) -> Result<CaptureData> {
    let mut file = File::open(path).context("Failed to open capture file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read capture file")?;

    let capture_data: CaptureData =
        serde_json::from_str(&contents).context("Failed to parse JSON")?;
    Ok(capture_data)
}

fn frame_to_positions(frame: &CapFrame) -> Vec<i32> {
    let mut positions = vec![0; MAX_SERVOS];
    for (key, &value) in &frame.pos {
        if let Ok(index) = key.parse::<usize>() {
            if index > 0 && index <= MAX_SERVOS {
                positions[index - 1] = value;
            }
        }
    }
    positions
}
