use anyhow::Result;
use runtime::hal::{Servo, ServoMultipleWriteCommand, MAX_SERVOS};
use tokio::time::{sleep, interval, Duration};

async fn joint_states(servo: &Servo) -> Result<()> {
    let mut interval = interval(Duration::from_millis(10)); // 100Hz
    loop {
        interval.tick().await;
        let servo_data = servo.read_continuous()?;
        for (i, servo_info) in servo_data.servo.iter().enumerate() {
            // println!("Servo {}: Feedback Joint Position = {}", i + 1, servo_info.target_location);
        }
    }
}

async fn joint_commands(servo: &Servo, position: i16, time: u16, speed: u16, send_only_positions: u8) -> Result<()> {
    let mut interval = interval(Duration::from_millis(20)); // 50Hz
    loop {
        interval.tick().await;
        let mut cmd = ServoMultipleWriteCommand {
            ids: [0; MAX_SERVOS],
            positions: [0; MAX_SERVOS],
            times: [0; MAX_SERVOS],
            speeds: [0; MAX_SERVOS],
            only_write_positions: send_only_positions,
        };

        for i in 0..MAX_SERVOS {
            cmd.ids[i] = (i + 1) as u8;
            cmd.positions[i] = position;
            cmd.times[i] = time;
            cmd.speeds[i] = speed;
        }

        servo.write_multiple(&cmd)?;

        println!("Command sent to move all servos to position {} with time {} ms and speed {}, send_only_positions: {}", position, time, speed, send_only_positions);
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let servo = Servo::new()?;
    servo.enable_readout()?;

    let position: i16 = 4000; // position: target position (0-4095)
    let time: u16 = 100; // time: movement time in milliseconds
    let speed: u16 = 512; // speed: movement speed (0-4095)
    let send_only_positions: u8 = 1; // send_only_positions: send only positions (0 or 1)

    tokio::join!(
        joint_states(&servo),
        joint_commands(&servo, position, time, speed, send_only_positions)
    );

    Ok(())
}
