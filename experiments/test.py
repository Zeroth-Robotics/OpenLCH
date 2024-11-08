from robot import Robot, RobotConfig

def main():
    config = RobotConfig()
    robot = Robot(config)
    robot.initialize()
    
    desired_positions = {
        "left_hip_pitch": 0.0,      
        "left_hip_yaw": 0.0,
        "left_hip_roll": 0.0,
        "left_knee_pitch": 0.0,
        "left_ankle_pitch": 0.0,
        "right_hip_pitch": 0.0,
        "right_hip_yaw": 0.0,
        "right_hip_roll": 0.0,
        "right_knee_pitch": 0.0,
        "right_ankle_pitch": 0.0,
        "right_elbow_yaw": 0.0,
        "right_shoulder_yaw": 0.0,
        "right_shoulder_pitch": 0.0,
        "left_shoulder_pitch": 0.0,
        "left_shoulder_yaw": 0.0,
        "left_elbow_yaw": 0.0,
    }

    robot.set_servo_positions(desired_positions)
    
    current_positions = robot.get_feedback_positions()
    print("Current Positions:", current_positions)
    
    current_velocities = robot.get_feedback_velocities()
    print("Current Velocities:", current_velocities)

if __name__ == "__main__":
    main()