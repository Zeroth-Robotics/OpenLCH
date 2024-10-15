use crate::hal::{
    IMUData, MemoryLockState, ServoData, ServoDirection, ServoInfo, ServoMode,
    ServoMultipleWriteCommand, ServoRegister, TorqueMode, MAX_SERVOS,
};
use anyhow::{bail, Context, Result};
use serialport::SerialPort;
use std::env;
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// Constants
const SERVO_START_BYTE: u8 = 0xFF;
const SERVO_BROADCAST_ID: u8 = 0xFE;
const MAX_SERVO_COMMAND_DATA: usize = 256;

// Servo commands
const SERVO_CMD_PING: u8 = 0x01;
const SERVO_CMD_READ: u8 = 0x02;
const SERVO_CMD_WRITE: u8 = 0x03;
const SERVO_CMD_REG_WRITE: u8 = 0x04;
const SERVO_CMD_ACTION: u8 = 0x05;
const SERVO_CMD_SYNC_WRITE: u8 = 0x83;
const SERVO_CMD_RESET: u8 = 0x06;

// Memory addresses
const SERVO_ADDR_TORQUE_SWITCH: u8 = 0x28;
const SERVO_ADDR_TARGET_POSITION: u8 = 0x2A;
const SERVO_ADDR_CURRENT_POSITION: u8 = 0x38;
const SERVO_ADDR_CURRENT_LOAD: u8 = 0x3C;
const SERVO_ADDR_CURRENT_VOLTAGE: u8 = 0x3E;
const SERVO_ADDR_CURRENT_CURRENT: u8 = 0x45;

const TORQUE_OFF: u8 = 0;
const TORQUE_ON: u8 = 1;

#[derive(Debug, Clone)]
pub struct ServoCommand {
    pub id: u8,
    pub address: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

pub struct ServoSerial {
    port: Box<dyn SerialPort>,
}

impl ServoSerial {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()?;
        Ok(ServoSerial { port })
    }

    fn calculate_checksum(&self, packet: &[u8]) -> u8 {
        let sum: u16 = packet[2..packet.len() - 1].iter().map(|&x| x as u16).sum();
        !((sum & 0xFF) as u8)
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), std::io::Error> {
        self.port.write_all(packet)
    }

    fn receive_packet(&mut self, max_length: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut packet = Vec::with_capacity(max_length);
        let mut buffer = [0u8; 1];

        while packet.len() < max_length {
            self.port.read_exact(&mut buffer)?;
            packet.push(buffer[0]);

            if packet.len() >= 4 && packet.len() == packet[3] as usize + 4 {
                break;
            }
        }

        Ok(packet)
    }

