use anyhow::Result;
use std::os::raw::{c_int, c_short, c_uchar, c_ushort, c_uint};

pub const MAX_SERVOS: usize = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ServoInfo {
    pub torque_switch: c_uchar,
    pub acceleration: c_uchar,
    pub target_location: c_short,
    pub running_time: c_ushort,
    pub running_speed: c_ushort,
    pub torque_limit: c_ushort,
    pub reserved1: [c_uchar; 6],
    pub lock_mark: c_uchar,
    pub current_location: c_short,
    pub current_speed: c_short,
    pub current_load: c_short,
    pub current_voltage: c_uchar,
    pub current_temperature: c_uchar,
    pub async_write_flag: c_uchar,
    pub servo_status: c_uchar,
    pub mobile_sign: c_uchar,
    pub reserved2: [c_uchar; 2],
    pub current_current: c_ushort,
}

#[repr(u8)]
pub enum ServoRegister {
    FirmwareMajorVersion = 0x00,
    FirmwareSubVersion = 0x01,
    ServoMainVersion = 0x03,
    ServoSubVersion = 0x04,
    ID = 0x05,
    BaudRate = 0x06,
    ReturnDelay = 0x07,
    ResponseStatusLevel = 0x08,
    MinAngleLimit = 0x09,
    MaxAngleLimit = 0x0B,
    MaxTemperatureLimit = 0x0D,
    MaxInputVoltage = 0x0E,
    MinInputVoltage = 0x0F,
    MaxTorque = 0x10,
    Phase = 0x12,
    UnloadingCondition = 0x13,
    LEDAlarmCondition = 0x14,
    PProportionalCoeff = 0x15,
    DDifferentialCoeff = 0x16,
    IIntegralCoeff = 0x17,
    MinStartupForce = 0x18,
    ClockwiseInsensitiveArea = 0x1A,
    CounterclockwiseInsensitiveArea = 0x1B,
    ProtectionCurrent = 0x1C,
    AngularResolution = 0x1E,
    PositionCorrection = 0x1F,
    OperationMode = 0x21,
    ProtectiveTorque = 0x22,
    ProtectionTime = 0x23,
    OverloadTorque = 0x24,
    SpeedClosedLoopPCoeff = 0x25,
    OverCurrentProtectionTime = 0x26,
    VelocityClosedLoopICoeff = 0x27,
    TorqueSwitch = 0x28,
    Acceleration = 0x29,
    TargetLocation = 0x2A,
    RunningTime = 0x2C,
    RunningSpeed = 0x2E,
    TorqueLimit = 0x30,
    LockMark = 0x37,
    CurrentLocation = 0x38,
    CurrentSpeed = 0x3A,
    CurrentLoad = 0x3C,
    CurrentVoltage = 0x3E,
    CurrentTemperature = 0x3F,
    AsyncWriteFlag = 0x40,
    ServoStatus = 0x41,
    MobileSign = 0x42,
    CurrentCurrent = 0x45,
}

