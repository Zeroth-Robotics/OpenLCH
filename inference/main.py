"""
this is for running the model on the robot

note that all arms are not used (as it's not used in the training)
"""

import numpy as np
import onnxruntime as ort
from openlch import HAL


def get_velocities(current_positions : list, previous_positions : list, dt : float) -> list:
    velocities = []
    for i in range(len(current_positions)):
        velocities.append((current_positions[i] - previous_positions[i]) / dt)
    return velocities

def get_servo_positions(hal: HAL) -> list:
    servo_positions = hal.servo.get_positions()
    positions = [pos for _, pos in servo_positions[:10]]
    print(f"[INFO]: GET servo positions: {positions}")
    return positions

def set_servo_positions(positions : list, hal : HAL) -> None:
    print(f"[INFO]: SET servo positions: {positions}")
    servo_positions = [(i, pos) for i, pos in enumerate(positions[:10])]
    hal.servo.set_positions(servo_positions)

def inference(session : ort.InferenceSession, hal : HAL) -> None:
    print(f"[INFO]: Inference starting...")

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

    for t in range(1000):

        ### ====[ UPDATE INPUT DATA ]=== ###
        elapsed_time = t * dt
        input_data["t.1"][0] = elapsed_time

        # get current positions
        current_positions = get_servo_positions(hal)

        # convert list to numpy array and change input data dof_pos
        input_data["dof_pos.1"] = np.array(current_positions).astype(np.float32)
        # input_data["dof_vel.1"] = np.array(get_velocities(current_positions, dt)).astype(np.float32)

        ### ====[ RUN MODEL ]=== ###
        desired_positions, actions, buffer = session.run(None, input_data)

        # update input data
        input_data["prev_actions.1"] = actions
        input_data["buffer.1"] = buffer
        input_data["dof_pos.1"] = desired_positions

        # convert current_positions to numpy array
        current_positions_np = np.array(current_positions, dtype=np.float32)

        # compute velocity
        if prev_positions is None:
            input_data["dof_vel.1"] = np.zeros(10, dtype=np.float32)
        else:
            input_data["dof_vel.1"] = (current_positions_np - prev_positions) / dt
        prev_positions = current_positions_np


        ### ====[ SEND TO ROBOT ]=== ###

        # convert from numpy array to list
        command_positions = desired_positions.tolist()

        # send to robot
        set_servo_positions(command_positions, hal)

if __name__ == "__main__":

    hal = HAL()

    MODEL_PATH = "standing_micro_fixed.onnx"
    session = ort.InferenceSession(MODEL_PATH)

    inference(session, hal)
