use tonic::{transport::Server, Request, Response, Status};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::net::IpAddr;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
use std::time::Duration;
use std::env;
use runtime::hal::{Servo, IMU, MAX_SERVOS, ServoMultipleWriteCommand, ServoData, ServoMode, ServoDirection, ServoRegister, TorqueMode};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use tokio::fs;
use std::path::PathBuf;
use std::pin::Pin;
use chrono::prelude::*;
use tokio_stream::{self, StreamExt, Stream};

pub mod servo_control {
    tonic::include_proto!("hal_pb");
}

use servo_control::servo_control_server::{ServoControl, ServoControlServer};
use servo_control::{Empty, JointPositions, WifiCredentials, ServoId, ServoInfo, ServoIds, IdChange, ChangeIdResponse, ServoInfoResponse, servo_info_response, change_id_response, VideoStreamUrls, CalibrationResponse, CalibrationStatus, TorqueSettings, TorqueEnableSettings, ImuData, Vector3, AudioChunk, UploadResponse, PlayRequest, RecordingConfig, CalibrationRequest};

#[derive(Debug)]
pub struct StsServoControl {
    servo: Arc<Mutex<Servo>>,
    imu: Arc<Mutex<Option<IMU>>>,
    last_positions: Arc<Mutex<ServoData>>,
    calibrating_servo: Arc<Mutex<Option<u8>>>,
    calibration_running: Arc<AtomicBool>,
    audio_files: Arc<RwLock<HashMap<String, PathBuf>>>,
    recording_running: Arc<AtomicBool>,
}

