import openlch
import numpy as np
from ahrs.filters import Madgwick
from ahrs.common import orientation
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import time
import argparse


def init_madgwick():
    # Increase beta to give more weight to accelerometer data
    # Default is usually around 0.1
    return Madgwick(frequency=100.0, beta=0.7)


def imu_data_to_numpy(imu_data):
    gyro = imu_data["gyro"]
    accel = imu_data["accel"]
    # Convert to proper units: gyro to rad/s, accel to g
    gyro_array = (
        np.array([gyro["x"], gyro["y"], gyro["z"]]) * np.pi / 180.0
    )  # deg/s to rad/s
    accel_array = np.array([accel["x"], accel["y"], accel["z"]]) / 1000.0  # mg to g
    return gyro_array, accel_array


class EulerPlotter:
    def __init__(self):
        # Create figure with three subplots side by side
        self.fig, (self.ax1, self.ax2, self.ax3) = plt.subplots(1, 3, figsize=(20, 5))

        # Euler angles plot (left)
        self.euler_lines = [
            self.ax1.plot([], [], label=angle)[0] for angle in ["Roll", "Pitch", "Yaw"]
        ]
        self.ax1.legend()
        self.ax1.set_ylim(-180, 180)
        self.ax1.set_xlabel("Samples")
        self.ax1.set_ylabel("Angle (degrees)")
        self.ax1.set_title("Euler Angles")

        # Raw data plot (middle)
        self.raw_lines_gyro = [
            self.ax2.plot([], [], label=f"Gyro {axis}")[0] for axis in ["X", "Y", "Z"]
        ]
        self.raw_lines_accel = [
            self.ax2.plot([], [], "--", label=f"Accel {axis}")[0]
            for axis in ["X", "Y", "Z"]
        ]
        self.ax2.legend()
        self.ax2.set_xlabel("Samples")
        self.ax2.set_ylabel("Value")
        self.ax2.set_title("Raw IMU Data")

        # Quaternion plot (right)
        self.quat_lines = [self.ax3.plot([], [], label=f"q{i}")[0] for i in range(4)]
        self.ax3.legend()
        self.ax3.set_ylim(-1.1, 1.1)
        self.ax3.set_xlabel("Samples")
        self.ax3.set_ylabel("Value")
        self.ax3.set_title("Quaternions")

        # Data storage
        self.euler_data = [[], [], []]
        self.gyro_data = [[], [], []]
        self.accel_data = [[], [], []]
        self.quat_data = [[], [], [], []]
        self.max_points = 100

    def update(self, euler_angles, gyro, accel, quaternions):
        # Update Euler angles
        for i, line in enumerate(self.euler_lines):
            self.euler_data[i].append(euler_angles[i])
            if len(self.euler_data[i]) > self.max_points:
                self.euler_data[i].pop(0)
            line.set_data(range(len(self.euler_data[i])), self.euler_data[i])

        # Update raw data
        for i, line in enumerate(self.raw_lines_gyro):
            self.gyro_data[i].append(gyro[i])
            if len(self.gyro_data[i]) > self.max_points:
                self.gyro_data[i].pop(0)
            line.set_data(range(len(self.gyro_data[i])), self.gyro_data[i])

        for i, line in enumerate(self.raw_lines_accel):
            self.accel_data[i].append(accel[i])
            if len(self.accel_data[i]) > self.max_points:
                self.accel_data[i].pop(0)
            line.set_data(range(len(self.accel_data[i])), self.accel_data[i])

        # Update quaternion data
        for i, line in enumerate(self.quat_lines):
            self.quat_data[i].append(quaternions[i])
            if len(self.quat_data[i]) > self.max_points:
                self.quat_data[i].pop(0)
            line.set_data(range(len(self.quat_data[i])), self.quat_data[i])

        # Update plot limits
        self.ax1.set_xlim(0, self.max_points)
        self.ax2.set_xlim(0, self.max_points)
        self.ax3.set_xlim(0, self.max_points)
        self.ax2.relim()
        self.ax2.autoscale_view()

        return (
            self.euler_lines
            + self.raw_lines_gyro
            + self.raw_lines_accel
            + self.quat_lines
        )


class AnimationState:
    def __init__(self):
        self.Q = np.array([1.0, 0.0, 0.0, 0.0])  # Initial quaternion
        self.gyro_bias = np.zeros(3)  # For storing gyro bias

    def calibrate_gyro(self, imu, samples=100):
        print("Calibrating gyro, keep IMU still...")
        # Collect samples while IMU is stationary
        bias_sum = np.zeros(3)
        for _ in range(samples):
            imu_data = imu.get_data()
            gyro, _ = imu_data_to_numpy(imu_data)
            bias_sum += gyro
            time.sleep(0.02)
        self.gyro_bias = bias_sum / samples
        print(
            f"Gyro bias: X={self.gyro_bias[0]:.3f}, Y={self.gyro_bias[1]:.3f}, Z={self.gyro_bias[2]:.3f} rad/s"
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--no-calibration", action="store_true", help="Disable gyro bias calibration"
    )
    args = parser.parse_args()

    hal = openlch.HAL()
    imu = hal.imu
    madgwick = init_madgwick()
    plotter = EulerPlotter()

    state = AnimationState()
    if not args.no_calibration:
        state.calibrate_gyro(imu)  # Only calibrate if flag is not set

    def animate(frame):
        imu_data = imu.get_data()
        gyro, accel = imu_data_to_numpy(imu_data)
        raw_gyro = np.array(
            [imu_data["gyro"]["x"], imu_data["gyro"]["y"], imu_data["gyro"]["z"]]
        )
        raw_accel = np.array(
            [imu_data["accel"]["x"], imu_data["accel"]["y"], imu_data["accel"]["z"]]
        )

        # Apply bias compensation only if calibration was performed
        if not args.no_calibration:
            gyro = gyro - state.gyro_bias

        state.Q = madgwick.updateIMU(state.Q, gyr=gyro, acc=accel)
        euler = np.degrees(orientation.q2euler(state.Q))
        return plotter.update(euler, raw_gyro, raw_accel, state.Q)

    ani = FuncAnimation(
        plotter.fig, animate, interval=10, blit=True, cache_frame_data=False
    )
    plt.show()
