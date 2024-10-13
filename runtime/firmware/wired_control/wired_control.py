import pygame
import asyncio
import websockets
import json
import time

import sys, time
import serial
import lewansoul_lx16a

SERIAL_PORT = '/dev/cu.usbserial-210'

ctrl = lewansoul_lx16a.ServoController(
    serial.Serial(SERIAL_PORT, 115200, timeout=1),
)
global prev_movement
prev_movement = None    
# Initialize Pygame
pygame.init()

# WebSocket server URI
# Load the pre-generated movement arrays from the JSON file
with open('movements.json', 'r') as f:
    movements = json.load(f)

# Active movement state key
current_movement = None

# Circular buffer to manage movement states
movement_buffer = []
current_step = 0

# Time tracking for transitions
last_transition_time = 0
transition_interval = 0.05  # 50ms

# Function to handle state transitions
async def handle_transitions(websocket):
    global current_step, last_transition_time

    # Check the current time
    current_time = time.time()
    elapsed_time = current_time - last_transition_time

    if elapsed_time >= transition_interval:
        if current_movement:
            # Get the current position from the buffer
            servo_states = movement_buffer[current_step]
            message = json.dumps({"servo_states": servo_states})
            for n, val in enumerate(servo_states):
                ctrl.move(n + 1, val, 20)

            # await websocket.send(message)

            # Move to the next step
            current_step = (current_step + 1) % len(movement_buffer)
            
            # Update the last transition time
            last_transition_time = current_time

# Function to set the current movement and prepare the buffer
def set_movement(movement_name):
    global current_movement, movement_buffer, current_step, last_transition_time, prev_movement
    if movement_name in movements and current_movement != movement_name:
        current_movement = movement_name
        movement_buffer = movements[movement_name]
        if prev_movement != current_movement:
            current_step = 0
            prev_movement = current_movement
        last_transition_time = time.time()  # Reset the timer
        print(f"Transitioning to movement: {movement_name}")

# Function to stop the current movement
def stop_movement():
    global current_movement
    current_movement = None
    print("Stopped movement")

# Main function to run the websocket client
async def main():
    # Set up a simple Pygame window (required to capture events)
    screen = pygame.display.set_mode((640, 480))
    pygame.display.set_caption('Servo Movement Controller')

    # Connect to the WebSocket server
    # async with websockets.connect(WEBSOCKET_URI) as websocket:

    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_w:  # Move forward on 'W' key press
                    set_movement("move_forward")
                elif event.key == pygame.K_a:  # Move left on 'A' key press
                    set_movement("move_left")
                elif event.key == pygame.K_d:  # Move left on 'A' key press
                    set_movement("dabbing")
                elif event.key == pygame.K_q:  # Quit on 'Q' key press
                    running = False
            elif event.type == pygame.KEYUP:
                if event.key in [pygame.K_w, pygame.K_a, pygame.K_d]:
                    stop_movement()

        # Call handle_transitions every 50ms if a movement is active
        await handle_transitions(None)

        # Allow other events to be processed
        await asyncio.sleep(0)  # Yield control to the event loop

    pygame.quit()

# Run the main function
asyncio.run(main())
