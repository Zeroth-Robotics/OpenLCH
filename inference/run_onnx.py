"""Use onnxruntime to run a model and plot torques over time."""

import matplotlib
import matplotlib.pyplot as plt
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

    # dt = 1 / 50.0
    dt = 1.0

    for t in range(1000):
        elapsed_time = t * dt
        input_data["t.1"][0] = elapsed_time
        positions, actions, buffer = session.run(None, input_data)
        input_data["prev_actions.1"] = actions
        input_data["buffer.1"] = buffer
        input_data["dof_pos.1"] = positions
        if prev_positions is None:
            input_data["dof_vel.1"] = np.zeros(10).astype(np.float32)
        else:
            input_data["dof_vel.1"] = (positions - prev_positions) / dt
        prev_positions = positions

        # Store time and torque data
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
    # python run_onnx.py
    run_onnx_model()