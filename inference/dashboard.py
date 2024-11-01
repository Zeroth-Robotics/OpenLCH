import os
import multiprocessing as mp
import matplotlib.pyplot as plt
import matplotlib.animation as animation
import numpy as np
import time

def plot_dashboard(data_queue: mp.Queue):
    # Joint names and colors
    joint_names = [
        "left_hip_pitch", "left_hip_yaw", "left_hip_roll", "left_knee_pitch",
        "left_ankle_pitch", "right_hip_pitch", "right_hip_yaw", "right_hip_roll",
        "right_knee_pitch", "right_ankle_pitch"
    ]
    num_joints = len(joint_names)
    joint_colors = plt.cm.get_cmap('tab10', num_joints)

    # Initialize data storage with 5-second window
    max_time_window = 5.0  # Changed from 1.0 to 5.0
    time_data = []
    freq_data = []
    position_time = []
    positions = []
    desired_positions = []
    velocity_time = []
    velocities = []

    # IMU data storage
    imu_time = []
    euler_data = [[], [], []]      # For Roll, Pitch, Yaw
    gyro_data = [[], [], []]       # For Gyro X, Y, Z
    accel_data = [[], [], []]      # For Accel X, Y, Z
    quat_data = [[], [], [], []]   # For q0, q1, q2, q3

    # Directory to save snapshots
    snapshots_dir = os.path.join(os.getcwd(), 'inference/experiments')
    if not os.path.exists(snapshots_dir):
        os.makedirs(snapshots_dir)

    # Time of the last snapshot
    last_snapshot_time = time.time()

    # Create figures
    fig_freq = plt.figure(figsize=(12, 3))
    fig_pos = plt.figure(figsize=(12, 6))
    fig_vel = plt.figure(figsize=(12, 6))
    fig_imu, (ax_euler, ax_raw, ax_quat) = plt.subplots(1, 3, figsize=(20, 5))

    # Graph 1: Inference speed
    ax_freq = fig_freq.add_subplot(111)
    ax_freq.set_title('Inference Speed Over Time')
    ax_freq.set_xlabel('Time (s)')
    ax_freq.set_ylabel('Frequency (Hz)')
    ax_freq.axhline(y=50, color='r', linestyle='--', label='50 Hz Reference')
    line_freq, = ax_freq.plot([], [], label='Actual Frequency')
    ax_freq.legend()

    # Graph 2: Joint Positions with dual y-axes
    ax_pos = fig_pos.add_subplot(111)
    ax_pos_deg = ax_pos.twinx()  # Create twin axis for degrees
    ax_pos.set_title('Joint Positions Over Time')
    ax_pos.set_xlabel('Time (s)')
    ax_pos.set_ylabel('Position (rad)')
    ax_pos_deg.set_ylabel('Position (deg)')
    lines_positions = []
    for i in range(num_joints):
        color = joint_colors(i)
        ax_pos.plot([], [], label=f"{joint_names[i]} Actual", color=color)
        ax_pos.plot([], [], linestyle='--', label=f"{joint_names[i]} Desired", color=color)

    ax_pos.legend(loc='center left', bbox_to_anchor=(1.02, 0.5))

    # Graph 3: Joint Velocities with dual y-axes
    ax_vel = fig_vel.add_subplot(111)
    ax_vel_deg = ax_vel.twinx()  # Create twin axis for degrees/s
    ax_vel.set_title('Joint Velocities Over Time')
    ax_vel.set_xlabel('Time (s)')
    ax_vel.set_ylabel('Velocity (rad/s)')
    ax_vel_deg.set_ylabel('Velocity (deg/s)')
    for i in range(num_joints):
        color = joint_colors(i)
        ax_vel.plot([], [], label=f"{joint_names[i]}", color=color)
    ax_vel.legend(loc='center left', bbox_to_anchor=(1.02, 0.5))

    # IMU Plots
    # Euler angles plot
    lines_euler = [ax_euler.plot([], [], label=angle)[0] for angle in ['Roll', 'Pitch', 'Yaw']]
    ax_euler.legend()
    ax_euler.set_ylim(-180, 180)
    ax_euler.set_xlabel('Time (s)')
    ax_euler.set_ylabel('Angle (degrees)')
    ax_euler.set_title('Euler Angles')

    # Raw IMU data plot
    lines_gyro = [ax_raw.plot([], [], label=f'Gyro {axis}')[0] for axis in ['X', 'Y', 'Z']]
    lines_accel = [ax_raw.plot([], [], '--', label=f'Accel {axis}')[0] for axis in ['X', 'Y', 'Z']]
    ax_raw.legend()
    ax_raw.set_xlabel('Time (s)')
    ax_raw.set_ylabel('Value')
    ax_raw.set_title('Raw IMU Data')
    ax_raw.set_ylim(-2, 2)  # Gyro in rad/s, Accel in g

    # Quaternion plot
    lines_quat = [ax_quat.plot([], [], label=f'q{i}')[0] for i in range(4)]
    ax_quat.legend()
    ax_quat.set_ylim(-1.1, 1.1)
    ax_quat.set_xlabel('Time (s)')
    ax_quat.set_ylabel('Value')
    ax_quat.set_title('Quaternions')

    def rad2deg(rad):
        return rad * 180.0 / np.pi

    def update(frame):
        nonlocal last_snapshot_time  # Added to modify the variable within the function
        current_time = time.time()

        # Remove old data outside the time window
        def prune_data(time_array, *data_arrays):
            while time_array and (current_time - time_array[0]) > max_time_window:
                time_array.pop(0)
                for data_array in data_arrays:
                    data_array.pop(0)

        # Prune old data
        prune_data(time_data, freq_data)
        prune_data(position_time, positions, desired_positions)
        prune_data(velocity_time, velocities)
        prune_data(imu_time, *gyro_data, *accel_data, *euler_data, *quat_data)

        # Get new data from queue
        while not data_queue.empty():
            try:
                data_type, data = data_queue.get_nowait()
                if data_type == 'frequency':
                    t, freq = data
                    time_data.append(t)
                    freq_data.append(freq)
                elif data_type == 'positions':
                    t, pos, desired_pos = data
                    position_time.append(t)
                    positions.append(pos)
                    desired_positions.append(desired_pos)
                elif data_type == 'velocities':
                    t, vel = data
                    velocity_time.append(t)
                    velocities.append(vel)
                elif data_type == 'imu':
                    t, gyro_list, accel_list, euler_list, quat_list = data
                    imu_time.append(t)
                    # Append gyro data
                    for i in range(3):
                        gyro_data[i].append(gyro_list[i])
                    # Append accel data
                    for i in range(3):
                        accel_data[i].append(accel_list[i])
                    # Append euler angles data
                    for i in range(3):
                        euler_data[i].append(euler_list[i])
                    # Append quaternion data
                    for i in range(4):
                        quat_data[i].append(quat_list[i])
            except Exception as e:
                print(f"Error processing queue data: {e}")

        # Update frequency plot
        if time_data and freq_data:
            ax_freq.clear()
            ax_freq.set_title('Inference Speed Over Time')
            ax_freq.set_xlabel('Time (s)')
            ax_freq.set_ylabel('Frequency (Hz)')
            ax_freq.axhline(y=50, color='r', linestyle='--', label='50 Hz Reference')
            ax_freq.plot(time_data, freq_data, label='Actual Frequency')
            ax_freq.legend()
            ax_freq.set_xlim(current_time - max_time_window, current_time)
            ax_freq.set_ylim(0, 60)  # Fixed y-axis limits for frequency

        # Update positions plot
        if position_time and positions and desired_positions:
            ax_pos.clear()
            ax_pos_deg.clear()
            ax_pos.set_title('Joint Positions Over Time')
            ax_pos.set_xlabel('Time (s)')
            ax_pos.set_ylabel('Position (rad)')
            ax_pos_deg.set_ylabel('Position (deg)')
            
            # Set fixed y-axis limits instead of calculating from data
            y_min_deg = -100
            y_max_deg = 100
            y_min_rad = np.deg2rad(y_min_deg)  # Convert to radians
            y_max_rad = np.deg2rad(y_max_deg)  # Convert to radians
            
            for i in range(num_joints):
                color = joint_colors(i)
                joint_actual_positions = [pos[i] for pos in positions]
                joint_desired_positions = [dpos[i] for dpos in desired_positions]
                
                ax_pos.plot(position_time, joint_actual_positions, 
                          label=f"{joint_names[i]} Actual", color=color)
                ax_pos.plot(position_time, joint_desired_positions, 
                          linestyle='--', label=f"{joint_names[i]} Desired", color=color)
            
            # Set fixed limits for both axes
            ax_pos.set_xlim(current_time - max_time_window, current_time)
            ax_pos.set_ylim(y_min_rad, y_max_rad)
            ax_pos_deg.set_ylim(y_min_deg, y_max_deg)

        # Update velocities plot
        if velocity_time and velocities:
            ax_vel.clear()
            ax_vel_deg.clear()
            ax_vel.set_title('Joint Velocities Over Time')
            ax_vel.set_xlabel('Time (s)')
            ax_vel.set_ylabel('Velocity (rad/s)')
            ax_vel_deg.set_ylabel('Velocity (deg/s)')
            
            # Store y limits for synchronizing axes
            y_min_rad = float('inf')
            y_max_rad = float('-inf')
            
            for i in range(num_joints):
                color = joint_colors(i)
                joint_velocities = [vel[i] for vel in velocities]
                
                # Update y limits
                y_min_rad = min(y_min_rad, min(joint_velocities))
                y_max_rad = max(y_max_rad, max(joint_velocities))
                
                ax_vel.plot(velocity_time, joint_velocities, 
                          label=f"{joint_names[i]}", color=color)
            
            # Set limits and ticks for both axes
            ax_vel.set_xlim(current_time - max_time_window, current_time)
            ax_vel.set_ylim(y_min_rad, y_max_rad)
            ax_vel_deg.set_ylim(rad2deg(y_min_rad), rad2deg(y_max_rad))
            
            # Add grid
            ax_vel.grid(True, alpha=0.3)
            ax_vel.legend(loc='center left', bbox_to_anchor=(1.02, 0.5))

        # Update IMU plots
        if imu_time:
            # Update Euler angles plot
            ax_euler.clear()
            ax_euler.set_title('Euler Angles')
            ax_euler.set_xlabel('Time (s)')
            ax_euler.set_ylabel('Angle (degrees)')
            ax_euler.set_ylim(-180, 180)
            for i in range(3):
                ax_euler.plot(imu_time, euler_data[i], label=['Roll', 'Pitch', 'Yaw'][i])
            ax_euler.legend()
            ax_euler.set_xlim(current_time - max_time_window, current_time)

            # Update raw IMU data plot
            ax_raw.clear()
            ax_raw.set_title('Raw IMU Data')
            ax_raw.set_xlabel('Time (s)')
            ax_raw.set_ylabel('Value (rad/s or g)')
            ax_raw.set_ylim(-2, 2)  # Fixed y-axis limits
            for i in range(3):
                ax_raw.plot(imu_time, gyro_data[i], label=f'Gyro {["X","Y","Z"][i]}')
                ax_raw.plot(imu_time, accel_data[i], linestyle='--', label=f'Accel {["X","Y","Z"][i]}')
            ax_raw.legend()
            ax_raw.set_xlim(current_time - max_time_window, current_time)
            ax_raw.grid(True, alpha=0.3)  # Optional: add grid for better readability

            # Update quaternions plot
            ax_quat.clear()
            ax_quat.set_title('Quaternions')
            ax_quat.set_xlabel('Time (s)')
            ax_quat.set_ylabel('Value')
            ax_quat.set_ylim(-1.1, 1.1)
            for i in range(4):
                ax_quat.plot(imu_time, quat_data[i], label=f'q{i}')
            ax_quat.legend()
            ax_quat.set_xlim(current_time - max_time_window, current_time)

        # Adjust layouts
        fig_freq.tight_layout()
        fig_pos.tight_layout()
        fig_vel.tight_layout()
        fig_imu.tight_layout()

        # Add padding for legends
        fig_pos.subplots_adjust(right=0.85)
        fig_vel.subplots_adjust(right=0.85)

        # Snapshot functionality
        if current_time - last_snapshot_time >= 5.0:  # Every 5 seconds
            # Remove previous images
            for filename in os.listdir(snapshots_dir):
                file_path = os.path.join(snapshots_dir, filename)
                try:
                    if os.path.isfile(file_path):
                        os.unlink(file_path)
                except Exception as e:
                    print(f"Error deleting file {file_path}: {e}")

            # Save new snapshots
            timestamp = int(current_time)
            fig_freq.savefig(os.path.join(snapshots_dir, f'freq_{timestamp}.png'))
            fig_pos.savefig(os.path.join(snapshots_dir, f'pos_{timestamp}.png'))
            fig_vel.savefig(os.path.join(snapshots_dir, f'vel_{timestamp}.png'))
            fig_imu.savefig(os.path.join(snapshots_dir, f'imu_{timestamp}.png'))

            # Update the last snapshot time
            last_snapshot_time = current_time

    # Create animations for each figure
    ani_freq = animation.FuncAnimation(fig_freq, update, interval=100)
    ani_pos = animation.FuncAnimation(fig_pos, update, interval=100)
    ani_vel = animation.FuncAnimation(fig_vel, update, interval=100)
    ani_imu = animation.FuncAnimation(fig_imu, update, interval=100)

    plt.show()

def run_dashboard():
    data_queue = mp.Queue()
    plot_process = mp.Process(target=plot_dashboard, args=(data_queue,))
    plot_process.start()
    return data_queue

if __name__ == '__main__':
    run_dashboard()
