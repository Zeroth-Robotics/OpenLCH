import pygame
import time

# Robot Control Functions (to be replaced by your actual robot control commands)
def move_forward():
    print("Robot moving forward")

def move_backward():
    print("Robot moving backward")

def turn_left():
    print("Robot turning left")

def turn_right():
    print("Robot turning right")

def stop():
    print("Robot stopping")

def wave_hand():
    print("Robot waving hand")

# Initialize Pygame and the Controller
pygame.init()

# Check for joysticks/controllers
if pygame.joystick.get_count() == 0:
    print("No controller found. Please connect your 8bitdo controller.")
    exit()

# Initialize the controller
joystick = pygame.joystick.Joystick(0)
joystick.init()
print(f"Initialized controller: {joystick.get_name()}")

print(f"Number of axes: {joystick.get_numaxes()}")

# Add this after pygame.init() but before the main loop
last_state = "stopped"
last_button_state = False  # Track button state

# Main Loop
try:
    while True:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                raise KeyboardInterrupt
        
        # Pump the event handler to update controller states
        pygame.event.pump()

        # Read joystick axes (for analog sticks, triggers, etc.)
        left_stick_y = joystick.get_axis(1)    # Y-axis of the left stick
        left_stick_x = joystick.get_axis(0)    # X-axis of the left stick (for turning)

        # Add deadzone constant
        DEADZONE = 0.2

        # Handle analog stick input with deadzone
        if abs(left_stick_y) < DEADZONE and abs(left_stick_x) < DEADZONE:
            if last_state != "stopped":
                stop()
                last_state = "stopped"
        elif left_stick_y < -DEADZONE:
            if last_state != "forward":
                move_forward()
                last_state = "forward"
        elif left_stick_y > DEADZONE:
            if last_state != "backward":
                move_backward()
                last_state = "backward"
        elif left_stick_x < -DEADZONE:
            if last_state != "left":
                turn_left()
                last_state = "left"
        elif left_stick_x > DEADZONE:
            if last_state != "right":
                turn_right()
                last_state = "right"

        # Add this after reading the axes
        # print(f"X: {left_stick_x:.2f}, Y: {left_stick_y:.2f}")

        # Read button presses
        button_state = joystick.get_button(0)  # Current button state
        if button_state and not last_button_state:  # Button just pressed
            wave_hand()
        last_button_state = button_state  # Update button state

        # Add a small delay to avoid maxing out CPU
        time.sleep(0.1)

except KeyboardInterrupt:
    # Gracefully handle the exit
    print("\nExiting program.")
finally:
    joystick.quit()
    pygame.quit()
