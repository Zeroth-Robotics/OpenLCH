# Runtime Documentation

## Local builds

### Prerequisites
- docker installed
- brew installed

```bash
brew install gitlab-ci-local
gitlab-ci-local --stage build-toolchain
```

### Build locally

```bash
gitlab-ci-local --stage build-runtime
```

```bash
cargo run
```

## Architecture

<img src="public/runtime.png" alt="Runtime Architecture">

The goal of the runtime is to provide a unified interface for:

- robot configuration and declaration
- control loop execution
- state management
- model inference
- error handling and safety

## `config/[robot-name].toml`

Define the robot: joints, servos, parameters, etc.

Based on robot from `kscalelabs/firmware` config.

## `robot.rs`

Create the robot struct based on the config.toml file with all the servos and joints.

Provides the states for the robot.

Based on robot from `kscalelabs/firmware` Robot struct.

## `HAL.rs`

Define the hardware abstraction layer for the servos controls from firmware.

Rust binding for C++ struct (WIP):

```bash
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
```

## `controller.rs`

`StandingControllerPID` -> controller for standing using pre-set positions and PID controller

`StandingControllerPPO` -> controller for standing using PPO model

## `model.rs`

onnx inference session and initalization.

## `main.rs`

initialize config, robot, controller and start standing using controller.
Building for target platform (Milk-V Duo S)

`docker run --rm -v $(pwd):/workspace -w /workspace openlch-runtime-sdk /root/.cargo/bin/cargo +nightly build --target riscv64gc-unknown-linux-musl -Zbuild-std --release`

Uploading to the board (ethernet over usb c)
`scp -O target/riscv64gc-unknown-linux-musl/release/runtime root@192.168.42.1:`

Remember to first build the docker container with sdk/toolchain

```
cd ../runtime-sdk
docker build . -t openlch-runtime-sdk
```

