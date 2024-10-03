use anyhow::{Result, bail};
use std::thread::sleep;
use std::time::Duration;
use runtime::hal::Servo;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc;

const CURRENT_THRESHOLD: f32 = 200.0; // mA
const CALIBRATION_SPEED: u16 = 100;
const MIN_SPEED: u16 = 10;
const SERVO_MAX_VALUE: i16 = 4095;

const SERVO_ADDR_EEPROM_WRITE: u8 = 0x1F;
const SERVO_ADDR_POSITION_CORRECTION: u8 = 0x1A;
const SERVO_ADDR_MIN_ANGLE: u8 = 0x09;
const SERVO_ADDR_MAX_ANGLE: u8 = 0x0B;
const SERVO_ADDR_OPERATION_MODE: u8 = 0x21;
const SERVO_ADDR_TARGET_POSITION: u8 = 0x2A;

pub fn calibrate_servo(servo: &Servo, servo_id: u8, running: &Arc<AtomicBool>) -> Result<()> {
    println!("Starting servo calibration for ID: {}", servo_id);

    servo.disable_readout()?;
    servo.set_mode(servo_id, 1)?; // Set to continuous mode

    let mut max_forward = 0;
    let mut max_backward = 0;

    for pass in 0..2 {
        if !running.load(Ordering::SeqCst) {
            println!("Calibration interrupted. Stopping servo...");
            servo.set_speed(servo_id, 0, 1)?;
            return Ok(());
        }

        let direction = if pass == 0 { 1 } else { -1 };
        println!("Starting calibration pass {}, direction: {}", 
                 pass + 1, if direction == 1 { "forward" } else { "backward" });

        servo.set_speed(servo_id, CALIBRATION_SPEED, direction)?;

        loop {
            let info = servo.read_info(servo_id)?;
            let position = info.current_location;
            let current = info.current_current as f32 * 6.5 / 100.0;

            if current > CURRENT_THRESHOLD {
                println!("Current threshold reached at position {}", position);

                // Stop
                servo.set_speed(servo_id, 0, direction)?;
                sleep(Duration::from_millis(100));

                // Back off
                servo.set_speed(servo_id, CALIBRATION_SPEED, -direction)?;
                sleep(Duration::from_millis(100));

                // Stop after backoff
                servo.set_speed(servo_id, 0, -direction)?;
                sleep(Duration::from_millis(100));

                // Move slowly to find exact position
                servo.set_speed(servo_id, MIN_SPEED, direction)?;
                while current <= CURRENT_THRESHOLD * 2.0 {
                    let info = servo.read_info(servo_id)?;
                    let current = info.current_current as f32 * 6.5 / 100.0;
                    sleep(Duration::from_millis(10));
                }

                // Stop at exact position
                servo.set_speed(servo_id, 0, direction)?;
                sleep(Duration::from_millis(100));

                let info = servo.read_info(servo_id)?;
                println!("Exact threshold position found: {}", info.current_location);

                // Back off again
                servo.set_speed(servo_id, CALIBRATION_SPEED, -direction)?;
                sleep(Duration::from_millis(100));

                // Stop after final backoff
                servo.set_speed(servo_id, 0, -direction)?;
                sleep(Duration::from_millis(100));

                let info = servo.read_info(servo_id)?;
                println!("Calibration complete for this direction. Final position: {}", info.current_location);

                if direction == 1 {
                    max_forward = info.current_location;
                    println!("Forward calibration complete. Max position: {}", max_forward);
                } else {
                    max_backward = info.current_location;
                    println!("Backward calibration complete. Max position: {}", max_backward);
                }

                break;
            }

            sleep(Duration::from_millis(10));
        }

        if pass < 1 {
            println!("Changing direction for next calibration pass...");
            sleep(Duration::from_millis(500));
        }
    }

    servo.set_speed(servo_id, CALIBRATION_SPEED, 1)?;
    sleep(Duration::from_millis(100));
    servo.set_speed(servo_id, 0, 1)?;

    // Calculate offset and limits
    let (offset, min_angle, max_angle) = if max_backward > max_forward {
        (SERVO_MAX_VALUE + 100 - max_backward, 100, max_forward + SERVO_MAX_VALUE - max_backward + 100)
    } else {
        (0, max_backward, max_forward)
    };

    let offset = offset.max(-2047).min(2047);
    let offset_value = if offset < 0 {
        (offset & 0x7FF) as u16 | 0x800
    } else {
        (offset & 0x7FF) as u16
    };

    // Write to EEPROM
    servo.write(servo_id, SERVO_ADDR_EEPROM_WRITE, &[0])?;
    write_servo_memory(servo, servo_id, SERVO_ADDR_POSITION_CORRECTION, offset_value)?;
    write_servo_memory(servo, servo_id, SERVO_ADDR_MIN_ANGLE, min_angle as u16)?;
    write_servo_memory(servo, servo_id, SERVO_ADDR_MAX_ANGLE, max_angle as u16)?;
    servo.write(servo_id, SERVO_ADDR_EEPROM_WRITE, &[1])?;

    println!("Calibration complete.");
    println!("Offset: {}", offset);
    println!("Min Angle: {}", min_angle);
    println!("Max Angle: {}", max_angle);

    sleep(Duration::from_millis(100));

    // Switch back to position control mode
    servo.write(servo_id, SERVO_ADDR_OPERATION_MODE, &[0])?;
    println!("Switched servo back to position control mode.");

    // Calculate and move to middle point
    let middle_point = (min_angle + max_angle) / 2;
    let position_data = [(middle_point & 0xFF) as u8, ((middle_point >> 8) & 0xFF) as u8];
    servo.write(servo_id, SERVO_ADDR_TARGET_POSITION, &position_data)?;
    println!("Moving servo to middle point: {}", middle_point);

    sleep(Duration::from_secs(1));

    println!("Calibration and positioning complete.");

    Ok(())
}

fn write_servo_memory(servo: &Servo, id: u8, address: u8, value: u16) -> Result<()> {
    let data = [(value & 0xFF) as u8, ((value >> 8) & 0xFF) as u8];
    servo.write(id, address, &data)
}

fn main() -> Result<()> {
    let servo = Arc::new(Servo::new()?);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nInterrupt signal received. Stopping calibration...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let result = calibrate_servo(&servo, 1, &running);

    if !running.load(Ordering::SeqCst) {
        println!("Calibration was interrupted. Cleaning up...");
        // Perform any necessary cleanup
        servo.set_speed(1, 0, 1)?; // Stop the servo
        servo.enable_readout()?;
    }

    Ok(())
}