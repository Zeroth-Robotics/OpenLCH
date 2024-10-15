"""Use onnxruntime to run a model."""

import time

import numpy as np
import onnxruntime as ort


def run_onnx_model() -> None:
    session = ort.InferenceSession("position_control.onnx")

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

    def get_joint_angles() -> np.ndarray:
        return np.zeros(10).astype(np.float32)

    def get_joint_velocities() -> np.ndarray:
        return np.zeros(10).astype(np.float32)

    def send_torques(torques: np.ndarray) -> None:
        print(torques)

    start_time = time.time()
    while True:
        elapsed_time = time.time() - start_time
        input_data["t.1"][0] = elapsed_time
        input_data["dof_pos.1"] = get_joint_angles()
        input_data["dof_vel.1"] = get_joint_velocities()
        torques, actions, buffer = session.run(None, input_data)
        input_data["prev_actions.1"] = actions
        input_data["buffer.1"] = buffer
        send_torques(torques)
        time.sleep(1 / 50)


if __name__ == "__main__":
    # python run_onnx.py
    run_onnx_model()