    pub fn servo_ping(&mut self, id: u8) -> Result<(), std::io::Error> {
        let packet = [
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            id,
            2,
            SERVO_CMD_PING,
            0, // Checksum placeholder
        ];
        let mut packet = packet.to_vec();
        packet[5] = self.calculate_checksum(&packet);

        self.send_packet(&packet)?;

        let response = self.receive_packet(6)?;
        if response.len() != 6 || response[2] != id || response[4] != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response",
            ));
        }

        Ok(())
    }

    pub fn servo_read(
        &mut self,
        id: u8,
        address: u8,
        length: u8,
    ) -> Result<Vec<u8>, std::io::Error> {
        let packet = [
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            id,
            4,
            SERVO_CMD_READ,
            address,
            length,
            0, // Checksum placeholder
        ];
        let mut packet = packet.to_vec();
        packet[7] = self.calculate_checksum(&packet);

        self.send_packet(&packet)?;

        let response = self.receive_packet(256)?;
        if response.len() < 6 || response[2] != id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response",
            ));
        }

        Ok(response[5..response.len() - 1].to_vec())
    }

    pub fn servo_read_command(&mut self, cmd: &ServoCommand) -> Result<Vec<u8>, std::io::Error> {
        self.servo_read(cmd.id, cmd.address, cmd.length)
    }

    pub fn servo_write(&mut self, id: u8, address: u8, data: &[u8]) -> Result<(), std::io::Error> {
        let mut packet = vec![
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            id,
            (data.len() + 3) as u8,
            SERVO_CMD_WRITE,
            address,
        ];
        packet.extend_from_slice(data);
        packet.push(self.calculate_checksum(&packet));

        self.send_packet(&packet)?;

        if id != SERVO_BROADCAST_ID {
            let response = self.receive_packet(6)?;
            if response.len() != 6 || response[2] != id || response[4] != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid response",
                ));
            }
        }

        Ok(())
    }

    pub fn servo_write_command(&mut self, cmd: &ServoCommand) -> Result<(), std::io::Error> {
        self.servo_write(cmd.id, cmd.address, &cmd.data)
    }

    pub fn servo_reg_write(
        &mut self,
        id: u8,
        address: u8,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut packet = vec![
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            id,
            (data.len() + 3) as u8,
            SERVO_CMD_REG_WRITE,
            address,
        ];
        packet.extend_from_slice(data);
        packet.push(self.calculate_checksum(&packet));

        self.send_packet(&packet)?;

        if id != SERVO_BROADCAST_ID {
            let response = self.receive_packet(6)?;
            if response.len() != 6 || response[2] != id || response[4] != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid response",
                ));
            }
        }

        Ok(())
    }

    pub fn servo_action(&mut self) -> Result<(), std::io::Error> {
        let packet = [
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            SERVO_BROADCAST_ID,
            2,
            SERVO_CMD_ACTION,
            0, // Checksum placeholder
        ];
        let mut packet = packet.to_vec();
        packet[5] = self.calculate_checksum(&packet);

        self.send_packet(&packet)
    }

    pub fn servo_sync_write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let mut packet = vec![
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            SERVO_BROADCAST_ID,
            (data.len() + 2) as u8,
            SERVO_CMD_SYNC_WRITE,
        ];
        packet.extend_from_slice(data);
        packet.push(self.calculate_checksum(&packet));

        self.send_packet(&packet)
    }

    pub fn servo_reset(&mut self, id: u8) -> Result<(), std::io::Error> {
        let packet = [
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            id,
            2,
            SERVO_CMD_RESET,
            0, // Checksum placeholder
        ];
        let mut packet = packet.to_vec();
        packet[5] = self.calculate_checksum(&packet);

        self.send_packet(&packet)?;

        if id != SERVO_BROADCAST_ID {
            let response = self.receive_packet(6)?;
            if response.len() != 6 || response[2] != id || response[4] != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid response",
                ));
            }
        }

        Ok(())
    }

    pub fn servo_move(
        &mut self,
        id: u8,
        position: i16,
        time: u16,
        speed: u16,
    ) -> Result<(), std::io::Error> {
        let data = [
            (position & 0xFF) as u8,
            ((position >> 8) & 0xFF) as u8,
            (time & 0xFF) as u8,
            ((time >> 8) & 0xFF) as u8,
            (speed & 0xFF) as u8,
            ((speed >> 8) & 0xFF) as u8,
        ];
        self.servo_write(id, SERVO_ADDR_TARGET_POSITION, &data)
    }

    pub fn servo_move_multiple(
        &mut self,
        ids: &[u8],
        positions: &[i16],
    ) -> Result<(), std::io::Error> {
        if ids.len() != positions.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Mismatched ids and positions lengths",
            ));
        }

        let mut data = Vec::with_capacity(1 + 1 + ids.len() * 3);
        data.push(SERVO_ADDR_TARGET_POSITION);
        data.push(2); // 2 bytes for position data

        for (&id, &position) in ids.iter().zip(positions.iter()) {
            data.push(id);
            data.push((position & 0xFF) as u8);
            data.push(((position >> 8) & 0xFF) as u8);
        }

        self.servo_sync_write(&data)
    }

    pub fn servo_move_multiple_sync(
        &mut self,
        cmd: &ServoMultipleWriteCommand,
    ) -> Result<(), std::io::Error> {
        if cmd.ids.len() != cmd.positions.len()
            || cmd.ids.len() != cmd.times.len()
            || cmd.ids.len() != cmd.speeds.len()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Mismatched input lengths",
            ));
        }

        let count = cmd.ids.len();
        if count == 0 || count > MAX_SERVOS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid count",
            ));
        }

        let mut packet = Vec::with_capacity(256);
        packet.extend_from_slice(&[
            SERVO_START_BYTE,
            SERVO_START_BYTE,
            SERVO_BROADCAST_ID,
            ((cmd.only_write_positions as u8 * 3 + 7) * count as u8 + 4),
            SERVO_CMD_SYNC_WRITE,
            SERVO_ADDR_TARGET_POSITION,
            if cmd.only_write_positions == 1 { 2 } else { 6 }, // Data length per servo
        ]);

        for i in 0..count {
            packet.push(cmd.ids[i]);
            packet.extend_from_slice(&cmd.positions[i].to_le_bytes());
            if cmd.only_write_positions == 0 {
                packet.extend_from_slice(&cmd.times[i].to_le_bytes());
                packet.extend_from_slice(&cmd.speeds[i].to_le_bytes());
            }
        }

        packet.push(self.calculate_checksum(&packet));

        self.send_packet(&packet)
    }

    pub fn servo_read_position(&mut self, id: u8) -> Result<i16, std::io::Error> {
        let data = self.servo_read(id, SERVO_ADDR_CURRENT_POSITION, 2)?;
        if data.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response length",
            ));
        }
        Ok(i16::from_le_bytes([data[0], data[1]]))
    }

    pub fn servo_read_current(&mut self, id: u8) -> Result<u16, std::io::Error> {
        let data = self.servo_read(id, SERVO_ADDR_CURRENT_CURRENT, 2)?;
        if data.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response length",
            ));
        }
        Ok(u16::from_le_bytes([data[0], data[1]]))
    }

    pub fn servo_read_load(&mut self, id: u8) -> Result<i16, std::io::Error> {
        let data = self.servo_read(id, SERVO_ADDR_CURRENT_LOAD, 2)?;
        if data.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response length",
            ));
        }
        Ok(i16::from_le_bytes([data[0], data[1]]))
    }

    pub fn servo_read_voltage(&mut self, id: u8) -> Result<u8, std::io::Error> {
        let data = self.servo_read(id, SERVO_ADDR_CURRENT_VOLTAGE, 1)?;
        if data.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response length",
            ));
        }
        Ok(data[0])
    }

    pub fn servo_read_position_and_status(
        &mut self,
        id: u8,
    ) -> Result<(i16, i16, i16), std::io::Error> {
        let data = self.servo_read(id, SERVO_ADDR_CURRENT_POSITION, 6)?;
        if data.len() != 6 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response length",
            ));
        }

        let current_location = i16::from_le_bytes([data[0], data[1]]);
        let current_speed = i16::from_le_bytes([data[2], data[3]]);
        let current_load = i16::from_le_bytes([data[4], data[5]]);

        Ok((current_location, current_speed, current_load))
    }

    pub fn servo_set_torque(&mut self, id: u8, torque_state: u8) -> Result<(), std::io::Error> {
        self.servo_write(id, SERVO_ADDR_TORQUE_SWITCH, &[torque_state])
    }

    pub fn servo_torque_on(&mut self, id: u8) -> Result<(), std::io::Error> {
        self.servo_set_torque(id, TORQUE_ON)
    }

    pub fn servo_torque_off(&mut self, id: u8) -> Result<(), std::io::Error> {
        self.servo_set_torque(id, TORQUE_OFF)
    }
}

