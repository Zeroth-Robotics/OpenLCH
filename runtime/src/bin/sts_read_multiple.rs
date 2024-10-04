use anyhow::Result;
use runtime::hal::{Servo, MAX_SERVOS};
use cursive::views::{TextView, LinearLayout, Panel, Dialog, DummyView};
use cursive::traits::*;
use std::sync::Arc;

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
        .child(TextView::new("Curr Pos").center().fixed_width(8))
        .child(TextView::new("Targ Pos").center().fixed_width(8))
        .child(TextView::new("Curr Spd").center().fixed_width(8))
        .child(TextView::new("Run Spd").center().fixed_width(8))
        .child(TextView::new("Load").center().fixed_width(8))
        .child(TextView::new("Torque").center().fixed_width(8))
        .child(TextView::new("Torq Lim").center().fixed_width(8))
        .child(TextView::new("Accel").center().fixed_width(8))
        .child(TextView::new("Volt").center().fixed_width(6))
        .child(TextView::new("Temp").center().fixed_width(6))
        .child(TextView::new("Curr").center().fixed_width(6))
        .child(TextView::new("Status").center().fixed_width(8))
        .child(TextView::new("Async").center().fixed_width(6))
        .child(TextView::new("Lock").center().fixed_width(6));
    layout.add_child(Panel::new(header).title("Servo Data"));

    // Add rows for each servo
    for i in 0..MAX_SERVOS {
        let row = LinearLayout::horizontal()
            .child(TextView::new(format!("{:2}", i)).center().fixed_width(4))
            .child(TextView::new("----").center().with_name(format!("CurrPos {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("TargPos {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("CurrSpd {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("RunSpd {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Load {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Torque {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("TorqLim {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Accel {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Volt {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Temp {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Curr {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Status {}", i)).fixed_width(8))
            .child(TextView::new("----").center().with_name(format!("Async {}", i)).fixed_width(6))
            .child(TextView::new("----").center().with_name(format!("Lock {}", i)).fixed_width(6));
        layout.add_child(row);
    }

    // Add a dummy view to push the task count to the bottom
    layout.add_child(DummyView.full_height());

    // Add task run count at the bottom
    layout.add_child(
        Panel::new(TextView::new("Task Run Count: 0").with_name("Task Count"))
            .title("Statistics")
            .full_width()
    );

    siv.add_fullscreen_layer(layout);

    // Set up a timer to update the UI
    siv.set_fps(30);

    // Clone Arc for the callback
    let servo_clone = Arc::clone(&servo);

    siv.add_global_callback('q', |s| s.quit());

    siv.set_global_callback(cursive::event::Event::Refresh, move |s| {
        match servo_clone.read_continuous() {
            Ok(data) => {
                for (i, servo_info) in data.servo.iter().enumerate() {
                    s.call_on_name(&format!("CurrPos {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.current_location));
                    });
                    s.call_on_name(&format!("TargPos {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.target_location));
                    });
                    s.call_on_name(&format!("CurrSpd {}", i), |view: &mut TextView| {
                        let speed = servo_info.current_speed as u16 & 0x7FFF; // Remove 15th bit
                        let sign = if servo_info.current_speed as u16 & 0x8000 != 0 { '-' } else { '+' };
                        view.set_content(format!("{}{:4}", sign, speed));
                    });
                    s.call_on_name(&format!("RunSpd {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.running_speed));
                    });
                    s.call_on_name(&format!("Load {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.current_load));
                    });
                    s.call_on_name(&format!("Torque {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.torque_switch));
                    });
                    s.call_on_name(&format!("TorqLim {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.torque_limit));
                    });
                    s.call_on_name(&format!("Accel {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.acceleration));
                    });
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
                    s.call_on_name(&format!("Async {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.async_write_flag));
                    });
                    s.call_on_name(&format!("Lock {}", i), |view: &mut TextView| {
                        view.set_content(format!("{:4}", servo_info.lock_mark));
                    });
                }
                s.call_on_name("Task Count", |view: &mut TextView| {
                    view.set_content(format!("Task Run Count: {}", data.task_run_count));
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