





class JointData:
    def __init__(self, name: str, policy_index: int, servo_id: int, offset_deg: float = 0.0):
        self.name = name
        self.policy_index = policy_index
        self.servo_id = servo_id
        self.current_position = 0.0      # in radians
        self.desired_position = 0.0      # in radians
        self.current_velocity = 0.0      # in radians/s
        self.offset_deg = offset_deg     # in degrees




joints = [
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


def get_servo_states(hal: HAL) -> None:
    if MOCK:
        for joint in joints:
            joint.current_position = 0.0
            joint.current_velocity = 0.0
        return
    
    # Placeholder for actual servo state retrieval
    servo_positions = hal.servo.get_positions() 
    servo_positions_dict = {id_: (pos, vel) for id_, pos, vel in servo_positions}
    
    for joint in joints:
        if joint.servo_id in servo_positions_dict:
            pos_deg, vel_deg_s = servo_positions_dict[joint.servo_id]
            joint.current_position = math.radians(pos_deg)
            joint.current_velocity = math.radians(vel_deg_s)
        else:
            joint.current_position = 0.0
            joint.current_velocity = 0.0


def set_servo_positions(hal: HAL) -> None:
    positions_deg = []
    for joint in joints:
        # convert from radians to degrees
        desired_pos_deg = math.degrees(joint.desired_position)
        desired_pos_deg += joint.offset_deg
        positions_deg.append((joint.servo_id, desired_pos_deg))

    # print(f"[INFO]: SET servo positions (deg): {positions_deg}")

    if MOCK:
        return

    hal.servo.set_positions(positions_deg)
