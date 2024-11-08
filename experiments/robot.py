import math
from openlch import HAL
import subprocess


class JointData:
    def __init__(
        self, name: str, policy_index: int, servo_id: int, offset_deg: float = 0.0
    ):
        self.name = name
        self.policy_index = policy_index
        self.servo_id = servo_id
        self.current_position = 0.0  # in radians
        self.desired_position = 0.0  # in radians
        self.current_velocity = 0.0  # in radians/s
        self.offset_deg = offset_deg  # in degrees


class Robot:
    def __init__(self):
        self.hal = HAL()
        self.joints = [
            # LEGS
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
            # ARMS
            JointData(name="right_elbow_yaw", policy_index=10, servo_id=11, offset_deg=0.0),
            JointData(name="right_shoulder_yaw", policy_index=11, servo_id=12, offset_deg=0.0),
            JointData(name="right_shoulder_pitch", policy_index=12, servo_id=13, offset_deg=0.0),
            JointData(name="left_shoulder_pitch", policy_index=13, servo_id=14, offset_deg=0.0),
            JointData(name="left_shoulder_yaw", policy_index=14, servo_id=15, offset_deg=0.0),
            JointData(name="left_elbow_yaw", policy_index=15, servo_id=16, offset_deg=0.0),
        ]

        # Create a dictionary mapping joint names to JointData instances
        self.joint_dict = {joint.name: joint for joint in self.joints}

    def initialize(self):
        print("--------------------------------")
        print("\n[INFO] Robot initializing...")

        print("\n[INFO] Checking connection to robot...")
        try:
            ping_result = subprocess.run(
                ["ping", "-c", "1", "192.168.42.1"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if ping_result.returncode != 0:
                raise RuntimeError("Could not ping robot at 192.168.42.1")
            print("[INFO] Successfully pinged robot")
        except Exception as e:
            print(f"[ERROR] Failed to connect to robot: {str(e)}")
            raise

        print("\n[INFO] Scanning servos...")
        print(self.hal.servo.scan())

        print("\n[INFO] Setting torque enable to true...")
        self.hal.servo.set_torque_enable(
            [(joint.servo_id, True) for joint in self.joints]
        )

        print("\n[INFO] Setting torque to 30.0...")
        self.hal.servo.set_torque([(joint.servo_id, 30.0) for joint in self.joints])

        print("\n[INFO] Setting initial positions to 0.0...")
        for joint in self.joints:
            joint.desired_position = 0.0
        self.set_servo_positions()

        print("\n[INFO] Robot initialized")
        print("--------------------------------")

    def get_servo_states(self):
        """Retrieve current servo positions and velocities."""
        servo_positions = self.hal.servo.get_positions()
        servo_positions_dict = {id_: (pos, vel) for id_, pos, vel in servo_positions}

        for joint in self.joints:
            if joint.servo_id in servo_positions_dict:
                pos_deg, vel_deg_s = servo_positions_dict[joint.servo_id]
                joint.current_position = math.radians(pos_deg)
                joint.current_velocity = math.radians(vel_deg_s)
            else:
                raise RuntimeError(
                    f"Failed to get state for servo ID {joint.servo_id} ({joint.name})"
                )

    def set_servo_positions(self):
        """Set servo positions based on desired joint positions."""
        positions_deg = []
        for joint in self.joints:
            desired_pos_deg = math.degrees(joint.desired_position) + joint.offset_deg
            positions_deg.append((joint.servo_id, desired_pos_deg))
        self.hal.servo.set_positions(positions_deg)

    def set_servo_positions_by_name(self, positions):
        """Set servo positions using joint names.

        Args:
            positions (dict): Dictionary with joint names as keys and desired positions in radians as values.
        """
        positions_deg = []
        for name, position in positions.items():
            if name in self.joint_dict:
                joint = self.joint_dict[name]
                joint.desired_position = position
                desired_pos_deg = math.degrees(position) + joint.offset_deg
                positions_deg.append((joint.servo_id, desired_pos_deg))
            else:
                raise ValueError(f"Joint name '{name}' not found.")
        self.hal.servo.set_positions(positions_deg)

    def set_joint_positions(self, positions):
        """Set desired positions for all joints.

        Args:
            positions (Iterable[float]): Desired positions in radians for each joint.
        """
        if len(positions) != len(self.joints):
            raise ValueError("Positions length does not match number of joints.")
        for joint, position in zip(self.joints, positions):
            joint.desired_position = position
        self.set_servo_positions()

    def get_current_positions(self):
        """Get the current positions of all joints.

        Returns:
            List[float]: Current joint positions in radians.
        """
        return [joint.current_position for joint in self.joints]

    def get_current_velocities(self):
        """Get the current velocities of all joints.

        Returns:
            List[float]: Current joint velocities in radians per second.
        """
        return [joint.current_velocity for joint in self.joints]

    def disable_motors(self):
        """Disable all motors."""
        self.hal.servo.set_torque_enable(
            [(joint.servo_id, False) for joint in self.joints]
        )
