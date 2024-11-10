"""
Module: robot.py
Simplified Robot class with computations in degrees and direct position/velocity methods.
"""

import subprocess
import logging
from typing import List, Dict, Tuple
from openlch import HAL
import math

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class Joint:
    """Represents a single joint of the robot, using degrees for positions and velocities."""

    def __init__(
        self, 
        name: str, 
        servo_id: int, 
        offset_deg: float = 0.0
    ):
        self.name: str = name
        self.servo_id: int = servo_id
        self.offset_deg: float = offset_deg  # Offset in degrees
        self.feedback_position: float = 0.0  # Feedback position in degrees
        self.feedback_velocity: float = 0.0  # Feedback velocity in degrees/s

class RobotConfig:
    """Configuration parameters for the Robot."""

    def __init__(self):
        self.joint_configs = [
            # LEGS
            {'name': "left_hip_pitch", 'servo_id': 10, 'offset_deg': 0.23 * 180/math.pi + 5.0},
            {'name': "left_hip_yaw", 'servo_id': 9, 'offset_deg': 45.0},
            {'name': "left_hip_roll", 'servo_id': 8, 'offset_deg': 0.0},
            {'name': "left_knee_pitch", 'servo_id': 7, 'offset_deg': -0.741 * 180/math.pi},
            {'name': "left_ankle_pitch", 'servo_id': 6, 'offset_deg': -0.5 * 180/math.pi},
            {'name': "right_hip_pitch", 'servo_id': 5, 'offset_deg': -0.23 * 180/math.pi - 5.0},
            {'name': "right_hip_yaw", 'servo_id': 4, 'offset_deg': -45.0},
            {'name': "right_hip_roll", 'servo_id': 3, 'offset_deg': 0.0},
            {'name': "right_knee_pitch", 'servo_id': 2, 'offset_deg': 0.741 * 180/math.pi},
            {'name': "right_ankle_pitch", 'servo_id': 1, 'offset_deg': 0.5 * 180/math.pi},
            # ARMS
            {'name': "right_elbow_yaw", 'servo_id': 11, 'offset_deg': 0.0},
            {'name': "right_shoulder_yaw", 'servo_id': 12, 'offset_deg': 0.0},
            {'name': "right_shoulder_pitch", 'servo_id': 13, 'offset_deg': 0.0},
            {'name': "left_shoulder_pitch", 'servo_id': 14, 'offset_deg': 0.0},
            {'name': "left_shoulder_yaw", 'servo_id': 15, 'offset_deg': 0.0},
            {'name': "left_elbow_yaw", 'servo_id': 16, 'offset_deg': 0.0},
        ]
        self.torque_enable = True
        self.torque_value = 20.0

class Robot:
    """Controls the robot's hardware and joint movements."""

    def __init__(self):
        self.hal = HAL()
        self.config = RobotConfig()
        self.joints: List[Joint] = [
            Joint(**joint_config) for joint_config in self.config.joint_configs
        ]
        self.joint_dict: Dict[str, Joint] = {joint.name: joint for joint in self.joints}

    def initialize(self) -> None:
        """Initializes the robot's hardware and joints."""
        logger.info("Robot initializing...")
        self.check_connection()
        self.setup_servos()
        self.set_initial_positions()
        logger.info("Robot initialized.")

    def check_connection(self) -> None:
        """Checks the connection to the robot."""
        logger.info("Checking connection to robot...")
        try:
            subprocess.run(
                ["ping", "-c", "1", "192.168.42.1"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=True
            )
            logger.info("Successfully pinged robot.")
        except subprocess.CalledProcessError:
            logger.error("Could not ping robot at 192.168.42.1")
            raise ConnectionError("Robot connection failed.")

    def setup_servos(self) -> None:
        """Sets up servo parameters."""
        logger.info("Scanning servos...")
        servo_ids = [joint.servo_id for joint in self.joints]
        available_servos = self.hal.servo.scan()
        logger.debug(f"Available servos: {available_servos}")

        self.hal.servo.set_torque_enable(
            [(servo_id, self.config.torque_enable) for servo_id in servo_ids]
        )
        self.hal.servo.set_torque(
            [(servo_id, self.config.torque_value) for servo_id in servo_ids]
        )

        self.hal.servo.enable_movement()

    def set_initial_positions(self) -> None:
        """Sets initial positions for all joints to 0 degrees."""
        logger.info("Setting initial positions to 0.0 degrees.")
        positions = {joint.name: 0.0 for joint in self.joints}
        self.set_desired_positions(positions)

    def get_feedback_positions(self) -> Dict[str, float]:
        """Gets feedback positions from the servos.

        Returns:
            A dictionary mapping joint names to their feedback positions in degrees.
        """
        servo_states = self.hal.servo.get_positions()  # [(id_, position_deg, velocity_deg_s), ...]
        # Build a mapping from servo_id to position
        servo_position_dict = {servo_id: position_deg for servo_id, position_deg, _ in servo_states}
        feedback_positions = {}
        for joint in self.joints:
            if joint.servo_id in servo_position_dict:
                position_deg = servo_position_dict[joint.servo_id] - joint.offset_deg
                joint.feedback_position = position_deg
                feedback_positions[joint.name] = position_deg
            else:
                logger.warning(f"Servo ID {joint.servo_id} not found in servo states.")
                feedback_positions[joint.name] = None  # Or handle as appropriate
        return feedback_positions

    def get_feedback_velocities(self) -> Dict[str, float]:
        """Gets feedback velocities from the servos.

        Returns:
            A dictionary mapping joint names to their feedback velocities in degrees/s.
        """
        servo_states = self.hal.servo.get_positions()  # [(id_, position_deg, velocity_deg_s), ...]
        # Build a mapping from servo_id to velocity
        servo_velocity_dict = {servo_id: velocity_deg_s for servo_id, _, velocity_deg_s in servo_states}
        feedback_velocities = {}
        for joint in self.joints:
            if joint.servo_id in servo_velocity_dict:
                velocity_deg_s = servo_velocity_dict[joint.servo_id]
                joint.feedback_velocity = velocity_deg_s
                feedback_velocities[joint.name] = velocity_deg_s
            else:
                logger.warning(f"Servo ID {joint.servo_id} not found in servo states.")
                feedback_velocities[joint.name] = None  # Or handle as appropriate
        return feedback_velocities

    def set_desired_positions(self, positions: Dict[str, float]) -> None:
        """Sets desired positions for specified joints directly to the servos.

        Args:
            positions: A dictionary mapping joint names to desired positions in degrees.
        """
        position_commands: List[Tuple[int, float]] = []
        for name, position in positions.items():
            if name in self.joint_dict:
                joint = self.joint_dict[name]
                desired_position = position + joint.offset_deg
                position_commands.append((joint.servo_id, desired_position))
            else:
                logger.error(f"Joint name '{name}' not found.")
                raise ValueError(f"Joint name '{name}' not found.")
        # Send positions to servos
        self.hal.servo.set_positions(position_commands)

    def disable_motors(self) -> None:
        logger.info("Disabling all motors.")
        self.hal.servo.disable_movement()
        servo_ids = [joint.servo_id for joint in self.joints]
        self.hal.servo.set_torque_enable(
            [(servo_id, False) for servo_id in servo_ids]
        )

