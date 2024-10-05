use anyhow::Result;
use runtime::hal::{Servo, MAX_SERVOS, TorqueMode, ServoRegister};
use cursive::views::{TextView, LinearLayout, DummyView, Panel, Dialog, EditView, SelectView, NamedView};
use cursive::traits::*;
use std::cmp::min;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicI16, Ordering};

static CALIBRATION_POSITION: AtomicI16 = AtomicI16::new(-1);
static CURRENT_POSITION: AtomicI16 = AtomicI16::new(0);

fn main() -> Result<()> {
    let servo = Arc::new(Servo::new()?);

    // Enable continuous readout
    servo.enable_readout()?;

    let mut siv = cursive::default();

    // Create a layout for our servo data
    let mut layout = LinearLayout::vertical();

    // Add header
    let header = LinearLayout::horizontal()
        .child(TextView::new("ID").center().fixed_width(4))
        .child(TextView::new("Pos").center().fixed_width(8))
        // .child(TextView::new("Targ Pos").center().fixed_width(8))
        .child(TextView::new("Spd").center().fixed_width(8))
        // .child(TextView::new("Run Spd").center().fixed_width(8))
        .child(TextView::new("Load").center().fixed_width(8))
        .child(TextView::new("Torque").center().fixed_width(8))
        // .child(TextView::new("Accel").center().fixed_width(8))
        .child(TextView::new("Volt").center().fixed_width(6))
        .child(TextView::new("Temp").center().fixed_width(6))
        .child(TextView::new("Curr").center().fixed_width(6))
        .child(TextView::new("Status").center().fixed_width(8))
        .child(TextView::new("Torq Lim").center().fixed_width(8));
        // .child(TextView::new("Async").center().fixed_width(6))
        // .child(TextView::new("Lock").center().fixed_width(6));
    layout.add_child(header);

    // Add rows for each servo
    for i in 0..MAX_SERVOS {
        let row = LinearLayout::horizontal()
            .child(TextView::new(format!("{:2}", i + 1)).center().with_name(format!("ID {}", i)).fixed_width(4))
            .child(TextView::new("----").center().with_name(format!("CurrPos {}", i)).fixed_width(8))
            // .child(TextView::new("----").center().with_name(format!("TargPos {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("CurrSpd {}", i)).fixed_width(8))
            // .child(TextView::new("----").center().with_name(format!("RunSpd {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Load {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Torque {}", i)).fixed_width(8))
            // .child(TextView::new("----").center().with_name(format!("Accel {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Volt {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Temp {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Curr {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Status {}", i)).fixed_width(8))
        .child(TextView::new("----").center().with_name(format!("TorqLim {}", i)).fixed_width(8));

            // .child(TextView::new("----").center().with_name(format!("Async {}", i)).fixed_width(6))
            // .child(TextView::new("----").center().with_name(format!("Lock {}", i)).fixed_width(6));
        layout.add_child(row.with_name(format!("servo_row_{}", i)));
        siv.call_on_name("ID 0", |view: &mut TextView| {
            view.set_content(">ID");
        });
    }

    // Add a dummy view to push the task count to the bottom
    layout.add_child(DummyView.full_height());

    // Add task run count at the bottom
    layout.add_child(
        Panel::new(
            LinearLayout::vertical()
                .child(TextView::new("Task Run Count: 0").with_name("Task Count"))
                .child(TextView::new("Last Update: N/A").with_name("Last Update"))
                .child(TextView::new("Servo polling rate: N/A").with_name("Servo polling rate"))
        )
        .title("Statistics")
        .full_width()
    );

    // Add instructions
    layout.add_child(
        TextView::new("Use Up/Down to select servo, Enter - servo settings, T - toggle torque, Q - quit")
            .center()
            .full_width()
    );

    layout.add_child(TextView::new("Min Angle: ----").with_name("MinAngle"));
    layout.add_child(TextView::new("Max Angle: ----").with_name("MaxAngle"));
    layout.add_child(TextView::new("Offset: ----").with_name("Offset"));
    layout.add_child(TextView::new("Calibration Pos: ----").with_name("CalibrationPos"));

    siv.add_fullscreen_layer(layout);

    // Set up a timer to update the UI
    siv.set_fps(30);

    // Clone Arc for the callback
    let servo_clone = Arc::clone(&servo);

    // Add a variable to keep track of the selected servo
    let selected_servo = Arc::new(Mutex::new(0));

    // Add variables for last update time and task count
    let last_update_time = Arc::new(Mutex::new(Instant::now()));

    siv.add_global_callback('q', |s| s.quit());

    // Modify Up and Down callbacks to wrap around
    let servo_clone_up = Arc::clone(&servo);
    let selected_servo_up = Arc::clone(&selected_servo);
    siv.add_global_callback(cursive::event::Event::Key(cursive::event::Key::Up), move |s| {
        let mut selected = selected_servo_up.lock().unwrap();
        *selected = (*selected + MAX_SERVOS - 1) % MAX_SERVOS;
        update_selected_row(s, *selected);
        update_angle_limits(s, *selected as u8 + 1, &servo_clone_up);
    });

    let servo_clone_down = Arc::clone(&servo);
    let selected_servo_down = Arc::clone(&selected_servo);
    siv.add_global_callback(cursive::event::Event::Key(cursive::event::Key::Down), move |s| {
        let mut selected = selected_servo_down.lock().unwrap();
        *selected = (*selected + 1) % MAX_SERVOS;
        update_selected_row(s, *selected);
        update_angle_limits(s, *selected as u8 + 1, &servo_clone_down);
    });

    let servo_clone_enter = Arc::clone(&servo);
    let selected_servo_enter = Arc::clone(&selected_servo);
    siv.add_global_callback(cursive::event::Event::Key(cursive::event::Key::Enter), move |s| {
        // Check if a settings dialog is already open
        if s.find_name::<Dialog>("servo_settings").is_some() {
            return; // Do nothing if a dialog is already open
        }

        let selected = *selected_servo_enter.lock().unwrap();
        let servo_id = selected as u8 + 1;
        open_servo_settings(s, servo_id, Arc::clone(&servo_clone_enter));
    });

    let servo_clone_toggle = Arc::clone(&servo);
    let selected_servo_toggle = Arc::clone(&selected_servo);
    siv.add_global_callback('t', move |s| {
        let selected = *selected_servo_toggle.lock().unwrap();
        let servo_id = selected as u8 + 1;
        toggle_servo_torque(s, servo_id, Arc::clone(&servo_clone_toggle));
    });

    let servo_clone_calibrate_start = Arc::clone(&servo);
    let selected_servo_calibrate_start = Arc::clone(&selected_servo);
    siv.add_global_callback('[', move |s| {
        let selected = *selected_servo_calibrate_start.lock().unwrap();
        let servo_id = selected as u8 + 1;
        start_calibration(s, servo_id, Arc::clone(&servo_clone_calibrate_start));
    });

    let servo_clone_calibrate_end = Arc::clone(&servo);
    let selected_servo_calibrate_end = Arc::clone(&selected_servo);
    siv.add_global_callback(']', move |s| {
        let selected = *selected_servo_calibrate_end.lock().unwrap();
        let servo_id = selected as u8 + 1;
        end_calibration(s, servo_id, Arc::clone(&servo_clone_calibrate_end));
    });

    siv.set_global_callback(cursive::event::Event::Refresh, move |s| {
        match servo_clone.read_continuous() {
            Ok(data) => {
                for (i, servo_info) in data.servo.iter().enumerate() {
                    s.call_on_name(&format!("CurrPos {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.current_location));
                    });
                    // s.call_on_name(&format!("TargPos {}", i), |view: &mut TextView| {
                    //     view.set_content(format!("{:4}", servo_info.target_location));
                    // });
                    s.call_on_name(&format!("CurrSpd {}", i), |view: &mut TextView| {
                        let speed = servo_info.current_speed as u16 & 0x7FFF; // Remove 15th bit
                        let sign = if servo_info.current_speed as u16 & 0x8000 != 0 { '-' } else { '+' };
                        view.set_content(format!("{}{:4}", sign, speed));
                    });
                    // s.call_on_name(&format!("RunSpd {}", i), |view: &mut TextView| {
                    //     view.set_content(format!("{:4}", servo_info.running_speed));
                    // });
                    s.call_on_name(&format!("Load {}", i), |view: &mut TextView| {
                        let speed = servo_info.current_load as u16 & 0x3FF; // Remove 10th bit
                        let sign = if servo_info.current_load as u16 & 0x400 != 0 { '-' } else { '+' };
                        view.set_content(format!("{}{:4}", sign, speed));
                    });
                    s.call_on_name(&format!("Torque {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.torque_switch));
                    });
                    s.call_on_name(&format!("TorqLim {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.torque_limit));
                    });
                    // s.call_on_name(&format!("Accel {}", i), |view: &mut TextView| {
                    //     view.set_content(format!("{:4}", servo_info.acceleration));
                    // });
                    s.call_on_name(&format!("Volt {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:2.1}V", servo_info.current_voltage as f32 / 10.0));
                    });
                    s.call_on_name(&format!("Temp {}", i), |view: &mut TextView| {
                        view.set_content(format!("{}°C", servo_info.current_temperature));
                    });
                    s.call_on_name(&format!("Curr {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.current_current));
                    });
                    s.call_on_name(&format!("Status {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:04X}", servo_info.servo_status));
                    });
                    // s.call_on_name(&format!("Async {}", i), |view: &mut TextView| {
                    //     view.set_content(format!("{:4}", servo_info.async_write_flag));
                    // });
                    s.call_on_name(&format!("Lock {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.lock_mark));
                    });
                }
                let mut last_update = last_update_time.lock().unwrap();
                let now = Instant::now();
                let time_delta = now.duration_since(*last_update);

                let update_rate = if data.task_run_count > 0 {
                    1000.0 / (time_delta.as_millis() as f64 / data.task_run_count as f64)
                } else {
                    0.0
                };

                s.call_on_name("Task Count", |view: &mut TextView| {
                    view.set_content(format!("Task Run Count: {}", data.task_run_count));
                });
                s.call_on_name("Last Update", |view: &mut TextView| {
                    view.set_content(format!("Last Update: {:?} ago", time_delta));
                });
                s.call_on_name("Servo polling rate", |view: &mut TextView| {
                    view.set_content(format!("Servo polling rate: {:.2} Hz", update_rate));
                });

                *last_update = now;

                let selected = *selected_servo.lock().unwrap();
                let current_pos = data.servo[selected].current_location;
                CURRENT_POSITION.store(current_pos, Ordering::Relaxed);

                s.call_on_name("CalibrationPos", |view: &mut TextView| {
                    let cal_pos = CALIBRATION_POSITION.load(Ordering::Relaxed);
                    view.set_content(format!("Calibration Pos: {}", if cal_pos >= 0 { cal_pos.to_string() } else { "----".to_string() }));
                });
            }
            Err(e) => {
                s.add_layer(Dialog::info(format!("Error reading servo data: {}", e)));
            }
        }
    });

    siv.run();

    Ok(())
}

