import pybullet as p
import pybullet_data
import time
import os
import sys
import math
from typing import List, Tuple, Dict

def initialize_pybullet(gui: bool = True) -> int:
    """
    Initialize PyBullet simulation.

    Args:
        gui (bool): Whether to use GUI or DIRECT mode.

    Returns:
        int: The client ID of the simulation.
    """
    if gui:
        client_id = p.connect(p.GUI)
    else:
        client_id = p.connect(p.DIRECT)

    if client_id < 0:
        print("Failed to connect to PyBullet")
        sys.exit(1)

    # set gravity
    p.setGravity(0, 0, 0)
    p.setTimeStep(1./240.)

    return client_id

def load_robot(urdf_path: str, start_pos: List[float], start_orientation: Tuple[float, float, float, float]) -> int:
    """
    Load the robot URDF into the simulation.

    Args:
        urdf_path (str): Path to the robot URDF file.
        start_pos (List[float]): Starting position [x, y, z].
        start_orientation (Tuple[float, float, float, float]): Starting orientation as quaternion.

    Returns:
        int: The robot body unique ID.
    """
    if not os.path.exists(urdf_path):
        print(f"URDF file not found at: {urdf_path}")
        sys.exit(1)

    try:
        robot_id = p.loadURDF(urdf_path, start_pos, start_orientation, useFixedBase=False)
    except p.error as e:
        print(f"Failed to load URDF. Error: {e}")
        sys.exit(1)

    return robot_id

def get_robot_joints(robot_id: int) -> Tuple[List[int], Dict[str, int]]:
    """
    Get controllable joints and link indices of the robot.

    Args:
        robot_id (int): The robot body unique ID.

    Returns:
        Tuple[List[int], Dict[str, int]]: Controllable joint indices, mapping from link names to indices.
    """
    num_joints = p.getNumJoints(robot_id)
    controllable_joints = []
    link_name_to_index = {}

    print("\nJoint and Link Information:")
    print("-" * 50)
    for i in range(num_joints):
        info = p.getJointInfo(robot_id, i)
        joint_type = info[2]
        joint_name = info[1].decode('utf-8')
        link_name = info[12].decode('utf-8')
        link_name_to_index[link_name] = i
        if joint_type == p.JOINT_REVOLUTE:
            controllable_joints.append(i)
        print(f"Joint {i}: Name={joint_name}, Type={joint_type}, Link={link_name}")
    
    print("\nControllable Joints:", controllable_joints)
    print("Available Links:", list(link_name_to_index.keys()))
    return controllable_joints, link_name_to_index

def calculate_inverse_kinematics(robot_id: int, target_link: str, target_position: List[float], link_name_to_index: Dict[str, int]) -> List[float]:
    """
    Calculate the inverse kinematics for a given target link and position.

    Args:
        robot_id (int): The robot body unique ID.
        target_link (str): Name of the target link.
        target_position (List[float]): Desired position [x, y, z] of the target link.
        link_name_to_index (Dict[str, int]): Mapping from link names to indices.

    Returns:
        List[float]: Joint angles to achieve the desired position.
    """
    link_index = link_name_to_index.get(target_link)
    if link_index is None:
        print(f"Link '{target_link}' not found in the robot model.")
        print(f"Available links: {list(link_name_to_index.keys())}")  # Add this debug line
        return []

    joint_angles = p.calculateInverseKinematics(robot_id, link_index, target_position)
    return joint_angles

def goto_position(robot_id: int, targets: Dict[str, List[float]], link_name_to_index: Dict[str, int], controllable_joints: List[int]) -> None:
    """
    Move specified robot links to their target positions.

    Args:
        robot_id (int): The robot body unique ID.
        targets (Dict[str, List[float]]): Mapping from link names ('left_hand', 'right_hand', 'left_foot', 'right_foot') to target positions [x, y, z].
        link_name_to_index (Dict[str, int]): Mapping from link names to indices.
        controllable_joints (List[int]): List of controllable joint indices.
    """
    print("-" * 50)
    # store joint angles for each target
    target_joint_angles = {}

    # calculate joint angles for each target
    for link_name, target_pos in targets.items():
        print(f"\n[INFO] Target Position for: {link_name} -> {target_pos}")
        link_index = link_name_to_index.get(link_name)
        if link_index is not None:
            current_pos = p.getLinkState(robot_id, link_index)[0]
            print(f"[INFO] Current Position for: {link_name} -> {current_pos}")
        
        joint_angles = calculate_inverse_kinematics(robot_id, link_name, target_pos, link_name_to_index)
        if joint_angles:
            # print(f"\n[INFO] Calculated joint angles: {joint_angles}")
            target_joint_angles[link_name] = joint_angles
        else:
            print(f"Failed to calculate joint angles for {link_name}")

    # average the joint angles if multiple targets are specified
    if target_joint_angles:
        num_joints = len(controllable_joints)
        joint_angles = [0.0] * num_joints
        for angles in target_joint_angles.values():
            for i in range(num_joints):
                joint_angles[i] += angles[i]
        joint_angles = [angle / len(target_joint_angles) for angle in joint_angles]

        # print("\nApplying joint angles:")
        # apply the joint angles
        for i, joint_index in enumerate(controllable_joints):
            # print(f"Setting joint {joint_index} to {joint_angles[i]}")
            p.setJointMotorControl2(
                robot_id, joint_index, p.POSITION_CONTROL, targetPosition=joint_angles[i])
    else:
        print("No valid targets provided.")

