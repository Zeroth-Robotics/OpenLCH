import pybullet as p
import pybullet_data
import time
import os
import sys

if p.connect(p.GUI) < 0:
    print("Failed to connect to PyBullet GUI")
    sys.exit(1)

p.setGravity(0, 0, 0)

current_dir = os.path.dirname(os.path.abspath(__file__))
urdf_path = os.path.join(current_dir, 'urdf')

p.setAdditionalSearchPath(urdf_path)
p.setAdditionalSearchPath(pybullet_data.getDataPath())

try:
    plane_id = p.loadURDF("plane.urdf")
except p.error:
    print("Warning: Could not load plane.urdf. Continuing without it.")

robot_urdf_path = os.path.join(urdf_path, 'robot_fixed.urdf')

if not os.path.exists(robot_urdf_path):
    print(f"URDF file not found at: {robot_urdf_path}")
    sys.exit(1)

start_pos = [0, 0, 0.5]
start_orientation = p.getQuaternionFromEuler([0, 0, 0])

try:
    robot_id = p.loadURDF(robot_urdf_path, start_pos, start_orientation, useFixedBase=False)
except p.error as e:
    print(f"Failed to load URDF. Error: {e}")
    sys.exit(1)

num_joints = p.getNumJoints(robot_id)
print(f"Number of joints: {num_joints}")

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

# define link indices for feet
right_foot_link_index = link_name_to_index.get("foot_right")
left_foot_link_index = link_name_to_index.get("foot_left")

if right_foot_link_index is None:
    print("'foot_right' not found in link names")
    sys.exit(1)

if left_foot_link_index is None:
    print("'foot_left' not found in link names")
    sys.exit(1)

# FIXME: these sliders are not working
# create sliders for controlling the target positions
try:
    right_foot_sliders = [
        p.addUserDebugParameter("Right_Foot_X", -1, 1, 0),
        p.addUserDebugParameter("Right_Foot_Y", -1, 1, -0.2),
        p.addUserDebugParameter("Right_Foot_Z", 0, 1, 0),
    ]
    print("Right foot sliders:", right_foot_sliders)
except Exception as e:
    print(f"Failed to create right foot sliders: {e}")
    sys.exit(1)

try:
    left_foot_sliders = [
        p.addUserDebugParameter("Left_Foot_X", -1, 1, 0),
        p.addUserDebugParameter("Left_Foot_Y", -1, 1, 0.2),
        p.addUserDebugParameter("Left_Foot_Z", 0, 1, 0),
    ]
    print("Left foot sliders:", left_foot_sliders)
except Exception as e:
    print(f"Failed to create left foot sliders: {e}")
    sys.exit(1)

while p.isConnected():
    # read target positions from sliders
    try:
        right_foot_target_pos = [p.readUserDebugParameter(slider) for slider in right_foot_sliders]
    except Exception as e:
        print(f"Failed to read right foot sliders: {e}")
        sys.exit(1)

    try:
        left_foot_target_pos = [p.readUserDebugParameter(slider) for slider in left_foot_sliders]
    except Exception as e:
        print(f"Failed to read left foot sliders: {e}")
        sys.exit(1)

    # compute inverse kinematics for the right foot
    right_foot_joint_angles = p.calculateInverseKinematics(
        robot_id, right_foot_link_index, right_foot_target_pos)

    # compute inverse kinematics for the left foot
    left_foot_joint_angles = p.calculateInverseKinematics(
        robot_id, left_foot_link_index, left_foot_target_pos)

    # apply joint angles to the robot
    for i, joint_index in enumerate(controllable_joints):
        if i < len(right_foot_joint_angles):
            p.setJointMotorControl2(robot_id, joint_index, p.POSITION_CONTROL, right_foot_joint_angles[i])
        elif i - len(right_foot_joint_angles) < len(left_foot_joint_angles):
            idx = i - len(right_foot_joint_angles)
            p.setJointMotorControl2(robot_id, joint_index, p.POSITION_CONTROL, left_foot_joint_angles[idx])
        else:
            break

    # step the simulation
    p.stepSimulation()
    time.sleep(1./240.)
