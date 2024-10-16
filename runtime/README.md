# Runtime Documentation

## Setup

1. Flash the Image to the SD Card
   - Download the OpenLCH image from the [releases page](https://github.com/Zeroth-Robotics/OpenLCH-buildroot/releases).
   - Use Balena Etcher (or any other flashing tool) to flash the image onto your SD card.
   - Once flashing is complete, remove and reinsert the SD card into your computer.

2. Configure Wi-Fi on Startup
   - Open the boot folder on the SD card (it should be visible on macOS since it is formatted as FAT32).
   - Create a new file named `wpa_supplicant.conf` in the boot folder and add the following content:

     ```bash
     ctrl_interface=/var/run/wpa_supplicant
     ap_scan=1
     update_config=1

     network={
       ssid="YOURWIFINAME"
       psk="YOURPASSWORD"
       key_mgmt=WPA-PSK
     }
     ```
   - Replace `YOURWIFINAME` and `YOURPASSWORD` with your Wi-Fi credentials.

3. Connect via USB
   - If you're connecting via USB, use the following details:
     - IP Address: `192.168.42.1`
     - Username: `root`
     - Password: `milkv`
     - SSH: `ssh root@192.168.42.1`

4. Download Required Artifacts
   - Go to OpenLCH Artifacts and download the following:
     - `runtime`
     - `servo`
     - `cviwrapper`

5. Transfer Files to the Target Device
   - Run the following commands to copy the necessary files to your device:
     ```bash
     scp -O runtime root@192.168.42.1:/usr/local/bin/
     scp -O servo root@192.168.42.1:/usr/local/bin/
     scp -O cviwrapper root@192.168.42.1:/usr/local/bin/
     ```

6. Run the Application
   - To run the servo setup, execute the following on the target device:
     ```bash
     # List available binaries
     ls /usr/local/bin/
     # Run your desired binary (example: runtime)
     sudo /usr/local/bin/runtime
     ```

7. Install Docker / Docker Desktop
   ```bash
   brew install gitlab-ci-local
   ```

## Build Runtime

```bash
cargo check && gitlab-ci-local --stage build-runtime  # check and build
```

Find binaries in `target/riscv64gc-unknown-linux-musl/release/`

### Copy to the milk-v board

```bash
scp -O target/riscv64gc-unknown-linux-musl/release/runtime $MILKV_IP:/usr/local/bin/
```

If the board is connected over USB, IP is `192.168.42.1`. Try `ping 192.168.42.1` to verify that the board is reachable.

## Architecture

<img src="public/runtime.png" alt="Runtime Architecture">

The goal of the runtime is to provide a unified interface for:

- Robot configuration and declaration
- Control loop execution
- State management
- Model inference
- Error handling and safety

### `config/[robot-name].toml`

Define the robot: joints, servos, parameters, etc.

Based on robot from `kscalelabs/firmware` config.

### `robot.rs`

Create the robot struct based on the config.toml file with all the servos and joints.

Provides the states for the robot.

Based on robot from `kscalelabs/firmware` Robot struct.

### `hal.rs`

Servo control code

### `controller.rs`

- `StandingControllerPID` -> controller for standing using pre-set positions and PID controller
- `StandingControllerPPO` -> controller for standing using PPO model

### `model.rs`

ONNX inference session and initialization.

### `main.rs`

Initialize config, robot, controller and start standing using controller.

## Hardware

### Servo ID

![Servo ID](https://github.com/user-attachments/assets/93db0404-bf76-4b4f-9201-665cc868364b)

### Wiring

![Wiring](https://github.com/user-attachments/assets/d1c02231-ae6e-4333-bf7e-74db47416b88)
