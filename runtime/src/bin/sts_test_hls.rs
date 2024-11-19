use anyhow::Result;
use std::sync::Arc;
use runtime::hal::{Servo, ServoRegister};
use anyhow::bail;
use std::env;

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        bail!("Usage: {} <servo_id> <target_position>", args[0]);
    }

    let servo_id: u8 = args[1].parse()?;
    let target_position: i16 = args[2].parse()?;

    let servo = Arc::new(Servo::new()?);
    println!("Scanning for servos...");
    servo.disable_readout()?;

    servo.write(servo_id, ServoRegister::Km, &[1])?;
    println!("Set Km to 1");
    servo.write(servo_id, ServoRegister::OperationMode, &[4])?;
    println!("Set Operation Mode to 4");
    servo.move_servo(servo_id, target_position, 1000, 2047)?;

    servo.enable_readout()?;
    println!("Scan complete.");
    Ok(())
}
