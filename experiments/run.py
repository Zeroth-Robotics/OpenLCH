from robot import Robot
import pygame
import time
import math
import threading
import multiprocessing as mp
import onnxruntime as ort
from model import inference
import os


def state_stand(robot : Robot) -> bool:
    print("Standing")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    return True


def state_walk(robot : Robot) -> bool:
    print("Walking")

    model_path = '../sim/sim/example/walking/walking_micro.onnx'
    if not os.path.isfile(model_path):
        print(f"Model file not found at {model_path}")
        return False
    policy = ort.InferenceSession(model_path)

    data_queue = mp.Queue()

    inference_thread = threading.Thread(target=inference, args=(policy, robot, data_queue))
    inference_thread.start()

    return True

def state_forward_recovery(robot : Robot) -> bool:
    print("Forward recovery")

    return True

def state_backward_recovery(robot : Robot) -> bool:
    print("Backward recovery")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])

    return True

def state_wave(robot : Robot) -> bool:
    print("Waving")
    
    initial_positions = {
        "left_shoulder_yaw": 0.0,
        "left_shoulder_pitch": 0.0,
        "left_elbow_yaw": 0.0,
    }
    robot.set_servo_positions_by_name(initial_positions)
    time.sleep(0.5)
    
    wave_up_positions = {
        "left_shoulder_pitch": math.radians(45.0),
        "left_elbow_yaw": math.radians(-100.0),
    }
    robot.set_servo_positions_by_name(wave_up_positions)
    time.sleep(0.5) 

    for _ in range(3):
        wave_out = {"left_shoulder_yaw": math.radians(15.0)}
        robot.set_servo_positions_by_name(wave_out)
        time.sleep(0.3)
        
        wave_in = {"left_shoulder_yaw": math.radians(-15.0)}
        robot.set_servo_positions_by_name(wave_in)
        time.sleep(0.3)
    
    robot.set_servo_positions_by_name(initial_positions)
    time.sleep(0.5)
    
    return True

def main():
    robot = Robot()
    try:
        robot.initialize()
        state_stand(robot)

        pygame.init()
        screen = pygame.display.set_mode((400, 300))
        pygame.display.set_caption("Robot Control")

        print("Press 'w' to walk, 'space' to stand, 'q' to wave, 'e' to sit, '1' for forward recovery, '2' for backward recovery, 'escape' to quit")
        
        running = True
        while running:
            try:
                for event in pygame.event.get():
                    if event.type == pygame.QUIT:
                        running = False
                    elif event.type == pygame.KEYDOWN:
                        try:
                            if event.key == pygame.K_w:
                                state_walk(robot)
                            elif event.key == pygame.K_SPACE:
                                state_stand(robot)
                            elif event.key == pygame.K_q:
                                state_wave(robot)
                            elif event.key == pygame.K_1:
                                state_forward_recovery(robot)
                            elif event.key == pygame.K_2:
                                state_backward_recovery(robot)
                            elif event.key == pygame.K_ESCAPE:
                                running = False
                        except Exception as e:
                            print(f"Error during state execution: {e}")
            except KeyboardInterrupt:
                print("\nCtrl+C detected, shutting down gracefully...")
                break
                        
    except Exception as e:
        print(f"Error during robot operation: {e}")
    except KeyboardInterrupt:
        print("\nCtrl+C detected, shutting down gracefully...")
        try:
            robot.disable_motors()
            print("Motors disabled")
        except:
            print("Error disabling motors")
    finally:
        try:
            robot.disable_motors()
            print("Motors disabled")
        except:
            print("Error disabling motors")
        pygame.quit()

if __name__ == "__main__":
    main()











