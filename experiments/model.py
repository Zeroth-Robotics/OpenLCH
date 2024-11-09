"""Inference script for running the model on the robot
Run:
    python experiments/model.py --model_path sim/examples/walking_micro.onnx
"""

import argparse
import math
import time
from collections import deque
import numpy as np
import onnxruntime as ort
import multiprocessing as mp
from robot import Robot, RobotConfig
from plot import run_dashboard
import threading


class Sim2simCfg:
    def __init__(
        self,
        num_actions=10,
        frame_stack=15,
        c_frame_stack=3,
        sim_duration=60.0,
        stiffness=5.0,
        damping=0.3,
        effort=1.0,
        dt=0.001,
        decimation=10,
        cycle_time=0.4,
        tau_factor=3,
        lin_vel=2.0,
        ang_vel=1.0,
        dof_pos=1.0,
        dof_vel=0.05,
        clip_observations=18.0,
        clip_actions=18.0,
        action_scale=0.25,
    ):
        self.num_actions = num_actions

        self.frame_stack = frame_stack
        self.c_frame_stack = c_frame_stack
        self.num_single_obs = 11 + self.num_actions * self.c_frame_stack
        self.num_observations = int(self.frame_stack * self.num_single_obs)

        self.sim_duration = sim_duration
        self.dt = dt
        self.decimation = decimation

        self.cycle_time = cycle_time

        self.tau_factor = tau_factor
        self.tau_limit = np.array([effort] * self.num_actions) * self.tau_factor
        self.kps = np.array([stiffness] * self.num_actions)
        self.kds = np.array([damping] * self.num_actions)

        self.lin_vel = lin_vel
        self.ang_vel = ang_vel
        self.dof_pos = dof_pos
        self.dof_vel = dof_vel

        self.clip_observations = clip_observations
        self.clip_actions = clip_actions

        self.action_scale = action_scale


class cmd:
    vx = 0.4
    vy = 0.0
    dyaw = 0.0


