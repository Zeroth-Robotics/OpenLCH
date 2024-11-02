"""Model configuration and inference module."""

import time
import math
from collections import deque
from typing import Deque

import numpy as np
import onnxruntime as ort

from robot import Robot
from plot import run_plot


class Sim2simCfg:
    """Configuration for the simulation."""
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


class Command:
    """Represents a walking command."""
    def __init__(self):
        self.vx = 0.0
        self.vy = 0.0
        self.dyaw = 0.0


def inference_loop(
    policy: ort.InferenceSession,
    robot: Robot
) -> None:
    """Main inference loop for walking."""
    logger = logging.getLogger(__name__)
    logger.info("Starting inference loop...")

    # Initialize configuration, command, and data queue
    cfg = Sim2simCfg()
    command = Command()
    data_queue = mp.Queue()

    # Start the plotting process
    plot_process = mp.Process(target=run_plot, args=(data_queue,))
    plot_process.start()

    action = np.zeros((cfg.num_actions), dtype=np.float32)
    hist_obs: Deque[np.ndarray] = deque(
        [np.zeros([1, cfg.num_single_obs], dtype=np.float32) for _ in range(cfg.frame_stack)],
        maxlen=cfg.frame_stack
    )

    target_frequency = 1 / (cfg.dt * cfg.decimation)  # e.g., 50 Hz
    target_loop_time = 1.0 / target_frequency

    last_time = time.time()  # Track cycle time

    try:
        while True:
            loop_start_time = time.time()

            current_time = time.time()
            cycle_time = current_time - last_time
            actual_frequency = 1.0 / cycle_time if cycle_time > 0 else 0
            last_time = current_time

            # Update servo states
            robot.get_servo_states()

            # Get current joint positions and velocities
            current_positions_np = np.array(
                robot.get_joint_positions(), dtype=np.float32
            )
            current_velocities_np = np.array(
                robot.get_joint_velocities(), dtype=np.float32
            )

            # Get IMU data
            gyro, accel, euler_angles, quaternions = robot.imu_handler.get_orientation()

            # Prepare observation
            obs = np.zeros([1, cfg.num_single_obs], dtype=np.float32)

            t = loop_start_time
            obs[0, 0] = math.sin(2 * math.pi * t / cfg.cycle_time)
            obs[0, 1] = math.cos(2 * math.pi * t / cfg.cycle_time)
            obs[0, 2] = command.vx * cfg.lin_vel
            obs[0, 3] = command.vy * cfg.lin_vel
            obs[0, 4] = command.dyaw * cfg.ang_vel
            obs[0, 5 : (cfg.num_actions + 5)] = current_positions_np * cfg.dof_pos
            obs[0, (cfg.num_actions + 5) : (2 * cfg.num_actions + 5)] = current_velocities_np * cfg.dof_vel
            obs[0, (2 * cfg.num_actions + 5) : (2 * cfg.num_actions + 5) + cfg.num_actions] = action
            obs[0, (3 * cfg.num_actions + 5) : (3 * cfg.num_actions + 5) + 3] = gyro
            obs[0, (3 * cfg.num_actions + 5) + 3 : (3 * cfg.num_actions + 5) + 6] = np.radians(euler_angles)
            obs = np.clip(obs, -cfg.clip_observations, cfg.clip_observations)

            hist_obs.append(obs)

            policy_input = np.concatenate(list(hist_obs), axis=1)
            ort_inputs = {policy.get_inputs()[0].name: policy_input}
            action = policy.run(None, ort_inputs)[0][0]

            action = np.clip(action, -cfg.clip_actions, cfg.clip_actions)
            scaled_action = action * cfg.action_scale

            # Update desired joint positions
            robot.set_servo_positions(scaled_action)

            loop_end_time = time.time()
            loop_duration = loop_end_time - loop_start_time
            sleep_time = max(0, target_loop_time - loop_duration)

            try:
                data_queue.put(('frequency', (current_time, actual_frequency)))

                current_positions = robot.get_joint_positions()
                desired_positions = scaled_action.tolist()
                data_queue.put(('positions', (current_time, current_positions, desired_positions)))

                current_velocities = robot.get_joint_velocities()
                data_queue.put(('velocities', (current_time, current_velocities)))

                data_queue.put((
                    'imu',
                    (
                        current_time,
                        gyro.tolist(),
                        accel.tolist(),
                        euler_angles.tolist(),
                        quaternions.tolist()
                    )
                ))
            except Exception as e:
                logger.error(f"Exception in sending data: {e}")

            time.sleep(sleep_time)
    except KeyboardInterrupt:
        logger.info("Inference loop interrupted by user.")
    finally:
        plot_process.terminate()
        plot_process.join()
        data_queue.close()


