from robot import Robot
import pygame
import time
import math






def state_stand(robot : Robot) -> bool:
    print("Standing")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    return True

    
def state_walk(robot : Robot) -> bool:
    print("Walking")

    return True

def state_forward_recovery(robot : Robot) -> bool:
    print("Forward recovery")

    """
    python run.py

    In the pop up window, press 1 key
    """




    # Getting feet on the ground
    robot.set_servo_positions_by_name({
        
        "left_hip_pitch": math.radians(30.0),
        "right_hip_pitch": math.radians(-30.0),

        "left_knee_pitch": math.radians(50.0),
        "right_knee_pitch": math.radians(-50.0),

        "left_ankle_pitch": math.radians(-30.0),
        "right_ankle_pitch": math.radians(30.0),
    })

    # 90 degree position
    robot.set_servo_positions_by_name({

        "left_shoulder_pitch": math.radians(30.0),
        "right_shoulder_pitch": math.radians(-30.0),

        "left_shoulder_yaw": math.radians(-20.0),
        "right_shoulder_yaw": math.radians(20.0),

        "left_elbow_yaw": math.radians(-60.0),
        "right_elbow_yaw": math.radians(60.0),

        "left_hip_pitch": math.radians(30.0),
        "right_hip_pitch": math.radians(-30.0),

        "left_knee_pitch": math.radians(70.0),
        "right_knee_pitch": math.radians(-70.0),

        "left_ankle_pitch": math.radians(0.0),
        "right_ankle_pitch": math.radians(0.0),
    })
 
    # Prep Position
    robot.set_servo_positions_by_name({

        "left_shoulder_pitch": math.radians(30.0),
        "right_shoulder_pitch": math.radians(-30.0),

        "left_shoulder_yaw": math.radians(20.0),
        "right_shoulder_yaw": math.radians(-20.0),

        "left_elbow_yaw": math.radians(-20.0),
        "right_elbow_yaw": math.radians(20.0),

        "left_hip_pitch": math.radians(30.0),
        "right_hip_pitch": math.radians(-30.0),

        "left_knee_pitch": math.radians(70.0),
        "right_knee_pitch": math.radians(-70.0),

        "left_ankle_pitch": math.radians(0.0),
        "right_ankle_pitch": math.radians(0.0),
    })

    time.sleep(1)

    robot.set_servo_positions_by_name({

        "left_shoulder_pitch": math.radians(120.0),
        "right_shoulder_pitch": math.radians(-120.0),

    #     "left_shoulder_yaw": math.radians(-20.0),
    #     "right_shoulder_yaw": math.radians(20.0),

    #     "left_elbow_yaw": math.radians(0.0),
    #     "right_elbow_yaw": math.radians(0.0),

        "left_hip_pitch": math.radians(80.0),
        "right_hip_pitch": math.radians(-80.0),

    #     "left_knee_pitch": math.radians(90.0),
    #     "right_knee_pitch": math.radians(-90.0),

    #     "left_ankle_pitch": math.radians(90.0),
    #     "right_ankle_pitch": math.radians(-90.0),
    })

    robot.set_servo_positions_by_name({

        # "left_shoulder_pitch": math.radians(120.0),
        # "right_shoulder_pitch": math.radians(-120.0),

    #     "left_shoulder_yaw": math.radians(-20.0),
    #     "right_shoulder_yaw": math.radians(20.0),

        # "left_elbow_yaw": math.radians(20.0),
        # "right_elbow_yaw": math.radians(-20.0),

        # "left_hip_pitch": math.radians(90.0),
        # "right_hip_pitch": math.radians(-90.0),

        "left_knee_pitch": math.radians(90.0),
        "right_knee_pitch": math.radians(-90.0),

        "left_ankle_pitch": math.radians(40.0),
        "right_ankle_pitch": math.radians(-40.0),
    })

    robot.set_servo_positions_by_name({

        # "left_shoulder_pitch": math.radians(120.0),
        # "right_shoulder_pitch": math.radians(-120.0),

        # "left_shoulder_yaw": math.radians(-20.0),
        # "right_shoulder_yaw": math.radians(20.0),

        "left_elbow_yaw": math.radians(0.0),
        "right_elbow_yaw": math.radians(0.0),

        # "left_hip_pitch": math.radians(90.0),
        # "right_hip_pitch": math.radians(-90.0),

        "left_knee_pitch": math.radians(90.0),
        "right_knee_pitch": math.radians(-90.0),

        "left_ankle_pitch": math.radians(0.0),
        "right_ankle_pitch": math.radians(-50.0),
    })


    time.sleep(1)

    # Box Position
    robot.set_servo_positions_by_name({

    #     "left_shoulder_pitch": math.radians(120.0),
    #     "right_shoulder_pitch": math.radians(-120.0),

        "left_shoulder_yaw": math.radians(-40.0),
        "right_shoulder_yaw": math.radians(40.0),

    #     "left_elbow_yaw": math.radians(20.0),
    #     "right_elbow_yaw": math.radians(-20.0),

    #     "left_hip_pitch": math.radians(90.0),
    #     "right_hip_pitch": math.radians(-90.0),

    #     "left_knee_pitch": math.radians(90.0),
    #     "right_knee_pitch": math.radians(-90.0),

        "left_ankle_pitch": math.radians(90.0),
        "right_ankle_pitch": math.radians(-90.0),
    })

    time.sleep(1)


    # Tilting torso
    robot.set_servo_positions_by_name({

        "left_shoulder_pitch": math.radians(120.0),
        "right_shoulder_pitch": math.radians(-120.0),

        "left_shoulder_yaw": math.radians(-40.0),
        "right_shoulder_yaw": math.radians(40.0),

        "left_elbow_yaw": math.radians(20.0),
        "right_elbow_yaw": math.radians(-20.0),

        "left_hip_pitch": math.radians(25.0),
        "right_hip_pitch": math.radians(-25.0),

        "left_knee_pitch": math.radians(60.0),
        "right_knee_pitch": math.radians(-60.0),

        "left_ankle_pitch": math.radians(50.0),
        "right_ankle_pitch": math.radians(-50.0),
    })

    time.sleep(4)

    # Setting different torque values for knee and ankle servos
    hip_joints = [joint for joint in robot.joints if "hip" in joint.name]
    # knee_joints = [joint for joint in robot.joints if "knee" in joint.name]
    # ankle_joints = [joint for joint in robot.joints if "ankle" in joint.name]
    robot.hal.servo.set_torque([(joint.servo_id, 15.0) for joint in hip_joints])
    # robot.hal.servo.set_torque([(joint.servo_id, 20.0) for joint in knee_joints])
    # robot.hal.servo.set_torque([(joint.servo_id, 20.0) for joint in ankle_joints])

     # Fully straight
    robot.set_servo_positions_by_name({
        
        "left_ankle_pitch": math.radians(-5.0),
        "right_ankle_pitch": math.radians(5.0),

        "left_knee_pitch": math.radians(0.0),
        "right_knee_pitch": math.radians(-0.0),

        "left_shoulder_pitch": math.radians(0.0),
        "right_shoulder_pitch": math.radians(-0.0),

        "left_shoulder_yaw": math.radians(-0.0),
        "right_shoulder_yaw": math.radians(0.0),

        "left_elbow_yaw": math.radians(0.0),
        "right_elbow_yaw": math.radians(-0.0),

        "left_hip_pitch": math.radians(0.0),
        "right_hip_pitch": math.radians(-0.0),

    })

    leg_joints = [joint for joint in robot.joints if "knee" in joint.name or "ankle" in joint.name]
    robot.hal.servo.set_torque([(joint.servo_id, 20.0) for joint in leg_joints])

    time.sleep(1)


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











