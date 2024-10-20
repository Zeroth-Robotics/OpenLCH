import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import onnxruntime as ort
from dataclasses import dataclass, field


# define robot data
@dataclass
class JointData:
    position : float = 0.0
    velocity : float = 0.0

@dataclass
class LegData:
    hip_roll : JointData = field(default_factory=JointData)
    hip_yaw : JointData = field(default_factory=JointData)
    hip_pitch : JointData = field(default_factory=JointData)
    knee_pitch : JointData = field(default_factory=JointData)
    ankle_pitch : JointData = field(default_factory=JointData)

    def set_feedback(self, feedback : list) -> None:
        self.hip_roll.position = 
        self.hip_yaw.position = 
        self.hip_pitch.position = 
        self.knee_pitch.position = 
        self.ankle_pitch.position =

        self.hip_roll.velocity = 
        self.hip_yaw.velocity = 
        self.hip_pitch.velocity = 
        self.knee_pitch.velocity = 
        self.ankle_pitch.velocity =

@dataclass
class ArmData:
    shoulder_yaw : JointData = field(default_factory=JointData)
    shoulder_pitch : JointData = field(default_factory=JointData)
    elbow_yaw : JointData = field(default_factory=JointData)

    def set_feedback(self, feedback : list) -> None:
        self.shoulder_yaw.position =
        self.shoulder_pitch.position = 
        self.elbow_yaw.position = 

        self.shoulder_yaw.velocity =
        self.shoulder_pitch.velocity = 
        self.elbow_yaw.velocity = 

@dataclass
class RobotData:
    left_leg: LegData = field(default_factory=LegData)
    right_leg: LegData = field(default_factory=LegData)
    left_arm : ArmData = field(default_factory=ArmData)
    right_arm : ArmData = field(default_factory=ArmData)



# class Joint:
#     def __init__(self, name, servo_id):
#         self.name = name
#         self.servo_id = servo_id 
#         self.current_position = 0.0


# DOF_NAMES = [
#     # "left_shoulder_yaw"
#     # "left_shoulder_pitch"
#     # "left_elbow_yaw" 
#     # "right_shoulder_yaw"
#     # "right_shoulder_pitch"
#     # "right_elbow_yaw" 

#     "left_hip_roll",
#     "left_hip_yaw",
#     "left_hip_pitch",
#     "left_knee_pitch",
#     "left_ankle_pitch",

#     "right_hip_roll",
#     "right_hip_yaw",
#     "right_hip_pitch",
#     "right_knee_pitch",
#     "right_ankle_pitch",

# ]

# SERVO_ID = {
#     "left_ankle_pitch" : 1,
#     "left_knee_pitch" : 2, 
#     "left_hip_pitch" : 3,
#     "left_hip_yaw" : 4,
#     "left_hip_roll" : 5,

#     "right_ankle_pitch" : 6,
#     "right_knee_pitch" : 7,
#     "right_hip_pitch" : 8, 
#     "right_hip_yaw" : 9,
#     "right_hip_roll" : 10,

#     # "left_shoulder_yaw" = 11,
#     # "left_shoulder_pitch" = 12,
#     # "left_elbow_yaw" = 13,
#     # "right_shoulder_yaw" = 14,
#     # "right_shoulder_pitch" = 15,
#     # "right_elbow_yaw" = 16,

# }


# joints = [Joint(name, SERVO_ID[name]) for name in DOF_NAMES]

# for joint in joints:
#     print(f"Joint: {joint.name}, Servo ID: {joint.servo_id}, Current Position: {joint.current_position}")





def inference(model_path : str) -> None:


    session = ort.InferenceSession(model_path)

    input_data = {
        "x_vel.1": np.zeros(1).astype(np.float32),
        "y_vel.1": np.zeros(1).astype(np.float32),
        "rot.1": np.zeros(1).astype(np.float32),
        "t.1": np.zeros(1).astype(np.float32),
        "dof_pos.1": np.zeros(10).astype(np.float32),
        "dof_vel.1": np.zeros(10).astype(np.float32),
        "prev_actions.1": np.zeros(10).astype(np.float32),
        "imu_ang_vel.1": np.zeros(3).astype(np.float32),
        "imu_euler_xyz.1": np.zeros(3).astype(np.float32),
        "buffer.1": np.zeros(574).astype(np.float32),
    }

    # Initialize lists to store time and torque data for plotting
    time_data = []
    position_data = [[] for _ in range(10)]  # Assuming 10 joints

    prev_positions: np.ndarray | None = None

    matplotlib.use("qtagg")
    _, ax = plt.subplots(figsize=(10, 6))
    lines = [ax.plot([], [], label=f"Joint {i+1}")[0] for i in range(10)]
    ax.set_xlabel("Time (s)")
    ax.set_ylabel("Position")
    ax.set_title("Joint Positions Over Time")
    ax.legend()


    dt = 1 / 50.0 # 50 Hz

    for t in range(10000):
        elapsed_time = t * dt
        input_data["t.1"][0] = elapsed_time

        # run model
        positions, actions, buffer = session.run(None, input_data)

        # update input data
        input_data["prev_actions.1"] = actions
        input_data["buffer.1"] = buffer
        input_data["dof_pos.1"] = positions

        # compute velocity
        if prev_positions is None:
            input_data["dof_vel.1"] = np.zeros(10).astype(np.float32)
        else:
            input_data["dof_vel.1"] = (positions - prev_positions) / dt
        prev_positions = positions

        # Store time and position data
        time_data.append(elapsed_time)
        for i, position in enumerate(positions):
            position_data[i].append(position)

        print(f"Position: {positions}")

    for i, line in enumerate(lines):
        line.set_data(time_data, position_data[i])
    ax.relim()
    ax.autoscale_view()

    plt.show()


if __name__ == "__main__":
    print("start")

    # inference("standing_micro.onnx")
