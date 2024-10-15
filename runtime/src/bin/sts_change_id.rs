use anyhow::{bail, Result};
use runtime::hal::{Servo, ServoRegister};
use std::env;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        bail!("Usage: {} <current_id> <new_id>", args[0]);
    }

    let current_id: u8 = args[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid current ID"))?;
    let new_id: u8 = args[2]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid new ID"))?;

    if new_id == 0 || new_id > 254 {
        bail!("New ID must be between 1 and 254");
    }

    let servo = Arc::new(Servo::new()?);
    println!("Changing servo ID from {} to {}", current_id, new_id);

    change_servo_id(&servo, current_id, new_id)?;

    println!("ID change complete. Verifying...");
    sleep(Duration::from_millis(500)); // Wait for the change to take effect

    if verify_servo_id(&servo, new_id)? {
        println!(
            "Verification successful. Servo ID has been changed to {}.",
            new_id
        );
    } else {
        println!("Verification failed. Please check the servo and try again.");
    }

    Ok(())
}

fn change_servo_id(servo: &Arc<Servo>, current_id: u8, new_id: u8) -> Result<()> {
    // Disable readout
    servo.disable_readout()?;

    // Unlock EEPROM
    servo.write(current_id, ServoRegister::LockMark, &[0])?;
    sleep(Duration::from_millis(10));

    // Write new ID
    servo.write(current_id, ServoRegister::ID, &[new_id])?;
    sleep(Duration::from_millis(10));

    // Lock EEPROM
    servo.write(new_id, ServoRegister::LockMark, &[1])?;
    sleep(Duration::from_millis(10));

    // Enable readout
    servo.enable_readout()?;

    Ok(())
}

fn verify_servo_id(servo: &Arc<Servo>, id: u8) -> Result<bool> {
    match servo.read(id, ServoRegister::ID, 1) {
        Ok(data) if data.len() == 1 && data[0] == id => Ok(true),
        _ => Ok(false),
    }
}
