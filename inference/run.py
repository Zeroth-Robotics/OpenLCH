"""Main script to initialize the robot and start walking."""

import argparse
import time
import threading
import onnxruntime as ort
from enum import Enum
from robot import Robot
from rl_walk import inference_loop

class RobotState(Enum):
    STAND = "stand"
    WALK = "walk"

class RobotController:
    def __init__(self, robot: Robot, policy: ort.InferenceSession):
        self.robot = robot
        self.policy = policy
        self.current_state = RobotState.STAND
        self.walking = False  # Flag to control walking loop
        self.walk_thread = None  # Thread for walking
            
    def stand(self):
        """Put robot in standing position."""
        print("Transitioning to STAND state")
        self.walking = False  # Stop the walking loop
        if self.walk_thread and self.walk_thread.is_alive():
            self.walk_thread.join()  # Wait for the walking thread to finish
        stand_positions = [0.0] * 10  # 10 joints
        self.robot.set_servo_positions(stand_positions)
        time.sleep(2)
            
    def walk(self):
        """Start walking using RL policy."""
        print("Transitioning to WALK state")
        self.walking = True
        # Start the walking loop in a new thread
        self.walk_thread = threading.Thread(target=self.walk_loop)
        self.walk_thread.start()

    def walk_loop(self):
        """Walking loop that can be stopped."""
        while self.walking:
            inference_loop(self.policy, self.robot)
            # Add a small sleep to prevent high CPU usage
            time.sleep(0.01)
            
    def handle_keyboard(self):
        """Handle keyboard input for state transitions."""
        print("Press 'w' to WALK")
        print("Press 'space' to STAND")
        print("Press 'q' to QUIT")
        
        while True:
            key = input().lower()
                
            if key == "w":
                if self.current_state != RobotState.WALK:
                    self.current_state = RobotState.WALK
                    self.walk()
            elif key == " ":  # space key
                if self.current_state != RobotState.STAND:
                    self.current_state = RobotState.STAND
                    self.stand()
            elif key == "q":
                print("Exiting program.")
                self.walking = False
                if self.walk_thread and self.walk_thread.is_alive():
                    self.walk_thread.join()
                break
            else:
                print("Invalid input. Press 'w' to WALK, 'space' to STAND, 'q' to QUIT.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument("--model_path", type=str, required=True, help="Path to the ONNX model")
    args = parser.parse_args()

    robot = Robot()
    policy = ort.InferenceSession(args.model_path)
    
    controller = RobotController(robot, policy)
    controller.stand()
    controller.handle_keyboard()






