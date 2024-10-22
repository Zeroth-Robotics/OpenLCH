import pybullet as p
import pybullet_data
import time
import os
import sys

# Connect to PyBullet physics server
p.connect(p.GUI)

# Set gravity
p.setGravity(0, 0, 0) # -9.81

# Get the absolute path to the 'urdf' directory
current_dir = os.path.dirname(os.path.abspath(__file__))
urdf_path = os.path.join(current_dir, 'urdf')

# Set additional search path to include the 'urdf' directory
p.setAdditionalSearchPath(urdf_path)

# Set the path for PyBullet data
p.setAdditionalSearchPath(pybullet_data.getDataPath())

# Optional: Load a plane
try:
    plane_id = p.loadURDF("plane.urdf")
except p.error:
    print("Warning: Could not load plane.urdf. Continuing without it.")

# Load the robot URDF
robot_urdf_path = os.path.join(urdf_path, 'robot_fixed.urdf')

# Check if the URDF file exists
if not os.path.exists(robot_urdf_path):
    print(f"URDF file not found at: {robot_urdf_path}")
    sys.exit(1)

start_pos = [0, 0, 0.5]  # Adjust the starting position if necessary
start_orientation = p.getQuaternionFromEuler([0, 0, 0])

# Attempt to load the URDF
try:
    robot_id = p.loadURDF(robot_urdf_path, start_pos, start_orientation, useFixedBase=False)
except p.error as e:
    print(f"Failed to load URDF. Error: {e}")
    sys.exit(1)

# Get number of joints
num_joints = p.getNumJoints(robot_id)
print(f"Number of joints: {num_joints}")

# Create lists and dictionaries for joint and link indices and names
joint_names = []
joint_indices = []
controllable_joints = []
link_names = []
link_name_to_index = {}

for i in range(num_joints):
    info = p.getJointInfo(robot_id, i)
    joint_name = info[1].decode('utf-8')
    joint_type = info[2]
    link_name = info[12].decode('utf-8')
    joint_names.append(joint_name)
    joint_indices.append(i)
    link_names.append(link_name)
    link_name_to_index[link_name] = i
    if joint_type == p.JOINT_REVOLUTE:
        controllable_joints.append(i)
    print(f"Joint index: {i}, Joint Name: {joint_name}, Type: {joint_type}, Link Name: {link_name}")

# Define link indices for feet
right_foot_link_index = link_name_to_index.get("foot_right")
left_foot_link_index = link_name_to_index.get("foot_left")

# Check if the foot links were found
if right_foot_link_index is None:
    print("'foot_right' not found in link names")
    sys.exit(1)

if left_foot_link_index is None:
    print("'foot_left' not found in link names")
    sys.exit(1)

# Create sliders for controlling the target positions
right_foot_sliders = [
    p.addUserDebugParameter("Right_Foot_X", -1, 1, 0),
    p.addUserDebugParameter("Right_Foot_Y", -1, 1, -0.2),
    p.addUserDebugParameter("Right_Foot_Z", 0, 1, 0),
]

left_foot_sliders = [
    p.addUserDebugParameter("Left_Foot_X", -1, 1, 0),
    p.addUserDebugParameter("Left_Foot_Y", -1, 1, 0.2),
    p.addUserDebugParameter("Left_Foot_Z", 0, 1, 0),
]

# Simulation loop
while p.isConnected():
    # Read target positions from sliders
    right_foot_target_pos = [p.readUserDebugParameter(slider) for slider in right_foot_sliders]
    left_foot_target_pos = [p.readUserDebugParameter(slider) for slider in left_foot_sliders]

    # Compute inverse kinematics for the right foot
    right_foot_joint_angles = p.calculateInverseKinematics(
        robot_id, right_foot_link_index, right_foot_target_pos)

    # Compute inverse kinematics for the left foot
    left_foot_joint_angles = p.calculateInverseKinematics(
        robot_id, left_foot_link_index, left_foot_target_pos)

    # Apply joint angles to the robot
    # The IK functions return joint angles for all joints,
    # so we need to assign them appropriately.
    for i, joint_index in enumerate(controllable_joints):
        if i < len(right_foot_joint_angles):
            # Control the right leg joints
            p.setJointMotorControl2(robot_id, joint_index, p.POSITION_CONTROL, right_foot_joint_angles[i])
        elif i - len(right_foot_joint_angles) < len(left_foot_joint_angles):
            # Control the left leg joints
            idx = i - len(right_foot_joint_angles)
            p.setJointMotorControl2(robot_id, joint_index, p.POSITION_CONTROL, left_foot_joint_angles[idx])
        else:
            # No more joints to control
            break

    # Step the simulation
    p.stepSimulation()
    time.sleep(1./240.)
