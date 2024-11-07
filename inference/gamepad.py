import pygame
import math
from robot import Robot
import time

def main():
    # Initialize the robot
    robot = Robot()
    robot.initialize()

    # Create a mapping from servo IDs to JointData instances
    servo_id_to_joint = {joint.servo_id: joint for joint in robot.joints}

    def prompt_for_joint():
        while True:
            user_input = input("Enter the servo ID you want to control: ").strip()
            # Remove any non-digit characters from the input
            filtered_input = ''.join(filter(str.isdigit, user_input))
            if not filtered_input:
                print("Invalid input. Please enter a numeric servo ID.")
                continue
            try:
                servo_id_input = int(filtered_input)
                if servo_id_input not in servo_id_to_joint:
                    print(f"Servo ID {servo_id_input} not found.")
                    continue
                return servo_id_to_joint[servo_id_input]
            except ValueError:
                print("Invalid input. Please enter a numeric servo ID.")

    # Initial joint selection
    joint = prompt_for_joint()
    if not joint:
        return

    print(f"Controlling joint '{joint.name}' with servo ID {joint.servo_id}")

    # Initialize Pygame for keyboard input
    pygame.init()
    screen = pygame.display.set_mode((400, 300))
    pygame.display.set_caption("Servo Control")

    # Retrieve current position of the joint
    robot.get_servo_states()
    current_position = joint.position  
    print(f"Joint '{joint.name}' current angle is {math.degrees(current_position):.2f} degrees")

    # Main control loop
    running = True
    while running:
        time.sleep(0.01)  # Small delay to reduce CPU usage
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False

                elif event.key == pygame.K_q:
                    # Switch to controlling a different joint
                    new_joint = prompt_for_joint()
                    if new_joint:
                        joint = new_joint
                        robot.get_servo_states()
                        current_position = joint.position  # Get current position of the new joint
                        print(f"Switched to controlling joint '{joint.name}' with servo ID {joint.servo_id}")
                        print(f"Joint '{joint.name}' current angle is {math.degrees(current_position):.2f} degrees")

                elif event.key == pygame.K_UP:
                    # Increase joint angle by 10 degrees
                    current_position += math.radians(10)
                    robot.set_servo_positions_by_name({joint.name: current_position})
                    print(f"Joint '{joint.name}' angle increased to {math.degrees(current_position):.2f} degrees")

                elif event.key == pygame.K_DOWN:
                    # Decrease joint angle by 10 degrees
                    current_position -= math.radians(10)
                    robot.set_servo_positions_by_name({joint.name: current_position})
                    print(f"Joint '{joint.name}' angle decreased to {math.degrees(current_position):.2f} degrees")

                elif event.key == pygame.K_SPACE:
                    # Save the current angle
                    saved_angle = math.degrees(current_position)
                    print(f"Saved position for joint '{joint.name}': {saved_angle:.2f} degrees")

        # Handle KeyboardInterrupt to allow graceful exit
        try:
            pass  # Placeholder for any additional logic
        except KeyboardInterrupt:
            print("\nCtrl+C detected, shutting down gracefully...")
            running = False

    # Cleanup after exiting the main loop
    try:
        robot.disable_motors()
        print("Motors disabled")
    except Exception as e:
        print(f"Error disabling motors: {e}")
    pygame.quit()

if __name__ == "__main__":
    main()