pub struct Servo {
    serial: Arc<Mutex<ServoSerial>>,
}

impl Servo {
    pub fn new() -> Result<Self> {
        let port_name = env::var("SERVO_PORT").unwrap_or_else(|_| "/dev/ttyUSB0".to_string());
        let baud_rate = env::var("SERVO_BAUD_RATE")
            .unwrap_or_else(|_| "115200".to_string())
            .parse::<u32>()
            .context("Failed to parse SERVO_BAUD_RATE")?;

        let serial = ServoSerial::new(&port_name, baud_rate)
            .map_err(|e| anyhow::anyhow!("Failed to create ServoSerial: {}", e))?;

        Ok(Servo {
            serial: Arc::new(Mutex::new(serial)),
        })
    }

    pub fn write(&self, id: u8, register: ServoRegister, data: &[u8]) -> Result<()> {
        let mut serial = self.serial.lock().unwrap();
        match serial.servo_write(id, register as u8, data) {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // Silently ignore errors
        }
    }

    pub fn read(&self, id: u8, register: ServoRegister, length: u8) -> Result<Vec<u8>> {
        let mut serial = self.serial.lock().unwrap();
        match serial.servo_read(id, register as u8, length) {
            Ok(data) => Ok(data),
            Err(_) => Ok(Vec::new()), // Return empty Vec on error
        }
    }

    pub fn move_servo(&self, id: u8, position: i16, time: u16, speed: u16) -> Result<()> {
        let mut serial = self.serial.lock().unwrap();
        serial
            .servo_move(id, position, time, speed)
            .map_err(|e| anyhow::anyhow!("Failed to move servo: {}", e))
    }

    pub fn set_mode(&self, id: u8, mode: ServoMode) -> Result<()> {
        self.write(id, ServoRegister::OperationMode, &[mode as u8])
    }

    pub fn set_speed(&self, id: u8, speed: u16, direction: ServoDirection) -> Result<()> {
        let speed = if direction == ServoDirection::Clockwise {
            speed
        } else {
            speed | 0x8000
        };
        self.write(id, ServoRegister::RunningSpeed, &speed.to_le_bytes())
    }

