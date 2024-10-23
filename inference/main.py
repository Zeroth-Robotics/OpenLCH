""" Inference script for running the model on the robot

python inference/main.py --model_path path/to/model.onnx
"""
import argparse
import numpy as np
import onnxruntime as ort

from sim.sim2sim import Sim2simCfg, get_policy_output
import time
from collections import deque
import math


MOCK = True
if not MOCK:
    from openlch import HAL
else:
    HAL = None


class cmd:
    vx = 0.0
    vy = 0.0
    dyaw = 0.0


def get_velocities(hal : HAL) -> list:
    if MOCK:
        return [0 for _ in range(10)]
    velocities = hal.servo.get_velocities()
    velocities = [vel for _, vel in velocities[:10]]
    return velocities


def get_servo_positions(hal: HAL) -> list:
    if MOCK:
        return [0 for _ in range(10)]
    servo_positions = hal.servo.get_positions()
    positions = [pos for _, pos in servo_positions[:10]]
    print(f"[INFO]: GET servo positions: {positions}")
    return positions


def set_servo_positions(positions : list, hal : HAL) -> None:
    print(f"[INFO]: SET servo positions: {positions}")
    if MOCK:
        return
    servo_positions = [(i, pos) for i, pos in enumerate(positions[:10])]
    hal.servo.set_positions(servo_positions)


def inference(session : ort.InferenceSession, hal : HAL, cfg : Sim2simCfg) -> None:
    print(f"[INFO]: Inference starting...")

    default = np.array([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], dtype=np.double)
    target_q = np.zeros((cfg.num_actions), dtype=np.double)
    action = np.zeros((cfg.num_actions), dtype=np.double)

    hist_obs = deque()
    for _ in range(cfg.frame_stack):
        hist_obs.append(np.zeros([1, cfg.num_single_obs], dtype=np.double))


    target_frequency = 1 / (cfg.dt * cfg.decimation)  # 100 Hz
    target_loop_time = 1.0 / target_frequency  # 4 ms

    while True:
        t = time.time()
        loop_start_time = time.time()

        # get current positions
        current_positions = get_servo_positions(hal)
        current_positions_np = np.array(current_positions, dtype=np.float32)

        # compute velocity
        current_velocities = get_velocities(hal)
        current_velocities_np = np.array(current_velocities, dtype=np.float32)

        # IMU mock
        omega = np.zeros(3, dtype=np.float32)
        eu_ang = np.zeros(3, dtype=np.float32)  

        obs = np.zeros([1, cfg.num_single_obs], dtype=np.float32)

        # pfb30 - double check the time flow
        obs[0, 0] = math.sin(2 * math.pi * t / cfg.cycle_time)
        obs[0, 1] = math.cos(2 * math.pi * t / cfg.cycle_time)
        obs[0, 2] = cmd.vx * cfg.lin_vel
        obs[0, 3] = cmd.vy * cfg.lin_vel
        obs[0, 4] = cmd.dyaw * cfg.ang_vel
        obs[0, 5 : (cfg.num_actions + 5)] = (current_positions_np - default) * cfg.dof_pos
        obs[0, (cfg.num_actions + 5) : (2 * cfg.num_actions + 5)] = current_velocities_np * cfg.dof_vel
        obs[0, (2 * cfg.num_actions + 5) : (3 * cfg.num_actions + 5)] = action
        obs[0, (3 * cfg.num_actions + 5) : (3 * cfg.num_actions + 5) + 3] = omega
        obs[0, (3 * cfg.num_actions + 5) + 3 : (3 * cfg.num_actions + 5) + 2 * 3] = eu_ang
        obs = np.clip(obs, -cfg.clip_observations, cfg.clip_observations)

        hist_obs.append(obs)
        hist_obs.popleft()

        policy_input = np.zeros([1, cfg.num_observations], dtype=np.float32)
        for i in range(cfg.frame_stack):
            policy_input[0, i * cfg.num_single_obs : (i + 1) * cfg.num_single_obs] = hist_obs[i][0, :]

        action[:] = get_policy_output(policy, policy_input)

        action = np.clip(action, -cfg.clip_actions, cfg.clip_actions)
        target_q = action * cfg.action_scale

        command_positions = target_q.tolist()

        set_servo_positions(command_positions, hal)

        # Calculate how long to sleep
        loop_end_time = time.time()
        loop_duration = loop_end_time - loop_start_time
        sleep_time = max(0, target_loop_time - loop_duration)

        time.sleep(sleep_time)
        print("Sleep time: ", sleep_time)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompypro")
    parser.add_argument("--model_path", type=str, required=True, help="examples/stompypro.onnx")
    args = parser.parse_args()

    hal = HAL() if not MOCK else None

    policy = ort.InferenceSession(args.model_path)

    cfg = Sim2simCfg(args.embodiment)

    inference(policy, hal, cfg)

