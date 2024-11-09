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

# Main Loop
try:
    while True:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                raise KeyboardInterrupt
        
        # Pump the event handler to update controller states
        pygame.event.pump()

        # Read joystick axes (for analog sticks, triggers, etc.)
        left_stick_y = joystick.get_axis(1)  # Typically the Y-axis of the left stick
        right_stick_x = joystick.get_axis(3)  # Typically the X-axis of the right stick

        # Add deadzone constant
        DEADZONE = 0.2

        # Handle analog stick input with deadzone
        if abs(left_stick_y) < DEADZONE and abs(right_stick_x) < DEADZONE:
            stop()
        elif left_stick_y < -DEADZONE:
            move_forward()
        elif left_stick_y > DEADZONE:
            move_backward()
        elif right_stick_x < -DEADZONE:
            turn_left()
        elif right_stick_x > DEADZONE:
            turn_right()

        # Read button presses
        if joystick.get_button(0):  # Assuming button 0 is mapped to "A"
            wave_hand()

        # Add a small delay to avoid maxing out CPU
        time.sleep(0.1)

except KeyboardInterrupt:
    # Gracefully handle the exit
    print("\nExiting program.")
finally:
    joystick.quit()
    pygame.quit()
