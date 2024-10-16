use runtime::hal::IMU;
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let mut imu = IMU::new()?;
    let target_duration = Duration::from_millis(20); // 50Hz = 20ms period

    println!("Starting IMU readings at 50Hz. Press Ctrl+C to stop.");
    println!("Timestamp,AccX,AccY,AccZ,GyroX,GyroY,GyroZ");

    loop {
        let start = Instant::now();

        // Read IMU data
        match imu.read_data() {
            Ok(data) => {
                let now = Instant::now();
                println!(
                    "{:.3},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
                    now.duration_since(start).as_secs_f64(),
                    data.acc_x,
                    data.acc_y,
                    data.acc_z,
                    data.gyro_x,
                    data.gyro_y,
                    data.gyro_z
                );
            }
            Err(e) => eprintln!("Error reading IMU data: {}", e),
        }

        // Calculate sleep duration to maintain 50Hz
        let elapsed = start.elapsed();
        if elapsed < target_duration {
            thread::sleep(target_duration - elapsed);
        } else {
            eprintln!("Warning: Loop took longer than 20ms");
        }
    }
}