def inference(policy: ort.InferenceSession, robot: Robot, data_queue: mp.Queue, stop_event: threading.Event) -> None:
    cfg = Sim2simCfg()

    print("[INFO]: Starting ONNX model inference...")

    print("\nModel inference configuration parameters:\n")
    print("{:<25} {:<15}".format("Parameter", "Value"))
    print("-" * 40)
    for attr, value in vars(cfg).items():
        # Convert numpy arrays to lists for printing
        if isinstance(value, np.ndarray):
            value = value.tolist()
        print("{:<25} {:<15}".format(attr, str(value)))
    print()

    action = np.zeros((cfg.num_actions), dtype=np.double)

    hist_obs = deque()
    for _ in range(cfg.frame_stack):
        hist_obs.append(np.zeros([1, cfg.num_single_obs], dtype=np.double))

    target_frequency = 1 / (cfg.dt * cfg.decimation)  # e.g., 10 Hz
    target_loop_time = 1.0 / target_frequency

    last_time = time.time()  # Track cycle time
    t_start = time.time()  # Start time in seconds
    t = 0.0  # in seconds

    # Get joint names
    joint_names = [joint.name for joint in robot.joints]

    while not stop_event.is_set():
        loop_start_time = time.time()
        t = time.time() - t_start  # Calculate elapsed time since start

        current_time = time.time()
        cycle_time = current_time - last_time
        actual_frequency = 1.0 / cycle_time if cycle_time > 0 else 0
        last_time = current_time

        # Get current positions and velocities in degrees
        feedback_positions = robot.get_feedback_positions()  # Dict[str, float] in degrees
        feedback_velocities = robot.get_feedback_velocities()  # Dict[str, float] in degrees/s

        # Convert positions and velocities to radians
        current_positions_np = np.radians(
            np.array([feedback_positions[name] for name in joint_names], dtype=np.float32)
        )
        current_velocities_np = np.radians(
            np.array([feedback_velocities[name] for name in joint_names], dtype=np.float32)
        )

        # Use only leg joints for policy input
        positions_leg = current_positions_np[: cfg.num_actions]
        velocities_leg = current_velocities_np[: cfg.num_actions]

        # Mock IMU data
        omega = np.zeros(3, dtype=np.float32)
        eu_ang = np.zeros(3, dtype=np.float32)

        obs = np.zeros([1, cfg.num_single_obs], dtype=np.float32)

        obs[0, 0] = math.sin(2 * math.pi * t / cfg.cycle_time)
        obs[0, 1] = math.cos(2 * math.pi * t / cfg.cycle_time)
        obs[0, 2] = cmd.vx * cfg.lin_vel
        obs[0, 3] = cmd.vy * cfg.lin_vel
        obs[0, 4] = cmd.dyaw * cfg.ang_vel
        obs[0, 5 : cfg.num_actions + 5] = positions_leg * cfg.dof_pos
        obs[0, cfg.num_actions + 5 : 2 * cfg.num_actions + 5] = velocities_leg * cfg.dof_vel
        obs[0, 2 * cfg.num_actions + 5 : 3 * cfg.num_actions + 5] = action
        obs[0, 3 * cfg.num_actions + 5 : 3 * cfg.num_actions + 5 + 3] = omega
        obs[0, 3 * cfg.num_actions + 5 + 3 : 3 * cfg.num_actions + 5 + 6] = eu_ang
        obs = np.clip(obs, -cfg.clip_observations, cfg.clip_observations)

        hist_obs.append(obs)
        hist_obs.popleft()

        # Prepare policy input
        policy_input = np.zeros([1, cfg.num_observations], dtype=np.float32)
        for i in range(cfg.frame_stack):
            start = i * cfg.num_single_obs
            end = (i + 1) * cfg.num_single_obs
            policy_input[0, start:end] = hist_obs[i][0, :]

        # Run policy inference
        ort_inputs = {policy.get_inputs()[0].name: policy_input}
        action[:] = policy.run(None, ort_inputs)[0][0]  # action in radians

        action = np.clip(action, -cfg.clip_actions, cfg.clip_actions)
        scaled_action = action * cfg.action_scale

        full_action = np.zeros(len(robot.joints), dtype=np.float32)
        full_action[: cfg.num_actions] = scaled_action  # Leg actions

        # Convert 'full_action' from radians to degrees
        full_action_deg = np.degrees(full_action)

        # Prepare positions dictionary
        desired_positions_dict = {
            name: position for name, position in zip(joint_names, full_action_deg)
        }

        # Set desired positions
        robot.set_desired_positions(desired_positions_dict)

        loop_end_time = time.time()
        loop_duration = loop_end_time - loop_start_time
        sleep_time = max(0, target_loop_time - loop_duration)

        # Send data to dashboard via multiprocessing Queue
        try:
            # Send inference speed data
            data_queue.put(("frequency", (current_time, actual_frequency)))

            # Send positions data
            data_queue.put(
                (
                    "positions",
                    (current_time, np.degrees(current_positions_np), list(desired_positions_dict.values())),
                )
            )

            # Send velocities data
            data_queue.put(("velocities", (current_time, np.degrees(current_velocities_np))))
        except Exception as e:
            print(f"Exception in sending data: {e}")

        if stop_event.is_set():
            break  
        time.sleep(sleep_time)

    print("[INFO]: Inference stopped.")

    robot.set_desired_positions({joint.name: 0.0 for joint in robot.joints})


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument(
        "--model_path", type=str, required=True, help="examples/standing_micro.onnx"
    )
    args = parser.parse_args()

    robot = Robot()
    robot.initialize()

    policy = ort.InferenceSession(args.model_path)

    data_queue = run_dashboard()

    stop_event = threading.Event()

    inference(policy, robot, data_queue, stop_event)
