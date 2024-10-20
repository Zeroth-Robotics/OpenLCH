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


        ### ====[ SEND TO ROBOT ]=== ###

        # convert from numpy array to list
        positions_list = current_positions.tolist()

        # send to robot
        set_servo_positions(positions_list)




if __name__ == "__main__":

    hal = HAL()

    MODEL_PATH = "standing_micro.onnx"
    session = ort.InferenceSession(MODEL_PATH)

    inference(session)# robot_data)