impl StsServoControl {
    pub fn new() -> Result<Self> {
        let servo = Servo::new()?;
        let imu = IMU::new().ok();
        servo.enable_readout()?;
        let initial_data = servo.read_continuous()?;
        
        Ok(Self {
            servo: Arc::new(Mutex::new(servo)),
            imu: Arc::new(Mutex::new(imu)),
            last_positions: Arc::new(Mutex::new(initial_data)),
            calibrating_servo: Arc::new(Mutex::new(None)),
            calibration_running: Arc::new(AtomicBool::new(false)),
            audio_files: Arc::new(RwLock::new(HashMap::new())),
            recording_running: Arc::new(AtomicBool::new(false)),
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

        task::spawn(async move {
            let servo = servo.lock().await;

            let is_hls = servo.get_servo_version(servo_id) == 10;

            servo.disable_movement().unwrap();


            servo.disable_readout().unwrap();

            if is_hls {
                servo.set_mode(servo_id, ServoMode::PWMOpenLoop).unwrap();
            } else {
                servo.set_mode(servo_id, ServoMode::ConstantSpeed).unwrap();
            }

            servo.write_servo_memory(servo_id, runtime::hal::ServoRegister::TorqueLimit, 150).unwrap();

            let mut max_forward = 0;
            let mut max_backward = 0;

            for pass in 0..2 {
                let direction = if pass == 0 { ServoDirection::Clockwise } else { ServoDirection::Counterclockwise };

                if is_hls {
                    servo.set_current(servo_id, (current_threshold / 10.) as u16, direction).unwrap();
                } else {
                    servo.set_speed(servo_id, calibration_speed, direction).unwrap();
                }


                let mut threshold_exceeded_count = 0;

                loop {
                    if !calibration_running.load(Ordering::SeqCst) {
                        servo.set_speed(servo_id, 0, ServoDirection::Clockwise).unwrap();
                        if is_hls {
                            servo.set_current(servo_id, 0, ServoDirection::Clockwise).unwrap();
                            servo.set_mode(servo_id, ServoMode::Position).unwrap();
                            servo.write(servo_id, ServoRegister::TorqueSwitch, &[0]).unwrap();
                        }
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
                    let current = info.current_current as f32;
                    
                    println!("Current location: {}, current: {}", position, current);


                    if !is_hls && current > current_threshold {
                        threshold_exceeded_count += 1;
                        
                        if threshold_exceeded_count >= 3 {
                            for _ in 0..3 { 
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                servo.set_speed(servo_id, 0, direction).unwrap();
                            }
                            tokio::time::sleep(Duration::from_millis(100)).await;

                            servo.set_speed(servo_id, calibration_speed, opposite_direction(direction)).unwrap();
                            tokio::time::sleep(Duration::from_millis(350)).await;

                            servo.set_speed(servo_id, 0, opposite_direction(direction)).unwrap();
                            tokio::time::sleep(Duration::from_millis(100)).await;

                            let info = servo.read_info(servo_id).unwrap();

                            if direction == ServoDirection::Clockwise {
                                max_forward = info.current_location;
                            } else {
                                max_backward = info.current_location;
                            }

                            break;
                        }
                    } else if is_hls && info.mobile_sign == 0 {
                        threshold_exceeded_count += 1;

                        if threshold_exceeded_count >= 10 {
                            if direction == ServoDirection::Clockwise {
                                max_forward = info.current_location;
                            } else {
                                max_backward = info.current_location;
                            }

                            servo.set_current(servo_id, 0, direction).unwrap();
                            break;
                        }
                    } else {
                        threshold_exceeded_count = 0;
                    }

                    tokio::time::sleep(Duration::from_millis(10)).await;
                }

                if pass < 1 {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }

            servo.write(servo_id, ServoRegister::LockMark, &[0]).unwrap();
            servo.set_speed(servo_id, 0, ServoDirection::Clockwise).unwrap();
            servo.write_servo_memory(servo_id, ServoRegister::TorqueLimit, 600).unwrap();
            servo.set_mode(servo_id, ServoMode::Position).unwrap();
            servo.write(servo_id, ServoRegister::LockMark, &[1]).unwrap();

            servo.enable_readout().unwrap();
            *calibrating_servo.lock().await = None;
            calibration_running.store(false, Ordering::SeqCst);
            Self::calculate_and_write_calibration(servo_id, &servo, max_backward, max_forward).await.unwrap();
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

    fn is_process_running(process_name: &str) -> bool {
        Command::new("ps")
            .args(&["-A"])
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|child| {
                let output = child.wait_with_output()?;
                Ok(String::from_utf8_lossy(&output.stdout).contains(process_name))
            })
            .unwrap_or(false)
    }

    fn start_process(process_name: &str, args: &[&str]) -> Result<(), std::io::Error> {
        Command::new(process_name)
            .args(args)
            .env("LD_LIBRARY_PATH", "/mnt/system/lib:/mnt/system/usr/lib")
            .env("PATH", "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:/mnt/system/usr/bin:/mnt/system/usr/sbin")
            .spawn()?;
        Ok(())
    }

    fn stop_process(process_name: &str) -> Result<(), std::io::Error> {
        Command::new("pkill")
            .args(&["-SIGINT", process_name])  // Send SIGINT (Ctrl+C) signal
            .status()?;
        Ok(())
    }

    async fn save_audio_chunk(audio_id: &str, data: &[u8]) -> Result<PathBuf, Status> {
        let audio_dir = PathBuf::from("/tmp/audio");
        fs::create_dir_all(&audio_dir).await
            .map_err(|e| Status::internal(format!("Failed to create audio directory: {}", e)))?;

        let file_path = audio_dir.join(format!("{}.wav", audio_id));
        fs::write(&file_path, data).await
            .map_err(|e| Status::internal(format!("Failed to write audio file: {}", e)))?;

        Ok(file_path)
    }

    fn play_audio(file_path: &Path, volume: f32) -> Result<(), Status> {
        let _volume_db = (20.0 * volume.log10()).round() as i32;
        // let volume_arg = if volume_db <= -144 {
        //     "-M".to_string()  // Mute
        // } else {
        //     format!("-v {}", volume_db)
        // };

        Command::new("tinyplay")
            .args(&[
                file_path.to_str().unwrap(),
                "-D", "1",
                "-c", "1",
            ])
            // .arg(&volume_arg)
            .status()
            .map_err(|e| Status::internal(format!("Failed to play audio: {}", e)))?;

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
                    speed: {
                        let speed_raw = info.current_speed as u16;
                        let speed_magnitude = speed_raw & 0x7FFF; // Remove 15th bit
                        let speed_sign = if speed_raw & 0x8000 != 0 { -1.0 } else { 1.0 };
                        speed_sign * (speed_magnitude as f32 * 360.0 / 4096.0)
                    },
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
        
        for id in 0..100 as u8 {
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
        
        // Handle speed conversion
        let speed_raw = servo_info.current_speed as u16;
        let speed_magnitude = speed_raw & 0x7FFF; // Remove 15th bit
        let speed_sign = if speed_raw & 0x8000 != 0 { -1.0 } else { 1.0 };
        let speed = speed_sign * (speed_magnitude as f32 * 360.0 / 4096.0);

        let info = ServoInfo {
            id: id as i32,
            temperature: servo_info.current_temperature as f32,
            current: servo_info.current_current as f32,
            voltage: ((servo_info.current_voltage as f32 / 10.0) * 10.0).round() / 10.0,
            speed,
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
        servo.write(id_change.old_id as u8, ServoRegister::LockMark, &[0])
            .map_err(|e| Status::internal(e.to_string()))?;
        servo.write(id_change.old_id as u8, runtime::hal::ServoRegister::ID, &[id_change.new_id as u8])
            .map_err(|e| Status::internal(e.to_string()))?;
        servo.write(id_change.old_id as u8, ServoRegister::LockMark, &[1])
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

    async fn start_calibration(
        &self,
        request: Request<CalibrationRequest>
    ) -> Result<Response<CalibrationResponse>, Status> {
        let request = request.into_inner();
        let servo_id = request.servo_id as u8;
        let calibration_speed = request.calibration_speed;
        let current_threshold = request.current_threshold;

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
        self.calibrate_servo(servo_id, calibration_speed as u16, current_threshold).await?;

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
        if !Self::is_process_running("cvi_camera") {
            Self::start_process("/usr/local/bin/cvi_camera", &[])
                .map_err(|e| Status::internal(format!("Failed to start cvi_camera: {}", e)))?;
        }

        if !Self::is_process_running("RTSPtoWeb") {
            Self::start_process("/usr/local/bin/RTSPtoWeb", &["-config", "/etc/rtsp2web.json"])
                .map_err(|e| Status::internal(format!("Failed to start RTSPtoWeb: {}", e)))?;
        }

        Ok(Response::new(Empty {}))
    }

    async fn stop_video_stream(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        if Self::is_process_running("cvi_camera") {
            Self::stop_process("cvi_camera")
                .map_err(|e| Status::internal(format!("Failed to stop cvi_camera: {}", e)))?;
        }

        if Self::is_process_running("RTSPtoWeb") {
            Self::stop_process("RTSPtoWeb")
                .map_err(|e| Status::internal(format!("Failed to stop RTSPtoWeb: {}", e)))?;
        }

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

    async fn set_torque(&self, request: Request<TorqueSettings>) -> Result<Response<Empty>, Status> {
        let torque_settings = request.into_inner();
        let servo = self.servo.lock().await;

        for setting in torque_settings.settings {
            let torque_value = (setting.torque * 10.0) as u16; // Convert 0-100% to 0-1000
            servo.write_servo_memory(setting.id as u8, ServoRegister::TorqueLimit, torque_value)
                .map_err(|e| Status::internal(format!("Failed to set torque for servo {}: {}", setting.id, e)))?;
        }

        Ok(Response::new(Empty {}))
    }

    async fn set_torque_enable(&self, request: Request<TorqueEnableSettings>) -> Result<Response<Empty>, Status> {
        let torque_enable_settings = request.into_inner();
        let servo = self.servo.lock().await;

        for setting in torque_enable_settings.settings {
            let torque_mode = if setting.enable {
                TorqueMode::Enabled
            } else {
                TorqueMode::Disabled
            };
            servo.set_torque_mode(setting.id as u8, torque_mode)
                .map_err(|e| Status::internal(format!("Failed to set torque enable for servo {}: {}", setting.id, e)))?;
        }

        Ok(Response::new(Empty {}))
    }

    async fn get_imu_data(&self, _request: Request<Empty>) -> Result<Response<ImuData>, Status> {
        let mut imu = self.imu.lock().await;
        
        let imu_data = match imu.as_mut() {
            Some(imu) => imu.read_data()
                .map_err(|e| Status::internal(format!("Failed to read IMU data: {}", e)))?,
            None => return Err(Status::unavailable("IMU is not available")),
        };

        Ok(Response::new(ImuData {
            gyro: Some(Vector3 {
                x: imu_data.gyro_x,
                y: imu_data.gyro_y,
                z: imu_data.gyro_z,
            }),
            accel: Some(Vector3 {
                x: imu_data.acc_x,
                y: imu_data.acc_y,
                z: imu_data.acc_z,
            }),
        }))
    }

    async fn upload_audio(&self, request: Request<tonic::Streaming<AudioChunk>>) -> Result<Response<UploadResponse>, Status> {
        let mut stream = request.into_inner();
        let audio_id = format!("audio_{}", chrono::Utc::now().timestamp_nanos());
        let mut all_data = Vec::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Status::internal(format!("Failed to receive chunk: {}", e)))?;
            all_data.extend(chunk.data);
        }

        let file_path = Self::save_audio_chunk(&audio_id, &all_data).await?;
        
        {
            let mut audio_files = self.audio_files.write().await;
            audio_files.insert(audio_id.clone(), file_path);
        }

        Ok(Response::new(UploadResponse {
            audio_id,
            result: Some(servo_control::upload_response::Result::Success(true)),
        }))
    }

    async fn play_audio(&self, request: Request<PlayRequest>) -> Result<Response<Empty>, Status> {
        let play_request = request.into_inner();
        let audio_files = self.audio_files.read().await;
        
        let file_path = audio_files.get(&play_request.audio_id)
            .ok_or_else(|| Status::not_found("Audio file not found"))?;

        Self::play_audio(file_path, play_request.volume)?;
        
        Ok(Response::new(Empty {}))
    }

    async fn start_recording(&self, request: Request<RecordingConfig>) -> Result<Response<Empty>, Status> {
        if self.recording_running.load(Ordering::SeqCst) {
            return Err(Status::already_exists("Recording is already in progress"));
        }

        let config = request.into_inner();
        let recording_running = self.recording_running.clone();
        
        // Create output directory if it doesn't exist
        tokio::fs::create_dir_all("/tmp/audio").await
            .map_err(|e| Status::internal(format!("Failed to create audio directory: {}", e)))?;

        recording_running.store(true, Ordering::SeqCst);
        
        tokio::spawn(async move {
            let channels = config.channels.to_string();
            let sample_rate = config.sample_rate.to_string();
            
            let mut child = match Command::new("tinycap")
                .args(&[
                    "/tmp/audio/recording.wav",
                    "-D", "0",
                    "-c", &channels,
                    "-r", &sample_rate,
                    "-b", "16",
                ])
                .spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        eprintln!("Failed to start recording: {}", e);
                        recording_running.store(false, Ordering::SeqCst);
                        return;
                    }
                };

            while recording_running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            // Send SIGINT to the process
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;
                signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).unwrap();
            }

            // Wait for the process to finish
            let _ = child.wait();
        });

        Ok(Response::new(Empty {}))
    }

    async fn stop_recording(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        if !self.recording_running.load(Ordering::SeqCst) {
            return Err(Status::not_found("No recording in progress"));
        }

        self.recording_running.store(false, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Response::new(Empty {}))
    }

    type GetRecordedAudioStream = Pin<Box<dyn Stream<Item = Result<AudioChunk, Status>> + Send + 'static>>;

    async fn get_recorded_audio(&self, _request: Request<Empty>) -> Result<Response<Self::GetRecordedAudioStream>, Status> {
        let recording_path = PathBuf::from("/tmp/audio/recording.wav");
        
        if !recording_path.exists() {
            return Err(Status::not_found("No recording found"));
        }

        let data = fs::read(&recording_path).await
            .map_err(|e| Status::internal(format!("Failed to read recording: {}", e)))?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        
        tokio::spawn(async move {
            const CHUNK_SIZE: usize = 32 * 1024; // 32KB chunks
            
            for chunk in data.chunks(CHUNK_SIZE) {
                let audio_chunk = AudioChunk {
                    data: chunk.to_vec(),
                    format: "wav".to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                
                if tx.send(Ok(audio_chunk)).await.is_err() {
                    break;
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::GetRecordedAudioStream))
    }

    async fn enable_movement(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let servo = self.servo.lock().await;
        if self.calibration_running.load(Ordering::SeqCst) {
            return Err(Status::internal("Calibration is running, cannot enable movement"));
        }   
        servo.enable_movement()
            .map_err(|e| Status::internal(format!("Failed to enable movement: {}", e)))?;

        Ok(Response::new(Empty {}))
    }

    async fn disable_movement(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let servo = self.servo.lock().await;
        servo.disable_movement()
            .map_err(|e| Status::internal(format!("Failed to disable movement: {}", e)))?;

        Ok(Response::new(Empty {}))
    }

    async fn set_position(&self, request: Request<servo_control::JointPosition>) -> Result<Response<Empty>, Status> {
        let position = request.into_inner();
        let servo = self.servo.lock().await;
        
        // Convert degrees to raw value
        let raw_position = Servo::degrees_to_raw(position.position);
        
        // Convert speed to raw value (assuming speed is in degrees/second)
        let speed = (position.speed.abs() * 4096.0 / 360.0) as u16;

        if servo.get_servo_version(position.id as u8) == 10 {
            servo.move_servo(position.id as u8, raw_position as i16, 1000, speed)
                .map_err(|e| Status::internal(format!("Failed to set position: {}", e)))?;
        } else {
            servo.move_servo(position.id as u8, raw_position as i16, 0, speed)
                .map_err(|e| Status::internal(format!("Failed to set position: {}", e)))?;
        }

        Ok(Response::new(Empty {}))
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
