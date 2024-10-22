use tonic::{transport::Server, Request, Response, Status};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::net::IpAddr;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
use std::time::Duration;

pub mod servo_control {
    tonic::include_proto!("hal_pb");
}

use servo_control::servo_control_server::{ServoControl, ServoControlServer};
use runtime::hal::{Servo, MAX_SERVOS, ServoMultipleWriteCommand, ServoData, ServoMode, ServoDirection, ServoRegister};
use servo_control::{Empty, JointPositions, WifiCredentials, ServoId, ServoInfo, ServoIds, IdChange, ChangeIdResponse, ServoInfoResponse, servo_info_response, change_id_response, VideoStreamUrls, CalibrationResponse, CalibrationStatus};

#[derive(Debug)]
pub struct StsServoControl {
    servo: Arc<Mutex<Servo>>,
    last_positions: Arc<Mutex<ServoData>>,
    calibrating_servo: Arc<Mutex<Option<u8>>>,
    calibration_running: Arc<AtomicBool>,
}

impl StsServoControl {
    pub fn new() -> Result<Self> {
        let servo = Servo::new()?;
        servo.enable_readout()?;
        let initial_data = servo.read_continuous()?;
        Ok(Self {
            servo: Arc::new(Mutex::new(servo)),
            last_positions: Arc::new(Mutex::new(initial_data)),
            calibrating_servo: Arc::new(Mutex::new(None)),
            calibration_running: Arc::new(AtomicBool::new(false)),
        })
    }

