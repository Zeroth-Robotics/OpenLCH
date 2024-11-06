from robot import Robot
import pygame
import time


def state_sit():
    print("Sitting")


def state_stand(robot : Robot) -> bool:
    print("Standing")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    return True

    
def state_walk(robot : Robot) -> bool:
    print("Walking")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    return True

def state_forward_recovery(robot : Robot) -> bool:
    print("Forward recovery")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])

    return True

def state_backward_recovery(robot : Robot) -> bool:
    print("Backward recovery")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])

    return True

def state_wave(robot : Robot) -> bool:
    print("Waving")
    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    robot.set_joint_positions([0.0, 30.0, 30.0, 30.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    robot.set_joint_positions([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    robot.set_servo_positions()

    return True

def main():
    robot = Robot()
    robot.initialize()
    
    state_stand(robot)

    pygame.init()
    screen = pygame.display.set_mode((400, 300))
    pygame.display.set_caption("Robot Control")

    print("Press 'w' to walk, 'space' to stand, 'q' to wave, 'e' to sit, '1' to forward recovery, '2' to backward recovery, 'escape' to quit")
    
    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_w:
                    state_walk(robot)
                elif event.key == pygame.K_SPACE:
                    state_stand()
                elif event.key == pygame.K_q:
                    state_wave(robot)
                elif event.key == pygame.K_e:
                    state_sit(robot)
                elif event.key == pygame.K_1:
                    state_forward_recovery()
                elif event.key == pygame.K_2:
                    state_backward_recovery(robot)
                elif event.key == pygame.K_ESCAPE:
                    running = False
    
    pygame.quit()

main()











