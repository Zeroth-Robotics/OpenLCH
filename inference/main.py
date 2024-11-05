""" Inference script for running the model on the robot
Run:
    python inference/main.py --model_path sim/examples/walking_micro.onnx
"""
import argparse
import math
import time
from collections import deque
from typing import List
import numpy as np
import onnxruntime as ort
import multiprocessing as mp
from robot import Robot
from dashboard import run_dashboard


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
        decimation=20,
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
        self.tau_limit = (
            np.array([effort] * self.num_actions) * self.tau_factor
        )
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
    vx = 0.0
    vy = 0.0
    dyaw = 0.0


def inference(policy: ort.InferenceSession, robot: Robot, cfg: Sim2simCfg, data_queue: mp.Queue) -> None:
    print(f"[INFO]: Inference starting...")

    action = np.zeros((cfg.num_actions), dtype=np.double)

    hist_obs = deque()
    for _ in range(cfg.frame_stack):
        hist_obs.append(np.zeros([1, cfg.num_single_obs], dtype=np.double))

    target_frequency = 1 / (cfg.dt * cfg.decimation)  # e.g., 50 Hz
    # print(f"Target frequency: {target_frequency} Hz")
    target_loop_time = 1.0 / target_frequency

    last_time = time.time()  # Track cycle time

    while True:
        loop_start_time = time.time()

        current_time = time.time()
        cycle_time = current_time - last_time
        actual_frequency = 1.0 / cycle_time if cycle_time > 0 else 0
        # print(f"Actual frequency: {actual_frequency:.2f} Hz (cycle time: {cycle_time*1000:.2f} ms)")
        last_time = current_time

        robot.get_servo_states()

        current_positions_np = np.array(robot.get_current_positions(), dtype=np.float32)
        current_velocities_np = np.array(robot.get_current_velocities(), dtype=np.float32)

        # Mock IMU data
        # FIXME
        omega = np.zeros(3, dtype=np.float32)
        eu_ang = np.zeros(3, dtype=np.float32)  

        obs = np.zeros([1, cfg.num_single_obs], dtype=np.float32)

        t = loop_start_time
        obs[0, 0] = math.sin(2 * math.pi * t / cfg.cycle_time)
        obs[0, 1] = math.cos(2 * math.pi * t / cfg.cycle_time)
        obs[0, 2] = cmd.vx * cfg.lin_vel
        obs[0, 3] = cmd.vy * cfg.lin_vel
        obs[0, 4] = cmd.dyaw * cfg.ang_vel
        obs[0, 5 : (cfg.num_actions + 5)] = (current_positions_np) * cfg.dof_pos 
        obs[0, (cfg.num_actions + 5) : (2 * cfg.num_actions + 5)] = current_velocities_np * cfg.dof_vel
        obs[0, (2 * cfg.num_actions + 5) : (3 * cfg.num_actions + 5)] = action
        obs[0, (3 * cfg.num_actions + 5) : (3 * cfg.num_actions + 5) + 3] = omega
        obs[0, (3 * cfg.num_actions + 5) + 3 : (3 * cfg.num_actions + 5) + 2 * 3] = eu_ang
        obs = np.clip(obs, -cfg.clip_observations, cfg.clip_observations)

        hist_obs.append(obs)
        hist_obs.popleft()

        policy_input = np.zeros([1, cfg.num_observations], dtype=np.float32)
        for i in range(cfg.frame_stack):
            start = i * cfg.num_single_obs
            end = (i + 1) * cfg.num_single_obs
            policy_input[0, start:end] = hist_obs[i][0, :]

        ort_inputs = {policy.get_inputs()[0].name: policy_input}
        action[:] = policy.run(None, ort_inputs)[0][0]

        action = np.clip(action, -cfg.clip_actions, cfg.clip_actions)
        scaled_action = action * cfg.action_scale

        robot.set_joint_positions(scaled_action)

        robot.set_servo_positions()

        loop_end_time = time.time()
        loop_duration = loop_end_time - loop_start_time
        sleep_time = max(0, target_loop_time - loop_duration)

        # Send data to dashboard via multiprocessing Queue
        try:
            # Send inference speed data
            data_queue.put(('frequency', (current_time, actual_frequency)))

            # Send positions data
            current_positions = robot.get_current_positions()
            desired_positions = [joint.desired_position + math.radians(joint.offset_deg) for joint in robot.joints]  # in radians
            data_queue.put(('positions', (current_time, current_positions, desired_positions)))

            # Send velocities data
            current_velocities = robot.get_current_velocities()
            data_queue.put(('velocities', (current_time, current_velocities)))
        except Exception as e:
            print(f"Exception in sending data: {e}")

        time.sleep(sleep_time)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument("--model_path", type=str, required=True, help="examples/standing_micro.onnx")
    args = parser.parse_args()

    robot = Robot()
    robot.initialize()

    policy = ort.InferenceSession(args.model_path)
    cfg = Sim2simCfg()

    data_queue = run_dashboard()

    inference(policy, robot, cfg, data_queue)