    fn get_interface_ip(interface: &str) -> Option<IpAddr> {
        let output = Command::new("ip")
            .args(&["addr", "show", interface])
            .output()
            .ok()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let ip_regex = Regex::new(r"inet\s+(\d+\.\d+\.\d+\.\d+)").ok()?;
        
        ip_regex.captures(&output_str)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    async fn calibrate_servo(&self, servo_id: u8, calibration_speed: u16, current_threshold: f32) -> Result<(), Status> {
        let servo = self.servo.clone();
        let calibrating_servo = self.calibrating_servo.clone();
        let calibration_running = self.calibration_running.clone();

        println!("Starting calibration for servo {}", servo_id);

        task::spawn(async move {
            let servo = servo.lock().await;
            println!("Acquired servo lock");
            servo.disable_readout().unwrap();
            println!("Disabled readout");
            servo.set_mode(servo_id, ServoMode::ConstantSpeed).unwrap();
            println!("Set servo mode to ConstantSpeed");

            // torque limit to 10%
            servo.write_servo_memory(servo_id, runtime::hal::ServoRegister::TorqueLimit, 150).unwrap();
            println!("Set torque limit to 10%");

            let mut max_forward = 0;
            let mut max_backward = 0;

            for pass in 0..2 {
                println!("Starting pass {}", pass);
                let direction = if pass == 0 { ServoDirection::Clockwise } else { ServoDirection::Counterclockwise };
                servo.set_speed(servo_id, calibration_speed, direction).unwrap();
                println!("Set speed to {} in {:?} direction", calibration_speed, direction);

                let mut threshold_exceeded_count = 0;

                loop {
                    if !calibration_running.load(Ordering::SeqCst) {
                        println!("Calibration cancelled");
                        servo.set_speed(servo_id, 0, ServoDirection::Clockwise).unwrap();
                        return;
                    }

                    let mut info = servo.read_info(servo_id).unwrap();
                    let mut retry_count = 0;
                    while info.current_current == 0 && info.current_location == 0 && retry_count < 3 {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        info = servo.read_info(servo_id).unwrap();
                        retry_count += 1;
                    }
                    let position = info.current_location;
                    let current = info.current_current as f32 * 6.5 / 100.0;
                    println!("Position: {}, Current: {}", position, current);

                    if current > current_threshold {
                        threshold_exceeded_count += 1;
                        println!("Current threshold exceeded {} time(s) in a row", threshold_exceeded_count);
                        
                        if threshold_exceeded_count >= 3 {
                            println!("Current threshold exceeded 3 times in a row");
                            for _ in 0..3 { 
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                servo.set_speed(servo_id, 0, direction).unwrap();
                                println!("Stopped servo");
                            }
                            tokio::time::sleep(Duration::from_millis(100)).await;

                            servo.set_speed(servo_id, calibration_speed, opposite_direction(direction)).unwrap();
                            println!("Moving in opposite direction");
                            tokio::time::sleep(Duration::from_millis(350)).await;

                            servo.set_speed(servo_id, 0, opposite_direction(direction)).unwrap();
                            println!("Stopped servo");
                            tokio::time::sleep(Duration::from_millis(100)).await;

                            let info = servo.read_info(servo_id).unwrap();

                            if direction == ServoDirection::Clockwise {
                                max_forward = info.current_location;
                                println!("Set max_forward to {}", max_forward);
                            } else {
                                max_backward = info.current_location;
                                println!("Set max_backward to {}", max_backward);
                            }

                            break;
                        }
                    } else {
                        threshold_exceeded_count = 0;
                    }

                    tokio::time::sleep(Duration::from_millis(10)).await;
                }

                if pass < 1 {
                    println!("Waiting between passes");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }

            servo.write(servo_id, ServoRegister::LockMark, &[0]).unwrap();
            servo.set_speed(servo_id, 0, ServoDirection::Clockwise).unwrap();
            servo.write_servo_memory(servo_id, ServoRegister::TorqueLimit, 600).unwrap();
            servo.set_mode(servo_id, ServoMode::Position).unwrap();
            servo.write(servo_id, ServoRegister::LockMark, &[1]).unwrap();

            println!("Stopped servo");

            servo.enable_readout().unwrap();
            println!("Enabled readout");
            *calibrating_servo.lock().await = None;
            calibration_running.store(false, Ordering::SeqCst);
            Self::calculate_and_write_calibration(servo_id, &servo, max_backward, max_forward).await.unwrap();
            println!("Calibration completed, max_forward: {}, max_backward: {}", max_forward, max_backward);
        });

        Ok(())
    }

    async fn calculate_and_write_calibration(servo_id: u8, servo: &Servo, min_pos: i16, max_pos: i16) -> Result<(), Status> {
        let mut max_pos = max_pos;

        if max_pos < min_pos {
            max_pos += 4096;
        }
    
        let offset_value = min_pos + (max_pos - min_pos) / 2 - 2048;
    
        // Convert offset to 12-bit signed value
        let offset_value = if offset_value < 0 {
            offset_value.abs() as u16 | 0x800 // (set negative bit)
        } else {
            if offset_value > 2048 {
                (offset_value - 4096).abs() as u16 | 0x800
            } else {
                offset_value as u16
            }
        };
    
        // Calculate new limits
        let min_angle = 2048 - (max_pos - min_pos) / 2;
        let max_angle = 2048 + (max_pos - min_pos) / 2;

        println!("Writing calibration, offset: {}, min_angle: {}, max_angle: {}", offset_value, min_angle, max_angle);

        // Unlock EEPROM
        servo.write(servo_id, ServoRegister::LockMark, &[0])
            .map_err(|e| Status::internal(format!("Failed to unlock EEPROM: {}", e)))?;
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Write new offset
        servo.write_servo_memory(servo_id, ServoRegister::PositionCorrection, offset_value)
            .map_err(|e| Status::internal(format!("Failed to write offset: {}", e)))?;
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Write new limits
        for _ in 0..3 {
            servo.write_servo_memory(servo_id, ServoRegister::MinAngleLimit, min_angle as u16)
                .map_err(|e| Status::internal(format!("Failed to write MinAngleLimit: {}", e)))?;
            tokio::time::sleep(Duration::from_millis(20)).await;
            let read_min = servo.read(servo_id, ServoRegister::MinAngleLimit, 2)
                .map_err(|e| Status::internal(format!("Failed to read MinAngleLimit: {}", e)))?;
            let read_min = u16::from_le_bytes([read_min[0], read_min[1]]);
            if read_min == min_angle as u16 {
                break;
            }
        }

        for _ in 0..3 {
            servo.write_servo_memory(servo_id, ServoRegister::MaxAngleLimit, max_angle as u16)
                .map_err(|e| Status::internal(format!("Failed to write MaxAngleLimit: {}", e)))?;
            tokio::time::sleep(Duration::from_millis(20)).await;
            let read_max = servo.read(servo_id, ServoRegister::MaxAngleLimit, 2)
                .map_err(|e| Status::internal(format!("Failed to read MaxAngleLimit: {}", e)))?;
            let read_max = u16::from_le_bytes([read_max[0], read_max[1]]);
            if read_max == max_angle as u16 {
                break;
            }
        }

        // Lock EEPROM
        servo.write(servo_id, ServoRegister::LockMark, &[1])
            .map_err(|e| Status::internal(format!("Failed to lock EEPROM: {}", e)))?;

        Ok(())
    }
}

#[tonic::async_trait]
impl ServoControl for StsServoControl {
    async fn get_positions(&self, _request: Request<Empty>) -> Result<Response<JointPositions>, Status> {
        let servo: tokio::sync::MutexGuard<'_, Servo> = self.servo.lock().await;
        let servo_data = servo.read_continuous().map_err(|e| Status::internal(e.to_string()))?;
        
        // Update last_positions
        *self.last_positions.lock().await = servo_data.clone();
        
        let positions = JointPositions {
            positions: servo_data.servo.iter()
                .take(MAX_SERVOS)
                .enumerate()
                .map(|(id, info)| servo_control::JointPosition {
                    id: (id + 1) as i32, // Add 1 to make IDs start from 1
                    position: Servo::raw_to_degrees(info.current_location as u16),
                })
                .collect(),
        };
        Ok(Response::new(positions))
    }

    async fn set_positions(&self, request: Request<JointPositions>) -> Result<Response<Empty>, Status> {
        let positions = request.into_inner();
        let servo = self.servo.lock().await;
        let mut last_positions = self.last_positions.lock().await;
        let calibration_running = self.calibration_running.clone();

        if calibration_running.load(Ordering::SeqCst) {
            return Err(Status::internal("Calibration is in progress"));
        }
        
        let mut cmd = ServoMultipleWriteCommand {
            only_write_positions: 1,
            ids: [0; MAX_SERVOS],
            positions: [0; MAX_SERVOS],
            times: [0; MAX_SERVOS],
            speeds: [0; MAX_SERVOS],
        };

        for i in 0..MAX_SERVOS {
            let servo_id = (i + 1) as u32; // Add 1 because IDs start from 1
            let position = positions.positions.iter()
                .find(|p| p.id == servo_id as i32)
                .map(|p| p.position)
                .unwrap_or_else(|| Servo::raw_to_degrees(last_positions.servo[i].current_location as u16));

            cmd.ids[i] = servo_id as u8;
            cmd.positions[i] = Servo::degrees_to_raw(position) as i16;
            // You can set times and speeds here if needed
            cmd.times[i] = 0;
            cmd.speeds[i] = 0;

            last_positions.servo[i].current_location = Servo::degrees_to_raw(position) as i16;
        }

        servo.write_multiple(&cmd)
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(Empty {}))
    }

    async fn set_wifi_info(&self, request: Request<WifiCredentials>) -> Result<Response<Empty>, Status> {
        println!("Got a request to set WiFi info: {:?}", request);
        let wifi_info = request.into_inner();
        
        let config_content = format!(
            "ctrl_interface=/var/run/wpa_supplicant
ap_scan=1
update_config=1

network={{
  ssid=\"{}\"
  psk=\"{}\"
  key_mgmt=WPA-PSK
}}",
            wifi_info.ssid, wifi_info.password
        );

        let path = Path::new("/boot/wpa_supplicant.conf");
        let mut file = File::create(path).map_err(|e| Status::internal(format!("Failed to create file: {}", e)))?;
        
        file.write_all(config_content.as_bytes())
            .map_err(|e| Status::internal(format!("Failed to write to file: {}", e)))?;

        println!("WiFi configuration written to /boot/wpa_supplicant.conf");

        match Command::new("sync").output() {
            Ok(_) => println!("Sync command executed successfully"),
            Err(e) => eprintln!("Failed to execute sync command: {}", e),
        }

        Ok(Response::new(Empty {}))
    }

    async fn scan(&self, _request: Request<Empty>) -> Result<Response<ServoIds>, Status> {
        let servo = self.servo.lock().await;
        let mut ids = Vec::new();
        
        for id in 0..MAX_SERVOS as u8 {
            if servo.scan(id).map_err(|e| Status::internal(e.to_string()))? {
                ids.push(id as u32);
            }
        }
        
        Ok(Response::new(ServoIds { ids: ids.iter().map(|id| *id as i32).collect() }))
    }

    async fn get_servo_info(&self, request: Request<ServoId>) -> Result<Response<ServoInfoResponse>, Status> {
        let id = request.into_inner().id as u8;
        let servo = self.servo.lock().await;
        
        let servo_info = servo.read_info(id).map_err(|e| Status::internal(e.to_string()))?;
        let (min_position, max_position) = servo.read_angle_limits(id).map_err(|e| Status::internal(e.to_string()))?;
        let min_position = Servo::raw_to_degrees(min_position as u16);
        let max_position = Servo::raw_to_degrees(max_position as u16);
        
        let info = ServoInfo {
            id: id as i32,
            temperature: servo_info.current_temperature as f32,
            current: servo_info.current_current as f32,
            voltage: ((servo_info.current_voltage as f32 / 10.0) * 10.0).round() / 10.0,
            speed: servo_info.current_speed as f32 / 4096.0 * 360.0,
            current_position: Servo::raw_to_degrees(servo_info.current_location as u16),
            min_position: min_position as f32,
            max_position: max_position as f32,
        };
        Ok(Response::new(ServoInfoResponse {
            result: Some(servo_info_response::Result::Info(info)),
        }))
    }

    async fn change_id(&self, request: Request<IdChange>) -> Result<Response<ChangeIdResponse>, Status> {
        let id_change = request.into_inner();
        let servo = self.servo.lock().await;
        
        // First, check if the new ID is already in use
        if servo.scan(id_change.new_id as u8).map_err(|e| Status::internal(e.to_string()))? {
            return Ok(Response::new(ChangeIdResponse {
                result: Some(change_id_response::Result::Error(servo_control::ErrorInfo {
                    message: "New ID is already in use".to_string(),
                    code: 1,
                })),
            }));
        }
        
        // Change the ID
        servo.write(id_change.old_id as u8, runtime::hal::ServoRegister::ID, &[id_change.new_id as u8])
            .map_err(|e| Status::internal(e.to_string()))?;
        
        // Verify the change
        if servo.scan(id_change.new_id as u8).map_err(|e| Status::internal(e.to_string()))? {
            Ok(Response::new(ChangeIdResponse {
                result: Some(change_id_response::Result::Success(true)),
            }))
        } else {
            Ok(Response::new(ChangeIdResponse {
                result: Some(change_id_response::Result::Error(servo_control::ErrorInfo {
                    message: "Failed to change ID".to_string(),
                    code: 2,
                })),
            }))
        }
    }

    async fn start_calibration(&self, request: Request<ServoId>) -> Result<Response<CalibrationResponse>, Status> {
        let servo_id = request.into_inner().id as u8;
        let mut calibrating_servo = self.calibrating_servo.lock().await;

        if calibrating_servo.is_some() {
            return Ok(Response::new(CalibrationResponse {
                result: Some(servo_control::calibration_response::Result::Error(servo_control::ErrorInfo {
                    message: "Another calibration is already in progress".to_string(),
                    code: 1,
                })),
            }));
        }

        *calibrating_servo = Some(servo_id);
        self.calibration_running.store(true, Ordering::SeqCst);
        
        // Start calibration in a separate task
        let calibration_speed = 300; // You may want to make this configurable
        let current_threshold = 600.0; // You may want to make this configurable
        self.calibrate_servo(servo_id, calibration_speed, current_threshold).await?;

        Ok(Response::new(CalibrationResponse {
            result: Some(servo_control::calibration_response::Result::Success(true)),
        }))
    }

    async fn cancel_calibration(&self, _request: Request<ServoId>) -> Result<Response<CalibrationResponse>, Status> {
        let mut calibrating_servo = self.calibrating_servo.lock().await;

        if calibrating_servo.is_none() {
            return Ok(Response::new(CalibrationResponse {
                result: Some(servo_control::calibration_response::Result::Error(servo_control::ErrorInfo {
                    message: "No calibration is currently in progress".to_string(),
                    code: 2,
                })),
            }));
        }

        self.calibration_running.store(false, Ordering::SeqCst);
        *calibrating_servo = None;

        Ok(Response::new(CalibrationResponse {
            result: Some(servo_control::calibration_response::Result::Success(true)),
        }))
    }

    async fn start_video_stream(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        // Stub implementation
        Ok(Response::new(Empty {}))
    }

    async fn stop_video_stream(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        // Stub implementation
        Ok(Response::new(Empty {}))
    }

    async fn get_video_stream_urls(&self, _request: Request<Empty>) -> Result<Response<VideoStreamUrls>, Status> {
        let usb0_ip = Self::get_interface_ip("usb0").unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
        let wlan0_ip = Self::get_interface_ip("wlan0").unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
        let stream_id = "s1";
        let channel_id = "0";

        let make_url = |ip: IpAddr, protocol: &str, port: u16, path: &str| -> String {
            format!("{}://{}:{}{}", protocol, ip, port, path)
        };

        let create_urls = |ip: IpAddr| -> (String, String, String, String, String) {
            (
                make_url(ip, "http", 8083, &format!("/stream/{}/channel/{}/webrtc", stream_id, channel_id)),
                make_url(ip, "http", 8083, &format!("/stream/{}/channel/{}/hls/live/index.m3u8", stream_id, channel_id)),
                make_url(ip, "http", 8083, &format!("/stream/{}/channel/{}/hlsll/live/index.m3u8", stream_id, channel_id)),
                make_url(ip, "ws", 8083, &format!("/stream/{}/channel/{}/mse?uuid={}&channel={}", stream_id, channel_id, stream_id, channel_id)),
                make_url(ip, "rtsp", 553, &format!("/{}/{}", stream_id, channel_id)),
            )
        };

        let (usb0_webrtc, usb0_hls, usb0_hls_ll, usb0_mse, usb0_rtsp) = create_urls(usb0_ip);
        let (wlan0_webrtc, wlan0_hls, wlan0_hls_ll, wlan0_mse, wlan0_rtsp) = create_urls(wlan0_ip);

        let urls = VideoStreamUrls {
            webrtc: vec![usb0_webrtc, wlan0_webrtc],
            hls: vec![usb0_hls, wlan0_hls],
            hls_ll: vec![usb0_hls_ll, wlan0_hls_ll],
            mse: vec![usb0_mse, wlan0_mse],
            rtsp: vec![usb0_rtsp, wlan0_rtsp],
        };

        Ok(Response::new(urls))
    }

    async fn get_calibration_status(&self, _request: Request<Empty>) -> Result<Response<CalibrationStatus>, Status> {
        let calibrating_servo = self.calibrating_servo.lock().await;
        Ok(Response::new(CalibrationStatus {
            is_calibrating: calibrating_servo.is_some(),
            calibrating_servo_id: calibrating_servo.unwrap_or(0) as i32,
        }))
    }
}

fn opposite_direction(direction: ServoDirection) -> ServoDirection {
    match direction {
        ServoDirection::Clockwise => ServoDirection::Counterclockwise,
        ServoDirection::Counterclockwise => ServoDirection::Clockwise,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let servo_control = StsServoControl::new()?;

    let service = tower::ServiceBuilder::new()
        .layer(tonic_web::GrpcWebLayer::new())
        .service(ServoControlServer::new(servo_control));

    Server::builder()
        // GrpcWeb is over http1 so we must enable it.
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
