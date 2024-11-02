"""Main script to initialize the robot and start walking using Pygame for input."""

import argparse
import time
import threading
import pygame  # Import Pygame
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
        self.walk_thread = None
        self.stop_walking_event = threading.Event()
        self.running = True 

    def stand(self):
        """Put robot in standing position."""
        print("Transitioning to STAND state")
        
        if self.walk_thread and self.walk_thread.is_alive():
            print("Stopping walking...")
            self.stop_walking_event.set()
            self.walk_thread.join()
            self.stop_walking_event.clear()

        stand_positions = [0.0] * 10  # 10 joints
        self.robot.set_servo_positions(stand_positions)
        time.sleep(2)

    def walk(self):
        """Start walking using RL policy."""
        print("Transitioning to WALK state")

        if not self.walk_thread or not self.walk_thread.is_alive():
            self.walk_thread = threading.Thread(
                target=inference_loop,
                args=(self.policy, self.robot, self.stop_walking_event)
            )
            self.walk_thread.start()
        else:
            print("Already walking.")

    def handle_events(self):
        """Handle Pygame events for state transitions."""
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.running = False
                if self.current_state == RobotState.WALK:
                    self.stand()
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_w:
                    if self.current_state != RobotState.WALK:
                        self.current_state = RobotState.WALK
                        self.walk()
                    else:
                        print("Already in WALK state.")
                elif event.key == pygame.K_SPACE:
                    if self.current_state != RobotState.STAND:
                        self.current_state = RobotState.STAND
                        self.stand()
                    else:
                        print("Already in STAND state.")

    def run(self):
        """Main loop to handle events."""
        pygame.init()
        screen = pygame.display.set_mode((200, 200))  # Minimal window
        pygame.display.set_caption("Robot Controller")

        print("Press 'w' to WALK")
        print("Press 'space' to STAND")
        print("Press 'Esc' or close window to exit.")

        clock = pygame.time.Clock()

        while self.running:
            self.handle_events()
            clock.tick(30)

        pygame.quit()

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument("--model_path", type=str, required=True, help="Path to the ONNX model")
    args = parser.parse_args()

    robot = Robot()
    policy = ort.InferenceSession(args.model_path)

    controller = RobotController(robot, policy)
    controller.stand()
    controller.run()






