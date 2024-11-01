""" Inference script for running the model on the robot
Run:
    python inference/main.py --model_path sim/examples/standing_micro.onnx --no-calibration

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
from ahrs.filters import Madgwick
from ahrs.common import orientation

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
    JointData(name="left_hip_yaw", policy_index=1, servo_id=9, offset_deg=0.0),
    JointData(name="left_hip_roll", policy_index=2, servo_id=8, offset_deg=0.0),
    JointData(name="left_knee_pitch", policy_index=3, servo_id=7, offset_deg=0.0),
    JointData(name="left_ankle_pitch", policy_index=4, servo_id=6, offset_deg=0.0),
    JointData(name="right_hip_pitch", policy_index=5, servo_id=5, offset_deg=0.0),
    JointData(name="right_hip_yaw", policy_index=6, servo_id=4, offset_deg=0.0),
    JointData(name="right_hip_roll", policy_index=7, servo_id=3, offset_deg=0.0),
    JointData(name="right_knee_pitch", policy_index=8, servo_id=2, offset_deg=0.0),
    JointData(name="right_ankle_pitch", policy_index=9, servo_id=1, offset_deg=0.0),
]


def get_servo_states(hal: HAL) -> None:
    """
    input -> get servo states (degrees)
    output -> current positions (radians)
    """
    if MOCK:
        for joint in joints:
            joint.current_position = 0.0
            joint.current_velocity = 0.0
        return
    
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
    """
    input -> desired positions (radians)
    output -> set servo positions (degrees)
    """
    positions_deg = []
    for joint in joints:
        # Convert from radians to degrees and apply offset
        desired_pos_deg = math.degrees(joint.desired_position)
        desired_pos_deg += joint.offset_deg
        positions_deg.append((joint.servo_id, desired_pos_deg))

    if MOCK:
        return

    hal.servo.set_positions(positions_deg)


def init_madgwick():
    # Initialize Madgwick filter with increased beta for accelerometer weight
    return Madgwick(frequency=100.0, beta=0.7)


def imu_data_to_numpy(imu_data):
    """
    Converts IMU data to numpy arrays for gyro and accelerometer.
    """
    gyro = imu_data['gyro']
    accel = imu_data['accel']
    # Convert to proper units: gyro to rad/s, accel to g
    gyro_array = np.array([gyro['x'], gyro['y'], gyro['z']]) * np.pi / 180.0  # deg/s to rad/s
    accel_array = np.array([accel['x'], accel['y'], accel['z']]) / 1000.0     # mg to g
    return gyro_array, accel_array


class IMUHandler:
    def __init__(self, imu, madgwick_filter):
        self.imu = imu
        self.madgwick = madgwick_filter
        self.Q = np.array([1.0, 0.0, 0.0, 0.0])  
        self.gyro_bias = np.zeros(3)

    def calibrate_gyro(self, samples=100):
        """
        Calibrates the gyro bias by averaging over a number of samples.
        Also initializes the orientation using initial accelerometer readings.
        """
        print("Calibrating gyro and initializing orientation, keep IMU still...")
        bias_sum = np.zeros(3)
        accel_sum = np.zeros(3)
        
        for _ in range(samples):
            imu_data = self.imu.get_data()
            gyro, accel = imu_data_to_numpy(imu_data)
            bias_sum += gyro
            accel_sum += accel
            time.sleep(0.02)
        
        self.gyro_bias = bias_sum / samples
        
        avg_accel = accel_sum / samples
        avg_accel = avg_accel / np.linalg.norm(avg_accel)
        
        roll = np.arctan2(avg_accel[1], avg_accel[2])
        pitch = np.arctan2(-avg_accel[0], np.sqrt(avg_accel[1]**2 + avg_accel[2]**2))
        
        cr, cp = np.cos(roll/2), np.cos(pitch/2)
        sr, sp = np.sin(roll/2), np.sin(pitch/2)
        
        self.Q = np.array([
            cr * cp,                 # w
            sr * cp,                 # x
            cr * sp,                 # y
            -sr * sp                 # z
        ])
        
        print(f"Initial roll: {np.degrees(roll):.1f}, pitch: {np.degrees(pitch):.1f}")

    def get_orientation(self):
        """
        Retrieves the current orientation using the Madgwick filter.
        """
        imu_data = self.imu.get_data()
        gyro, accel = imu_data_to_numpy(imu_data)
        gyro -= self.gyro_bias
        self.Q = self.madgwick.updateIMU(self.Q, gyr=gyro, acc=accel)
        euler_angles = np.degrees(orientation.q2euler(self.Q))
        return gyro, accel, euler_angles, self.Q  # gyro in rad/s, accel in g, euler_angles in degrees, quaternion


def inference(policy: ort.InferenceSession, hal: HAL, cfg: Sim2simCfg, data_queue: mp.Queue, imu_handler: 'IMUHandler') -> None:
    print(f"[INFO]: Inference starting...")

    action = np.zeros((cfg.num_actions), dtype=np.double)

    hist_obs = deque()
    for _ in range(cfg.frame_stack):
        hist_obs.append(np.zeros([1, cfg.num_single_obs], dtype=np.double))

    target_frequency = 1 / (cfg.dt * cfg.decimation)  # e.g., 50 Hz
    target_loop_time = 1.0 / target_frequency

    last_time = time.time()  # Track cycle time

    while True:
        loop_start_time = time.time()

        current_time = time.time()
        cycle_time = current_time - last_time
        actual_frequency = 1.0 / cycle_time if cycle_time > 0 else 0
        last_time = current_time

        get_servo_states(hal)

        current_positions_np = np.array([joint.current_position for joint in joints], dtype=np.float32)
        current_velocities_np = np.array([joint.current_velocity for joint in joints], dtype=np.float32)

        gyro, accel, euler_angles, quaternions = imu_handler.get_orientation()

        # FIXME
        # omega = gyro.astype(np.float32) 
        # eu_ang = np.radians(euler_angles).astype(np.float32) 
       
        omega = np.zeros(3, dtype=np.float32)
        eu_ang = np.zeros(3, dtype=np.float32)

        obs = np.zeros([1, cfg.num_single_obs], dtype=np.float32)

        t = loop_start_time
        obs[0, 0] = math.sin(2 * math.pi * t / cfg.cycle_time)
        obs[0, 1] = math.cos(2 * math.pi * t / cfg.cycle_time)
        obs[0, 2] = cmd.vx * cfg.lin_vel
        obs[0, 3] = cmd.vy * cfg.lin_vel
        obs[0, 4] = cmd.dyaw * cfg.ang_vel
        obs[0, 5 : (cfg.num_actions + 5)] = current_positions_np * cfg.dof_pos
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

        # ===== SEND DATA TO DASHBOARD =====
        try:
            data_queue.put(('frequency', (current_time, actual_frequency)))

            current_positions = [joint.current_position for joint in joints]  # in radians
            desired_positions = [joint.desired_position for joint in joints]  # in radians
            data_queue.put(('positions', (current_time, current_positions, desired_positions)))

            current_velocities = [joint.current_velocity for joint in joints]  # in radians/s
            data_queue.put(('velocities', (current_time, current_velocities)))

            data_queue.put(('imu', (current_time, gyro.tolist(), accel.tolist(), euler_angles.tolist(), quaternions.tolist())))

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
    parser.add_argument('--no-calibration', action='store_true', help='Disable gyro bias calibration')
    args = parser.parse_args()

    hal = HAL() if not MOCK else None

    if not MOCK:
        initialize(hal)

    policy = ort.InferenceSession(args.model_path)
    cfg = Sim2simCfg()

    # Initialize IMU and Madgwick filter
    imu = hal.imu if not MOCK else None
    madgwick_filter = init_madgwick()
    imu_handler = IMUHandler(imu, madgwick_filter)

    if not args.no_calibration and not MOCK:
        imu_handler.calibrate_gyro()

    # Start the dashboard process and get the data queue
    from dashboard import run_dashboard
    data_queue = run_dashboard()

    # Start the inference loop
    inference(policy, hal, cfg, data_queue, imu_handler)



