"""
this is for running the model on the robot

note that all arms are not used (as it's not used in the training)
"""

import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import onnxruntime as ort
from dataclasses import dataclass, field
from openlch import HAL


# @dataclass
# class JointData:
#     position : float = 0.0
#     velocity : float = 0.0

# @dataclass
# class LegData:
#     hip_roll : JointData = field(default_factory=JointData)
#     hip_yaw : JointData = field(default_factory=JointData)
#     hip_pitch : JointData = field(default_factory=JointData)
#     knee_pitch : JointData = field(default_factory=JointData)
#     ankle_pitch : JointData = field(default_factory=JointData)

#     def set_feedback(self, position_feedback : list, velocity_feedback : list) -> None:
#         self.hip_roll.position = position_feedback[0]
#         self.hip_yaw.position = position_feedback[1]
#         self.hip_pitch.position = position_feedback[2]
#         self.knee_pitch.position = position_feedback[3]
#         self.ankle_pitch.position = position_feedback[4]

#         self.hip_roll.velocity = velocity_feedback[0]
#         self.hip_yaw.velocity = velocity_feedback[1]
#         self.hip_pitch.velocity = velocity_feedback[2]
#         self.knee_pitch.velocity = velocity_feedback[3]
#         self.ankle_pitch.velocity = velocity_feedback[4]

# @dataclass
# class ArmData:
#     shoulder_yaw : JointData = field(default_factory=JointData)
#     shoulder_pitch : JointData = field(default_factory=JointData)
#     elbow_yaw : JointData = field(default_factory=JointData)

#     def set_feedback(self, position_feedback : list, velocity_feedback : list) -> None:
#         self.shoulder_yaw.position = position_feedback[0]
#         self.shoulder_pitch.position = position_feedback[1]
#         self.elbow_yaw.position = position_feedback[2]

#         self.shoulder_yaw.velocity = velocity_feedback[0]
#         self.shoulder_pitch.velocity = velocity_feedback[1]
#         self.elbow_yaw.velocity = velocity_feedback[2]

# @dataclass
# class RobotData:
#     left_leg: LegData = field(default_factory=LegData)
#     right_leg: LegData = field(default_factory=LegData)
#     left_arm : ArmData = field(default_factory=ArmData)
#     right_arm : ArmData = field(default_factory=ArmData)



# def initialize_robot_data(robot_data : RobotData) -> None:
#     robot_data.left_leg.set_feedback(get_servo_positions()[0:5], [0.0, 0.0, 0.0, 0.0, 0.0])
#     robot_data.right_leg.set_feedback(get_servo_positions()[5:10], [0.0, 0.0, 0.0, 0.0, 0.0])
#     # robot_data.left_arm.set_feedback(get_servo_positions()[10:13], [0.0, 0.0, 0.0])
#     # robot_data.right_arm.set_feedback(get_servo_positions()[13:16], [0.0, 0.0, 0.0])
#     print("robot data initialized: ", robot_data)


def get_velocities(current_positions : list, previous_positions : list, dt : float) -> list:
    velocities = []
    for i in range(len(current_positions)):
        velocities.append((current_positions[i] - previous_positions[i]) / dt)
    return velocities


def get_servo_positions() -> list:
    positions = hal.servo.get_position()
    return positions

def set_servo_positions(positions : list) -> None:
    hal.servo.set_position(positions)

def inference(session : ort.InferenceSession #, robot_data : RobotData
             ) -> None:
    
    # initialized input data
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

    prev_positions: np.ndarray | None = None

    dt = 1 / 50.0 # 50 Hz

    for t in range(10000):

        ### ====[ UPDATE INPUT DATA ]=== ###
        elapsed_time = t * dt
        input_data["t.1"][0] = elapsed_time

        # get current positions
        current_positions = get_servo_positions()
        # convert list to numpy array and change input data dof_pos
        input_data["dof_pos.1"] = np.array(current_positions).astype(np.float32)
        input_data["dof_vel.1"] = np.array(get_velocities(current_positions, dt)).astype(np.float32)

        ### ====[ RUN MODEL ]=== ###
        desired_positions, actions, buffer = session.run(None, input_data)

        # update input data
        input_data["prev_actions.1"] = actions
        input_data["buffer.1"] = buffer
        input_data["dof_pos.1"] = desired_positions

        # compute velocity
        if prev_positions is None:
            input_data["dof_vel.1"] = np.zeros(10).astype(np.float32)
        else:
            input_data["dof_vel.1"] = (current_positions - prev_positions) / dt
        prev_positions = current_positions

        # print(f"Position: {positions}")

        ### ====[ SEND TO ROBOT ]=== ###

        # convert from numpy array to list
        positions_list = current_positions.tolist()

        # send to robot
        set_servo_positions(positions_list)

        # print(f"Positions List: {positions_list}")
        # update robot data
        # robot_data.left_leg.set_feedback(positions_list[0:5], velocity_list[0:5])
        # robot_data.right_leg.set_feedback(positions_list[5:10], velocity_list[5:10])

        # print robot for each leg data
        # print(f"Left Leg: {robot_data.left_leg.hip_roll.position}, {robot_data.left_leg.hip_yaw.position}, {robot_data.left_leg.hip_pitch.position}, {robot_data.left_leg.knee_pitch.position}, {robot_data.left_leg.ankle_pitch.position}")
        # print(f"Right Leg: {robot_data.right_leg.hip_roll.position}, {robot_data.right_leg.hip_yaw.position}, {robot_data.right_leg.hip_pitch.position}, {robot_data.right_leg.knee_pitch.position}, {robot_data.right_leg.ankle_pitch.position}")



    # for i, line in enumerate(lines):
    #     line.set_data(time_data, position_data[i])
    # ax.relim()
    # ax.autoscale_view()

    # plt.show()



if __name__ == "__main__":

    hal = HAL()
    # robot_data = RobotData()
    # initialize_robot_data(robot_data)

    MODEL_PATH = "standing_micro.onnx"
    session = ort.InferenceSession(MODEL_PATH)

    inference(session)# robot_data)
