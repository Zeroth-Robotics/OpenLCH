"""Robot configuration and control module."""

import math
import time
from typing import List, Dict, Tuple
import numpy as np
from openlch import HAL
from ahrs.filters import Madgwick
from ahrs.common import orientation

class JointData:
    """Represents a single joint in the robot."""
    def __init__(self, name: str, policy_index: int, servo_id: int, offset_deg: float = 0.0):
        self.name = name
        self.policy_index = policy_index
        self.servo_id = servo_id
        self.current_position = 0.0      # in radians
        self.current_velocity = 0.0      # in radians/s
        self.offset_deg = offset_deg     # in degrees

class IMUHandler:
    """Handles IMU data processing and orientation estimation."""
    def __init__(self, imu, madgwick_filter):
        self.imu = imu
        self.madgwick = madgwick_filter
        self.Q = np.array([1.0, 0.0, 0.0, 0.0])
        self.gyro_bias = np.zeros(3)
        self.calibrate_gyro()

    def calibrate_gyro(self, samples=100):
        """
        Calibrates the gyro bias and initializes orientation.
        """
        print("Calibrating gyro and initializing orientation, keep IMU still...")
        bias_sum = np.zeros(3)
        accel_sum = np.zeros(3)

        for _ in range(samples):
            imu_data = self.imu.get_data()
            gyro, accel = self.imu_data_to_numpy(imu_data)
            bias_sum += gyro
            accel_sum += accel
            time.sleep(0.02)

        self.gyro_bias = bias_sum / samples

        avg_accel = accel_sum / samples
        avg_accel = avg_accel / np.linalg.norm(avg_accel)

        roll = np.arctan2(avg_accel[1], avg_accel[2])
        pitch = np.arctan2(-avg_accel[0], np.sqrt(avg_accel[1] ** 2 + avg_accel[2] ** 2))

        cr, cp = np.cos(roll / 2), np.cos(pitch / 2)
        sr, sp = np.sin(roll / 2), np.sin(pitch / 2)

        self.Q = np.array([
            cr * cp,     # w
            sr * cp,     # x
            cr * sp,     # y
            -sr * sp     # z
        ])

        print(f"Initial roll: {np.degrees(roll):.1f}, pitch: {np.degrees(pitch):.1f}")

    def get_orientation(self):
        """
        Retrieves the current orientation using the Madgwick filter.
        """
        imu_data = self.imu.get_data()
        gyro, accel = self.imu_data_to_numpy(imu_data)
        gyro -= self.gyro_bias
        self.Q = self.madgwick.updateIMU(self.Q, gyr=gyro, acc=accel)
        euler_angles = np.degrees(orientation.q2euler(self.Q))
        return gyro, accel, euler_angles, self.Q  # gyro in rad/s, accel in g, euler_angles in degrees, quaternion

    @staticmethod
    def imu_data_to_numpy(imu_data):
        """
        Converts IMU data to numpy arrays for gyro and accelerometer.
        """
        gyro = imu_data['gyro']
        accel = imu_data['accel']
        # Convert to proper units: gyro to rad/s, accel to g
        gyro_array = np.array([gyro['x'], gyro['y'], gyro['z']]) * np.pi / 180.0  # deg/s to rad/s
        accel_array = np.array([accel['x'], accel['y'], accel['z']]) / 1000.0     # mg to g
        return gyro_array, accel_array

class Robot:
    """Encapsulates robot hardware interactions."""
    def __init__(self, hal: HAL):
        self.hal = hal

        print("Initializing joints...")
        self.joints: List[JointData] = self.initialize_joints()

        print("Initializing servos...")
        self.servo_positions_dict: Dict[int, Tuple[float, float]] = {}
        self.initialize_servos()

        print("Initializing IMU...")
        self.imu = self.hal.imu
        self.madgwick_filter = Madgwick(frequency=100.0, beta=0.7)
        self.imu_handler = IMUHandler(self.imu, self.madgwick_filter)

        print("Initialization complete.")

    def initialize_joints(self) -> List[JointData]:
        """Initializes the joint data."""
        return [
            JointData(name="left_hip_pitch", policy_index=0, servo_id=10, offset_deg=0.0),
            JointData(name="left_hip_yaw", policy_index=1, servo_id=9, offset_deg=45.0),
            JointData(name="left_hip_roll", policy_index=2, servo_id=8, offset_deg=0.0),
            JointData(name="left_knee_pitch", policy_index=3, servo_id=7, offset_deg=0.0),
            JointData(name="left_ankle_pitch", policy_index=4, servo_id=6, offset_deg=0.0),
            JointData(name="right_hip_pitch", policy_index=5, servo_id=5, offset_deg=0.0),
            JointData(name="right_hip_yaw", policy_index=6, servo_id=4, offset_deg=-45.0),
            JointData(name="right_hip_roll", policy_index=7, servo_id=3, offset_deg=0.0),
            JointData(name="right_knee_pitch", policy_index=8, servo_id=2, offset_deg=0.0),
            JointData(name="right_ankle_pitch", policy_index=9, servo_id=1, offset_deg=0.0),
        ]

    def initialize_servos(self) -> None:
        """Initializes the robot's servos and sets initial positions."""
        self.hal.servo.scan()
        self.hal.servo.set_torque_enable([(joint.servo_id, True) for joint in self.joints])
        time.sleep(1)
        self.hal.servo.set_torque([(joint.servo_id, 30.0) for joint in self.joints])
        time.sleep(1)

        # Set initial desired positions to zero
        initial_positions = [0.0] * len(self.joints)
        self.set_servo_positions(initial_positions)
        time.sleep(3)

        print("Starting inference in...")
        for i in range(3, 0, -1):
            print(f"{i}...")
            time.sleep(0.5)

    def get_servo_states(self) -> None:
        """
        Reads servo states and updates current positions and velocities.
        """
        servo_positions = self.hal.servo.get_positions()
        self.servo_positions_dict = {id_: (pos, vel) for id_, pos, vel in servo_positions}

        for joint in self.joints:
            if joint.servo_id in self.servo_positions_dict:
                pos_deg, vel_deg_s = self.servo_positions_dict[joint.servo_id]
                joint.current_position = math.radians(pos_deg - joint.offset_deg)
                joint.current_velocity = math.radians(vel_deg_s)
            else:
                joint.current_position = 0.0
                joint.current_velocity = 0.0

    def set_servo_positions(self, desired_positions: List[float]) -> None:
        """
        Sends desired positions to servos, applying offsets.

        :param desired_positions: A list of desired joint positions in radians.
        """
        if len(desired_positions) != len(self.joints):
            raise ValueError("Length of desired_positions must match number of joints.")

        positions_deg = []
        for joint, desired_pos in zip(self.joints, desired_positions):
            # Convert from radians to degrees and apply offset
            pos_deg = math.degrees(desired_pos) + joint.offset_deg
            positions_deg.append((joint.servo_id, pos_deg))

        self.hal.servo.set_positions(positions_deg)

    def get_joint_positions(self) -> List[float]:
        """
        Returns the current positions of the joints.

        :return: A list of current joint positions in radians.
        """
        return [joint.current_position for joint in self.joints]

    def get_joint_velocities(self) -> List[float]:
        """
        Returns the current velocities of the joints.

        :return: A list of current joint velocities in radians per second.
        """
        return [joint.current_velocity for joint in self.joints]