fn update_selected_row(s: &mut cursive::Cursive, selected: usize) {
    for i in 0..MAX_SERVOS {
        s.call_on_name(&format!("ID {}", i), |view: &mut TextView| {
            view.set_content(format!("{:2}", i + 1));
        });
    }
    s.call_on_name(&format!("ID {}", selected), |view: &mut TextView| {
        view.set_content(format!(">{:2}", selected + 1));
    });
}

fn update_angle_limits(s: &mut cursive::Cursive, servo_id: u8, servo: &Arc<Servo>) {
    match servo.read_angle_limits(servo_id) {
        Ok((min_angle, max_angle)) => {
            s.call_on_name("MinAngle", |view: &mut TextView| {
                view.set_content(format!("Min Angle: {}", min_angle));
            });
            s.call_on_name("MaxAngle", |view: &mut TextView| {
                view.set_content(format!("Max Angle: {}", max_angle));
            });
        }
        Err(e) => {
            s.call_on_name("MinAngle", |view: &mut TextView| {
                view.set_content("Min Angle: Error".to_string());
            });
            s.call_on_name("MaxAngle", |view: &mut TextView| {
                view.set_content("Max Angle: Error".to_string());
            });
            eprintln!("Error reading angle limits: {}", e);
        }
    }

    // Update offset
    match servo.read(servo_id, ServoRegister::PositionCorrection, 2) {
        Ok(data) => {
            let offset = i16::from_le_bytes([data[0], data[1]]);
            s.call_on_name("Offset", |view: &mut TextView| {
                view.set_content(format!("Offset: {}", offset));
            });
        }
        Err(e) => {
            s.call_on_name("Offset", |view: &mut TextView| {
                view.set_content("Offset: Error".to_string());
            });
            eprintln!("Error reading offset: {}", e);
        }
    }
}

