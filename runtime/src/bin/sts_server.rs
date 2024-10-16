use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod servo_control {
    tonic::include_proto!("hal_pb");
}

use servo_control::servo_control_server::{ServoControl, ServoControlServer};
use servo_control::{Empty, JointPositions, WifiCredentials, ServoId, ServoInfo, ServoIds, IdChange, ChangeIdResponse, ServoInfoResponse, servo_info_response, change_id_response};
use runtime::hal::{Servo, MAX_SERVOS, ServoMultipleWriteCommand, ServoData};

#[derive(Debug)]
pub struct StsServoControl {
    servo: Arc<Mutex<Servo>>,
    last_positions: Arc<Mutex<ServoData>>,
}

impl StsServoControl {
    pub fn new() -> Result<Self> {
        let servo = Servo::new()?;
        servo.enable_readout()?;
        let initial_data = servo.read_continuous()?;
        Ok(Self {
            servo: Arc::new(Mutex::new(servo)),
            last_positions: Arc::new(Mutex::new(initial_data)),
        })
    }
}

#[tonic::async_trait]
impl ServoControl for StsServoControl {
    async fn get_positions(&self, _request: Request<Empty>) -> Result<Response<JointPositions>, Status> {
        let servo = self.servo.lock().await;
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
        let last_positions = self.last_positions.lock().await;
        
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
        
        let info = ServoInfo {
            id: id as i32,
            temperature: servo_info.current_temperature as f32,
            current: servo_info.current_current as f32,
            voltage: ((servo_info.current_voltage as f32 / 10.0) * 10.0).round() / 10.0,
            speed: servo_info.current_speed as f32 / 4096.0 * 360.0,
            current_position: Servo::raw_to_degrees(servo_info.current_location as u16),
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let servo_control = StsServoControl::new()?;

    Server::builder()
        .add_service(ServoControlServer::new(servo_control))
        .serve(addr)
        .await?;

    Ok(())
}