    pub fn read_info(&self, id: u8) -> Result<ServoInfo> {
        let mut serial = self.serial.lock().unwrap();
        let data = serial.servo_read(id, ServoRegister::TorqueSwitch as u8, 30)?;

        if data.len() != 30 {
            bail!("Failed to read servo info: incorrect data length");
        }

        Ok(ServoInfo {
            torque_switch: data[0],
            acceleration: data[1],
            target_location: i16::from_le_bytes([data[2], data[3]]),
            running_time: u16::from_le_bytes([data[4], data[5]]),
            running_speed: u16::from_le_bytes([data[6], data[7]]),
            torque_limit: u16::from_le_bytes([data[8], data[9]]),
            reserved1: [data[10], data[11], data[12], data[13], data[14], data[15]],
            lock_mark: data[15],
            current_location: i16::from_le_bytes([data[16], data[17]]),
            current_speed: i16::from_le_bytes([data[18], data[19]]),
            current_load: i16::from_le_bytes([data[20], data[21]]),
            current_voltage: data[22],
            current_temperature: data[23],
            async_write_flag: data[24],
            servo_status: data[25],
            mobile_sign: data[26],
            reserved2: [data[27], data[28]],
            current_current: u16::from_le_bytes([data[28], data[29]]),
        })
    }

    pub fn read_continuous(&self) -> Result<ServoData> {
        let mut data = ServoData {
            servo: [ServoInfo::default(); MAX_SERVOS],
            task_run_count: 0,
        };

        for i in 0..MAX_SERVOS {
            if let Ok(info) = self.read_info(i as u8) {
                data.servo[i] = info;
            }
        }

        data.task_run_count += 1;
        Ok(data)
    }

    pub fn write_multiple(&self, cmd: &ServoMultipleWriteCommand) -> Result<()> {
        let mut serial = self.serial.lock().unwrap();
        let adapted_cmd = ServoMultipleWriteCommand {
            only_write_positions: cmd.only_write_positions,
            ids: cmd.ids,
            positions: cmd.positions,
            times: cmd.times,
            speeds: cmd.speeds,
        };
        serial
            .servo_move_multiple_sync(&adapted_cmd)
            .map_err(|e| anyhow::anyhow!("Failed to write multiple servo positions: {}", e))
    }

    pub fn read_pid(&self, id: u8) -> Result<(u8, u8, u8)> {
        let p = self.read(id, ServoRegister::PProportionalCoeff, 1)?[0];
        let i = self.read(id, ServoRegister::IIntegralCoeff, 1)?[0];
        let d = self.read(id, ServoRegister::DDifferentialCoeff, 1)?[0];
        Ok((p, i, d))
    }

    pub fn set_pid(&self, id: u8, p: u8, i: u8, d: u8) -> Result<()> {
        // Unlock flash
        self.write(
            id,
            ServoRegister::LockMark,
            &[MemoryLockState::Unlocked as u8],
        )?;

        // Set PID parameters
        self.write(id, ServoRegister::PProportionalCoeff, &[p])?;
        self.write(id, ServoRegister::IIntegralCoeff, &[i])?;
        self.write(id, ServoRegister::DDifferentialCoeff, &[d])?;

        // Lock flash
        self.write(
            id,
            ServoRegister::LockMark,
            &[MemoryLockState::Locked as u8],
        )?;

        Ok(())
    }

    pub fn set_memory_lock(&self, id: u8, state: MemoryLockState) -> Result<()> {
        self.write(id, ServoRegister::LockMark, &[state as u8])
    }

    pub fn read_angle_limits(&self, id: u8) -> Result<(i16, i16)> {
        let min_limit = i16::from_le_bytes(
            self.read(id, ServoRegister::MinAngleLimit, 2)?
                .try_into()
                .unwrap(),
        );
        let max_limit = i16::from_le_bytes(
            self.read(id, ServoRegister::MaxAngleLimit, 2)?
                .try_into()
                .unwrap(),
        );
        Ok((min_limit, max_limit))
    }

    pub fn set_torque_mode(&self, id: u8, mode: TorqueMode) -> Result<()> {
        self.write(id, ServoRegister::TorqueSwitch, &[mode as u8])
    }

    pub fn write_servo_memory(&self, id: u8, register: ServoRegister, value: u16) -> Result<()> {
        let data = value.to_le_bytes();
        self.write(id, register, &data)
    }

    pub fn scan(&self, id: u8) -> Result<bool> {
        match self.read(id, ServoRegister::ID, 1) {
            Ok(data) if data.len() == 1 && data[0] == id => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub fn enable_readout(&self) -> Result<()> {
        Ok(())
    }

    pub fn disable_readout(&self) -> Result<()> {
        Ok(())
    }
}

impl Drop for Servo {
    fn drop(&mut self) {
        // No need for explicit deinitialization in this implementation
    }
}

pub struct IMU {}

impl IMU {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(IMU {})
    }

    pub fn read_data(&mut self) -> Result<IMUData, Box<dyn std::error::Error>> {
        Ok(IMUData {
            acc_x: 0.0,
            acc_y: 0.0,
            acc_z: 0.0,
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
        })
    }
}