fn open_servo_settings(s: &mut cursive::Cursive, servo_id: u8, servo: Arc<Servo>) {
    let dialog = Dialog::new()
        .title(format!("Servo {} Settings", servo_id))
        .content(
            LinearLayout::vertical()
                .child(TextView::new("Position:"))
                .child(EditView::new().with_name("position"))
                .child(TextView::new("Speed:"))
                .child(EditView::new().with_name("speed"))
                .child(TextView::new("Torque:"))
                .child(SelectView::new()
                    .item("Enabled", Arc::new(TorqueMode::Enabled))
                    .item("Disabled", Arc::new(TorqueMode::Disabled))
                    // .item("Stiff", Arc::new(TorqueMode::Stiff))
                    .with_name("torque"))
        )
        .button("Apply", move |s| {
            let position = s.call_on_name("position", |view: &mut EditView| {
                view.get_content().parse::<i16>().ok()
            }).unwrap();
            let speed = s.call_on_name("speed", |view: &mut EditView| {
                view.get_content().parse::<u16>().unwrap_or(0)
            }).unwrap();
            let torque_mode = s.call_on_name("torque", |view: &mut SelectView<Arc<TorqueMode>>| {
                view.selection().unwrap_or_else(|| Arc::new(TorqueMode::Enabled.into()))
            }).unwrap();

            // Apply settings
            if let Err(e) = servo.set_torque_mode(servo_id, (**torque_mode).clone()) {
                s.add_layer(Dialog::info(format!("Error setting torque mode: {}", e)));
            }

            // Move servo only if position is provided
            if let Some(pos) = position {
                if let Err(e) = servo.move_servo(servo_id, pos, 0, speed) {
                    s.add_layer(Dialog::info(format!("Error moving servo: {}", e)));
                }
            }

            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        })
        .with_name("servo_settings"); // Add this line to name the dialog

    s.add_layer(dialog);
}

