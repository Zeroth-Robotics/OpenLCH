import math
from openlch import HAL
import time


class JointData:
    def __init__(self, name: str, policy_index: int, servo_id: int, offset_deg: float = 0.0):
        self.name = name
        self.policy_index = policy_index
        self.servo_id = servo_id
        self.current_position = 0.0      # in radians
        self.desired_position = 0.0      # in radians
        self.current_velocity = 0.0      # in radians/s
        self.offset_deg = offset_deg     # in degrees


class Robot:
    def __init__(self):
        self.hal = HAL()
        self.joints = [
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

    def initialize(self):
        """Initialize the robot hardware and set initial positions."""
        self.hal.servo.scan()
        self.hal.servo.set_torque_enable([(joint.servo_id, True) for joint in self.joints])
        time.sleep(1)
        self.hal.servo.set_torque([(joint.servo_id, 30.0) for joint in self.joints])
        time.sleep(1)
        self.hal.servo.disable_movement()
        
        # Set initial desired positions
        for joint in self.joints:
            joint.desired_position = 0.0
        self.set_servo_positions()
        time.sleep(3)

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
                raise RuntimeError(f"Failed to get state for servo ID {joint.servo_id} ({joint.name})")

    def set_servo_positions(self):
        """Set servo positions based on desired joint positions."""
        positions_deg = []
        for joint in self.joints:
            desired_pos_deg = math.degrees(joint.desired_position) + joint.offset_deg
            positions_deg.append((joint.servo_id, desired_pos_deg))
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
