# Runtime Documentation

## Setup

### Prerequisites
```
# Install docker / docker desktop
brew install gitlab-ci-local
```

### Build

```bash
gitlab-ci-local --stage build-runtime
```

Find binaries in target/riscv64gc-unknown-linux-musl/release/

### Copy to the milk-v board
`scp -O target/riscv64gc-unknown-linux-musl/release/runtime $MILKV_IP:/usr/local/bin/`

If the board is connected over usb, ip is `192.168.42.1`

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

## `hal.rs`

Servo control code

## `controller.rs`

`StandingControllerPID` -> controller for standing using pre-set positions and PID controller

`StandingControllerPPO` -> controller for standing using PPO model

## `model.rs`

onnx inference session and initalization.

## `main.rs`

initialize config, robot, controller and start standing using controller.
