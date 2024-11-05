""" Inference script for running the model on the robot
Run:
    python inference/main.py --model_path sim/examples/walking_micro.onnx

    (from repo root)
TODO:
    - connect this with the sim2sim config
    - add sim2real
    - add real2sim
"""
import argparse
import math
import time
from collections import deque
from typing import List
import numpy as np
import onnxruntime as ort
import multiprocessing as mp

MOCK = False 

if not MOCK:
    from openlch import HAL
else:
    HAL = None


class JointData:
    def __init__(self, name: str, policy_index: int, servo_id: int, offset_deg: float = 0.0):
        self.name = name
        self.policy_index = policy_index
        self.servo_id = servo_id
        self.current_position = 0.0      # in radians
        self.desired_position = 0.0      # in radians
        self.current_velocity = 0.0      # in radians/s
        self.offset_deg = offset_deg     # in degrees


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


joints = [
    JointData(name="left_hip_pitch", policy_index=0, servo_id=10, offset_deg=0.0),
    JointData(name="left_hip_yaw", policy_index=1, servo_id=9, offset_deg=45.0),
    JointData(name="left_hip_roll", policy_index=2, servo_id=8, offset_deg=0.0),
    JointData(name="left_knee_pitch", policy_index=3, servo_id=7, offset_deg=0.0),
    JointData(name="left_ankle_pitch", policy_index=4, servo_id=6, offset_deg=0.0),
    JointData(name="right_hip_pitch", policy_index=5, servo_id=5, offset_deg=0.0),
    JointData(name="right_hip_yaw", policy_index=6, servo_id=4, offset_deg=-45.0),
    JointData(name="right_hip_roll", policy_index=7, servo_id=3, offset_deg=0.0),
    JointData(name="right_knee_pitch", policy_index=8, servo_id=2, offset_deg=0.0),
    JointData(name="right_ankle_pitch", policy_index=9, servo_id=1, offset_deg=0.0),
]


def get_servo_states(hal: HAL) -> None:
    if MOCK:
        for joint in joints:
            joint.current_position = 0.0
            joint.current_velocity = 0.0
        return
    
    # Placeholder for actual servo state retrieval
    servo_positions = hal.servo.get_positions() 
    servo_positions_dict = {id_: (pos, vel) for id_, pos, vel in servo_positions}
    
    for joint in joints:
        if joint.servo_id in servo_positions_dict:
            pos_deg, vel_deg_s = servo_positions_dict[joint.servo_id]
            joint.current_position = math.radians(pos_deg)
            joint.current_velocity = math.radians(vel_deg_s)
        else:
            joint.current_position = 0.0
            joint.current_velocity = 0.0


def set_servo_positions(hal: HAL) -> None:
    positions_deg = []
    for joint in joints:
        # convert from radians to degrees
        desired_pos_deg = math.degrees(joint.desired_position) + joint.offset_deg
        positions_deg.append((joint.servo_id, desired_pos_deg))

    # print(f"[INFO]: SET servo positions (deg): {positions_deg}")

    if MOCK:
        return

    hal.servo.set_positions(positions_deg)


def inference(policy: ort.InferenceSession, hal: HAL, cfg: Sim2simCfg, data_queue: mp.Queue) -> None:
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

        get_servo_states(hal)

        current_positions_np = np.array([joint.current_position for joint in joints], dtype=np.float32)
        current_velocities_np = np.array([joint.current_velocity for joint in joints], dtype=np.float32)

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

        for joint in joints:
            joint.desired_position = scaled_action[joint.policy_index]

        set_servo_positions(hal)

        loop_end_time = time.time()
        loop_duration = loop_end_time - loop_start_time
        sleep_time = max(0, target_loop_time - loop_duration)

        # Send data to dashboard via multiprocessing Queue
        try:
            # Send inference speed data
            data_queue.put(('frequency', (current_time, actual_frequency)))

            # Send positions data
            current_positions = [joint.current_position for joint in joints]
            desired_positions = [joint.desired_position + math.radians(joint.offset_deg) for joint in joints]  # in radians
            data_queue.put(('positions', (current_time, current_positions, desired_positions)))

            # Send velocities data
            current_velocities = [joint.current_velocity for joint in joints]
            data_queue.put(('velocities', (current_time, current_velocities)))
        except Exception as e:
            print(f"Exception in sending data: {e}")

        time.sleep(sleep_time)


def initialize(hal: HAL) -> None:
    if MOCK:
        return

    hal.servo.scan()

    hal.servo.set_torque_enable([(joint.servo_id, True) for joint in joints])
    time.sleep(1)
    hal.servo.set_torque([(joint.servo_id, 30.0) for joint in joints])
    time.sleep(1)

    hal.servo.disable_movement() 

    for joint in joints:
        joint.desired_position = 0.0  # in radians

    set_servo_positions(hal)
    time.sleep(3)

    print("Starting inference in...")
    for i in range(3, 0, -1):
        print(f"{i}...")
        time.sleep(0.5)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument("--model_path", type=str, required=True, help="examples/standing_micro.onnx")
    args = parser.parse_args()

    hal = HAL() if not MOCK else None

    if not MOCK:
        initialize(hal)

    policy = ort.InferenceSession(args.model_path)
    cfg = Sim2simCfg()

    # Start the dashboard process and get the data queue
    from dashboard import run_dashboard
    data_queue = run_dashboard()

    # Start the inference loop
    inference(policy, hal, cfg, data_queue)



