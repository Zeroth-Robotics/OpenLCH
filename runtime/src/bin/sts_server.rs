use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use serde_json;
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;

use runtime::hal::{Servo, ServoMode, ServoDirection, MemoryLockState, TorqueMode, ServoMultipleWriteCommand, ServoData, MAX_SERVOS};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", content = "params")]
enum Command {
    Move { id: u8, position: i16, time: u16, speed: u16 },
    SetMode { id: u8, mode: ServoMode },
    SetSpeed { id: u8, speed: u16, direction: ServoDirection },
    ReadInfo { id: u8 },
    ReadPID { id: u8 },
    SetPID { id: u8, p: u8, i: u8, d: u8 },
    SetMemoryLock { id: u8, state: MemoryLockState },
    ReadAngleLimits { id: u8 },
    SetTorqueMode { id: u8, mode: TorqueMode },
    Scan { id: u8 },
    WriteMultiple { cmd: ServoMultipleWriteCommand },
    ReadContinuous,
}

#[derive(Serialize, Deserialize)]
struct Response {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

async fn handle_client(servo: Arc<Mutex<Servo>>, stream: TcpStream, addr: SocketAddr) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            eprintln!("Error accepting WebSocket connection: {:?}", e);
            return;
        }
    };
    println!("New WebSocket connection: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        };

        if msg.is_text() || msg.is_binary() {
            let response = match serde_json::from_str::<Command>(msg.to_text().unwrap()) {
                Ok(cmd) => handle_command(&servo, cmd).await,
                Err(e) => Response {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid command format: {}", e)),
                },
            };
            let response_json = serde_json::to_string(&response).unwrap();
            if let Err(e) = ws_sender.send(response_json.into()).await {
                eprintln!("Error sending response: {:?}", e);
                break;
            }
        }
    }

    println!("WebSocket connection closed: {}", addr);
}

async fn handle_command(servo: &Arc<Mutex<Servo>>, cmd: Command) -> Response {
    let servo = servo.lock();
    match cmd {
        Command::Move { id, position, time, speed } => {
            match servo.move_servo(id, position, time, speed) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::SetMode { id, mode } => {
            match servo.set_mode(id, mode) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::SetSpeed { id, speed, direction } => {
            match servo.set_speed(id, speed, direction) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::ReadInfo { id } => {
            match servo.read_info(id) {
                Ok(info) => Response { success: true, data: Some(serde_json::to_value(info).unwrap()), error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::ReadPID { id } => {
            match servo.read_pid(id) {
                Ok(pid) => Response { success: true, data: Some(serde_json::to_value(pid).unwrap()), error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::SetPID { id, p, i, d } => {
            match servo.set_pid(id, p, i, d) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::SetMemoryLock { id, state } => {
            match servo.set_memory_lock(id, state) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::ReadAngleLimits { id } => {
            match servo.read_angle_limits(id) {
                Ok(limits) => Response { success: true, data: Some(serde_json::to_value(limits).unwrap()), error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::SetTorqueMode { id, mode } => {
            match servo.set_torque_mode(id, mode) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::Scan { id } => {
            match servo.scan(id) {
                Ok(found) => Response { success: true, data: Some(serde_json::to_value(found).unwrap()), error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::WriteMultiple { cmd } => {
            match servo.write_multiple(&cmd) {
                Ok(_) => Response { success: true, data: None, error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
        Command::ReadContinuous => {
            match servo.read_continuous() {
                Ok(data) => Response { success: true, data: Some(serde_json::to_value(data).unwrap()), error: None },
                Err(e) => Response { success: false, data: None, error: Some(e.to_string()) },
            }
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let servo = Arc::new(Mutex::new(Servo::new()?));

    while let Ok((stream, addr)) = listener.accept().await {
        let servo_clone = Arc::clone(&servo);
        tokio::spawn(async move {
            handle_client(servo_clone, stream, addr).await;
        });
    }

    Ok(())
}