fn toggle_servo_torque(s: &mut cursive::Cursive, servo_id: u8, servo: Arc<Servo>) {
    let servo_clone = Arc::clone(&servo);
    
    match servo_clone.read_info(servo_id) {
        Ok(info) => {
            let new_torque_mode = if info.torque_switch == 0 {
                TorqueMode::Enabled
            } else {
                TorqueMode::Disabled
            };
            
            if let Err(e) = servo_clone.set_torque_mode(servo_id, new_torque_mode) {
                s.add_layer(Dialog::info(format!("Error setting torque mode: {}", e)));
            }
        }
        Err(e) => {
            s.add_layer(Dialog::info(format!("Error reading servo info: {}", e)));
        }
    }
}

fn start_calibration(s: &mut cursive::Cursive, servo_id: u8, servo: Arc<Servo>) {
    if let Err(e) = servo.write(servo_id, ServoRegister::PositionCorrection, &[0, 0]) {
        s.add_layer(Dialog::info(format!("Error setting position correction to 0: {}", e)));
        return;
    }

    std::thread::sleep(Duration::from_millis(20));

    match servo.read_info(servo_id) {
        Ok(info) => {
            CALIBRATION_POSITION.store(info.current_location, Ordering::Relaxed);
            s.add_layer(Dialog::info(format!("Calibration started for servo {}. Current position: {}", servo_id, info.current_location)));
        }
        Err(e) => {
            s.add_layer(Dialog::info(format!("Error reading servo info: {}", e)));
        }
    }
}

fn end_calibration(s: &mut cursive::Cursive, servo_id: u8, servo: Arc<Servo>) {
    let min_pos = CALIBRATION_POSITION.load(Ordering::Relaxed);
    if min_pos < 0 {
        s.add_layer(Dialog::info("Please start calibration first by pressing '['"));
        return;
    }

    let mut max_pos = CURRENT_POSITION.load(Ordering::Relaxed);

    if max_pos < min_pos {
        max_pos += 4096;
    }

    let offset = min_pos + (max_pos - min_pos) / 2 - 2048;

    // Convert offset to 12-bit signed value
    let offset_value = if offset < 0 {
        (offset & 0x7FF) as u16 | 0x800 // Set sign bit
    } else {
        (offset & 0x7FF) as u16
    };

    // Calculate new limits
    let min_angle = 2048 - (max_pos - min_pos) / 2;
    let max_angle = 2048 + (max_pos - min_pos) / 2;

    // Write new values to EEPROM
    if let Err(e) = write_calibration_to_eeprom(servo_id, &servo, offset_value, min_angle, max_angle) {
        s.add_layer(Dialog::info(format!("Error writing calibration to EEPROM: {}", e)));
        return;
    }

    // Update the UI
    s.call_on_name("MinAngle", |view: &mut TextView| {
        view.set_content(format!("Min Angle: {}", min_angle));
    });
    s.call_on_name("MaxAngle", |view: &mut TextView| {
        view.set_content(format!("Max Angle: {}", max_angle));
    });
    s.call_on_name("Offset", |view: &mut TextView| {
        view.set_content(format!("Offset: {}", offset));
    });

    CALIBRATION_POSITION.store(-1, Ordering::Relaxed);
    s.add_layer(Dialog::info(format!("Calibration completed for servo {}. New offset: {}", servo_id, offset)));
}

fn write_calibration_to_eeprom(servo_id: u8, servo: &Servo, offset: u16, min_angle: i16, max_angle: i16) -> Result<()> {
    // Unlock EEPROM
    servo.write(servo_id, ServoRegister::LockMark, &[0])?;
    std::thread::sleep(Duration::from_millis(10));

    // Write new offset
    servo.write_servo_memory(servo_id, ServoRegister::PositionCorrection, offset)?;
    std::thread::sleep(Duration::from_millis(10));

    // Write new limits
    servo.write_servo_memory(servo_id, ServoRegister::MinAngleLimit, min_angle as u16)?;
    std::thread::sleep(Duration::from_millis(10));
    servo.write_servo_memory(servo_id, ServoRegister::MaxAngleLimit, max_angle as u16)?;
    std::thread::sleep(Duration::from_millis(10));

    // Lock EEPROM
    servo.write(servo_id, ServoRegister::LockMark, &[1])?;

    Ok(())
}