#[repr(u8)]
pub enum MemoryLockState {
    Unlocked = 0,
    Locked = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ServoData {
    pub servo: [ServoInfo; MAX_SERVOS],
    pub task_run_count: c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ServoMultipleWriteCommand {
    pub only_write_positions: c_uchar,
    pub ids: [c_uchar; MAX_SERVOS],
    pub positions: [c_short; MAX_SERVOS],
    pub times: [c_ushort; MAX_SERVOS],
    pub speeds: [c_ushort; MAX_SERVOS],
}

#[link(name = "sts3215")]
extern "C" {
    fn servo_init() -> c_int;
    fn servo_deinit();
    fn servo_write(id: c_uchar, address: c_uchar, data: *const c_uchar, length: c_uchar) -> c_int;
    fn servo_read(id: c_uchar, address: c_uchar, length: c_uchar, data: *mut c_uchar) -> c_int;
    fn servo_move(id: c_uchar, position: c_short, time: c_ushort, speed: c_ushort) -> c_int;
    fn enable_servo_readout() -> c_int;
    fn disable_servo_readout() -> c_int;
    fn set_servo_mode(id: c_uchar, mode: c_uchar) -> c_int;
    fn set_servo_speed(id: c_uchar, speed: c_ushort, direction: c_int) -> c_int;
    fn servo_read_info(id: c_uchar, info: *mut ServoInfo) -> c_int;
    fn read_servo_positions(servo_data: *mut ServoData) -> c_int;
    fn servo_write_multiple(cmd: *const ServoMultipleWriteCommand) -> c_int;
}

pub enum ServoMode {
    Position = 0,
    ConstantSpeed = 1,
    PWMOpenLoop = 2,
    StepServo = 3,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum ServoDirection {
    Clockwise = 0,
    Counterclockwise = 1,
}

impl PartialEq for ServoDirection {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

pub struct Servo {
    _private: (), // Prevent direct construction
}

impl Servo {
    pub fn new() -> Result<Self> {
        let result = unsafe { servo_init() };
        if result != 0 {
            anyhow::bail!("Failed to initialize servo");
        }
        Ok(Servo { _private: () })
    }

    pub fn write(&self, id: u8, register: ServoRegister, data: &[u8]) -> Result<()> {
        let result = unsafe { servo_write(id, register as u8, data.as_ptr(), data.len() as c_uchar) };
        if result != 0 {
            anyhow::bail!("Failed to write to servo");
        }
        Ok(())
    }

    pub fn read(&self, id: u8, register: ServoRegister, length: u8) -> Result<Vec<u8>> {
        let mut data = vec![0u8; length as usize];
        let result = unsafe { servo_read(id, register as u8, length, data.as_mut_ptr()) };
        if result != 0 {
            anyhow::bail!("Failed to read from servo");
        }
        Ok(data)
    }

    pub fn move_servo(&self, id: u8, position: i16, time: u16, speed: u16) -> Result<()> {
        let result = unsafe { servo_move(id, position, time, speed) };
        if result != 0 {
            anyhow::bail!("Failed to move servo");
        }
        Ok(())
    }

    pub fn enable_readout(&self) -> Result<()> {
        let result = unsafe { enable_servo_readout() };
        if result != 0 {
            anyhow::bail!("Failed to enable servo readout");
        }
        Ok(())
    }

    pub fn disable_readout(&self) -> Result<()> {
        let result = unsafe { disable_servo_readout() };
        if result != 0 {
            anyhow::bail!("Failed to disable servo readout");
        }
        Ok(())
    }

    pub fn set_mode(&self, id: u8, mode: ServoMode) -> Result<()> {
        let result = unsafe { set_servo_mode(id, mode as u8) };
        if result != 0 {
            anyhow::bail!("Failed to set servo mode");
        }
        Ok(())
    }

    pub fn set_speed(&self, id: u8, speed: u16, direction: ServoDirection) -> Result<()> {
        let result = unsafe { set_servo_speed(id, speed, direction as i32) };
        if result != 0 {
            anyhow::bail!("Failed to set servo speed");
        }
        Ok(())
    }

    pub fn read_info(&self, id: u8) -> Result<ServoInfo> {
        let mut info = ServoInfo {
            torque_switch: 0,
            acceleration: 0,
            target_location: 0,
            running_time: 0,
            running_speed: 0,
            torque_limit: 0,
            reserved1: [0; 6],
            lock_mark: 0,
            current_location: 0,
            current_speed: 0,
            current_load: 0,
            current_voltage: 0,
            current_temperature: 0,
            async_write_flag: 0,
            servo_status: 0,
            mobile_sign: 0,
            reserved2: [0; 2],
            current_current: 0,
        };
        let result = unsafe { servo_read_info(id, &mut info) };
        if result != 0 {
            anyhow::bail!("Failed to read servo info");
        }
        Ok(info)
    }

    pub fn read_continuous(&self) -> Result<ServoData> {
        let mut data = ServoData {
            servo: [ServoInfo {
                torque_switch: 0,
                acceleration: 0,
                target_location: 0,
                running_time: 0,
                running_speed: 0,
                torque_limit: 0,
                reserved1: [0; 6],
                lock_mark: 0,
                current_location: 0,
                current_speed: 0,
                current_load: 0,
                current_voltage: 0,
                current_temperature: 0,
                async_write_flag: 0,
                servo_status: 0,
                mobile_sign: 0,
                reserved2: [0; 2],
                current_current: 0,
            }; MAX_SERVOS],
            task_run_count: 0,
        };
        let result = unsafe { read_servo_positions(&mut data) };
        if result != 0 {
            anyhow::bail!("Failed to read continuous servo data");
        }
        Ok(data)
    }

    pub fn write_multiple(&self, cmd: &ServoMultipleWriteCommand) -> Result<()> {
        let result = unsafe { servo_write_multiple(cmd) };
        if result != 0 {
            anyhow::bail!("Failed to write multiple servo positions");
        }
        Ok(())
    }

    pub fn read_pid(&self, id: u8) -> Result<(u8, u8, u8)> {
        let p = self.read(id, ServoRegister::PProportionalCoeff, 1)?[0];
        let i = self.read(id, ServoRegister::IIntegralCoeff, 1)?[0];
        let d = self.read(id, ServoRegister::DDifferentialCoeff, 1)?[0];
        Ok((p, i, d))
    }

    pub fn set_pid(&self, id: u8, p: u8, i: u8, d: u8) -> Result<()> {
        // Unlock flash
        self.write(id, ServoRegister::LockMark, &[MemoryLockState::Unlocked as u8])?;

        // Set PID parameters
        self.write(id, ServoRegister::PProportionalCoeff, &[p])?;
        self.write(id, ServoRegister::IIntegralCoeff, &[i])?;
        self.write(id, ServoRegister::DDifferentialCoeff, &[d])?;

        // Lock flash
        self.write(id, ServoRegister::LockMark, &[MemoryLockState::Locked as u8])?;

        Ok(())
    }

    pub fn set_memory_lock(&self, id: u8, state: MemoryLockState) -> Result<()> {
        self.write(id, ServoRegister::LockMark, &[state as u8])
    }

    pub fn read_angle_limits(&self, id: u8) -> Result<(i16, i16)> {
        let min_limit = i16::from_le_bytes(self.read(id, ServoRegister::MinAngleLimit, 2)?.try_into().unwrap());
        let max_limit = i16::from_le_bytes(self.read(id, ServoRegister::MaxAngleLimit, 2)?.try_into().unwrap());
        Ok((min_limit, max_limit))
    }
}

impl Drop for Servo {
    fn drop(&mut self) {
        unsafe { servo_deinit() };
    }
}