## Documentation

## `HAL.rs`

Define the hardware abstraction layer for the servos controls from firmware.

Rust binding for C++ struct (WIP):
//     uint8_t torque_switch;         // 0x28 (1 byte)
//     uint8_t acceleration;          // 0x29 (1 byte)
//     int16_t target_location;       // 0x2A (2 bytes)
//     uint16_t running_time;         // 0x2C (2 bytes)
//     uint16_t running_speed;        // 0x2E (2 bytes)
//     uint16_t torque_limit;         // 0x30 (2 bytes)
//     uint8_t reserved1[6];          // 0x32-0x37 (6 bytes, reserved)
//     uint8_t lock_mark;             // 0x37 (1 byte)
//     int16_t current_location;      // 0x38 (2 bytes)
//     int16_t current_speed;         // 0x3A (2 bytes)
//     int16_t current_load;          // 0x3C (2 bytes)
//     uint8_t current_voltage;       // 0x3E (1 byte)
//     uint8_t current_temperature;   // 0x3F (1 byte)
//     uint8_t async_write_flag;      // 0x40 (1 byte)
//     uint8_t servo_status;          // 0x41 (1 byte)
//     uint8_t mobile_sign;           // 0x42 (1 byte)
//     uint8_t reserved2[2];          // 0x43-0x44 (2 bytes, reserved)
//     uint16_t current_current;      // 0x45 (2 bytes)



## `config.toml`
Define joint limits, motor IDs, PID gains, etc.

Based on robot from kscalelabs/firmware config.


## `robot.rs`
Defines robot struct and methods to get joint states and imu states.

Based on robot from kscalelabs/firmware Robot struct


## `controller.rs`
`StandingControllerPID` -> controller for standing using pre-set positions and PID controller

`StandingControllerPPO` -> controller for standing using PPO model 

## `model.rs`
onnx inference session and initalization.

## `main.rs`
initialize config, robot, controller and start standing using controller.















