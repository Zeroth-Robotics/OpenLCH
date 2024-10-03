use anyhow::Result;
use std::sync::Arc;
use runtime::hal::Servo;

fn main() -> Result<()> {
    let servo = Arc::new(Servo::new()?);
    println!("Scanning for servos...");
    servo.disable_readout()?;

    for id in 1..=100 {
        match scan_servo(&servo, id) {
            Ok(true) => println!("Servo found at ID: {}", id),
            Ok(false) => (), // No servo at this ID, continue silently
            Err(e) => eprintln!("Error scanning ID {}: {}", id, e),
        }
    }

    servo.enable_readout()?;
    println!("Scan complete.");
    Ok(())
}

fn scan_servo(servo: &Arc<Servo>, id: u8) -> Result<bool> {
    // Try to read the servo ID from memory address 0x5
    match servo.read(id, 0x5, 1) {
        Ok(data) if data.len() == 1 && data[0] == id => Ok(true),
        Ok(_) => Ok(false), // Received data, but it doesn't match the ID
        Err(_) => Ok(false), // No response, assume no servo at this ID
    }
}