def create_position_parameters() -> Dict[str, List[int]]:
    """
    Create GUI parameters for target positions.
    
    Returns:
        Dict[str, List[int]]: Dictionary mapping link names to their parameter IDs
    """
    param_ids = {}
    
    # Define the links and their default positions
    links = {
        'Left_Hand': [-0.15, 0.05, 0.4],
        'hand_right': [0.15, 0.05, 0.4],
        'foot_left': [0.1, -0.1, 0.3],
        'foot_right': [-0.1, -0.1, 0.3]
    }
    
    for link, default_pos in links.items():
        param_ids[link] = [
            p.addUserDebugParameter(f"{link}_x", -1, 1, default_pos[0]),
            p.addUserDebugParameter(f"{link}_y", -1, 1, default_pos[1]),
            p.addUserDebugParameter(f"{link}_z", 0, 1, default_pos[2])
        ]
    
    return param_ids

def get_target_positions(param_ids: Dict[str, List[int]]) -> Dict[str, List[float]]:
    """
    Get current target positions from GUI parameters.
    
    Args:
        param_ids (Dict[str, List[int]]): Dictionary mapping link names to their parameter IDs
        
    Returns:
        Dict[str, List[float]]: Current target positions for each link
    """
    targets = {}
    for link, ids in param_ids.items():
        targets[link] = [
            p.readUserDebugParameter(ids[0]),
            p.readUserDebugParameter(ids[1]),
            p.readUserDebugParameter(ids[2])
        ]
    return targets

def visualize_targets(targets: Dict[str, List[float]], point_ids: Dict[str, int]) -> Dict[str, int]:
    """
    Visualize target positions with red dots.
    
    Args:
        targets (Dict[str, List[float]]): Target positions for each link
        point_ids (Dict[str, int]): Existing debug point IDs
        
    Returns:
        Dict[str, int]: Updated debug point IDs
    """
    for link, pos in targets.items():
        if link in point_ids:
            p.removeUserDebugItem(point_ids[link])
        point_ids[link] = p.addUserDebugPoints([pos], [[1, 0, 0]], pointSize=6)  # Remove the [0]
    return point_ids

def main():
    # initialize simulation
    client_id = initialize_pybullet(gui=True)

    # Set up search paths
    current_dir = os.path.dirname(os.path.abspath(__file__))
    urdf_path = os.path.join(current_dir, 'urdf')
    p.setAdditionalSearchPath(urdf_path)
    p.setAdditionalSearchPath(pybullet_data.getDataPath())

    # load plane
    try:
        plane_id = p.loadURDF("plane.urdf")
    except p.error:
        print("Warning: Could not load plane.urdf. Continuing without it.")

    # load robot
    robot_urdf_path = os.path.join(urdf_path, 'robot_fixed.urdf')
    start_pos = [0, 0, 0.5]
    start_orientation = p.getQuaternionFromEuler([0, 0, 0])
    robot_id = load_robot(robot_urdf_path, start_pos, start_orientation)

    # get robot joint and link info
    controllable_joints, link_name_to_index = get_robot_joints(robot_id)

    # Create GUI parameters instead of hard-coded targets
    param_ids = create_position_parameters()
    point_ids = {}  # Store debug point IDs

    # simulation loop
    while p.isConnected():
        # Get current target positions from GUI
        targets = get_target_positions(param_ids)
        
        # Visualize target positions
        point_ids = visualize_targets(targets, point_ids)
        
        # move robot to target positions
        goto_position(robot_id, targets, link_name_to_index, controllable_joints)
        p.stepSimulation()
        time.sleep(1./240.)

if __name__ == "__main__":
    main()
