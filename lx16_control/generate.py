import json
import numpy as np

# Define the number of servos
NUM_SERVOS = 12

# Static movement states with positions and transition times
raw_movements = {
    "move_forward": [
        {"pos": [867, 91, 225, 903, 330, 769, 250, 387, 587, 197, 210, 562, 569], "trans_time": 600},
        {"pos": [868, 91, 225, 1004, 330, 769, 250, 387, 587, 199, 151, 562, 570], "trans_time": 600},
        {"pos": [867, 91, 225, 1001, 329, 769, 250, 387, 587, 225, 299, 560, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 1019, 330, 766, 250, 386, 587, 211, 143, 562, 569], "trans_time": 600},
        {"pos": [867, 92, 225, 837, 328, 766, 250, 387, 586, 210, 144, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 955, 328, 769, 250, 387, 587, 210, 82, 561, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 956, 328, 769, 250, 387, 587, 213, 250, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 1037, 331, 767, 250, 387, 587, 210, 158, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 865, 329, 768, 250, 387, 587, 210, 159, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 946, 329, 768, 250, 387, 587, 211, 78, 561, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 948, 328, 769, 250, 387, 587, 213, 234, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 1016, 329, 768, 250, 387, 587, 210, 150, 562, 569], "trans_time": 600},
        {"pos": [867, 91, 225, 857, 330, 767, 250, 387, 587, 210, 151, 562, 569], "trans_time": 600},
    ],
    "zero": [
        {"pos": [865, 92, 228, 895, 514, 628, 249, 385, 586, 338, 195, 387, 570], "trans_time": 600},
        {"pos": [865, 92, 228, 895, 514, 628, 249, 385, 586, 338, 195, 387, 570], "trans_time": 600}

    ],
    "bounce": [
    {"pos": [867, 90, 225, 895, 510, 625, 247, 386, 587, 341, 195, 391, 570], "trans_time": 600},
    {"pos": [867, 91, 225, 887, 374, 751, 247, 387, 587, 210, 187, 544, 570], "trans_time": 600},
    ],
    #  "dabbing": [
        
    #     # {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
    #     # {"pos": [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 961, -1, 112], "trans_time": 300},
    #     # {"pos":  [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 960, -1, 300], "trans_time": 300},

    #     {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
    #     {"pos": [619, 254, 1193, 907, 2, 20, 446, 483, 100, -10, 960, -1, 0], "trans_time": 300},
    #     {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},


        
        # {"pos": [1081, 147, -83, 907, 126, -19, 82, 393, 166, 44, 959, 0, 0], "trans_time": 300},
        # {"pos": [1081, 147, -83, 907, 216, -188, 82, 394, 166, 199, 960, -194, 0], "trans_time": 300},
        # {"pos": [896, 59, -194, 857, 342, 1194, 240, 394, 165, 288, 921, 1193, 0], "trans_time": 300},

    # ],
}

def interpolate_positions(start, end, steps):
    """Interpolate between start and end positions over the given number of steps."""
    return np.linspace(start, end, steps).tolist()

# Generate the fully interpolated movements
movements = {}

for movement_name, movement_data in raw_movements.items():
    interpolated_movement = []
    for i in range(len(movement_data)):
        start_pos = np.array(movement_data[i]["pos"])
        transition_time = movement_data[i]["trans_time"]
        steps = max(1, int(transition_time / 50))  # Calculate the number of 50ms steps
        end_pos = np.array(movement_data[(i + 1) % len(movement_data)]["pos"])  # Next position
        
        # Interpolate between the current position and the next
        interpolated_positions = interpolate_positions(start_pos, end_pos, steps)
        
        # Append the interpolated positions to the movement list
        interpolated_movement.extend(interpolated_positions)

    movements[movement_name] = interpolated_movement

# Save the movements to a JSON file
with open('movements.json', 'w') as f:
    json.dump(movements, f, indent=4)

print("Interpolated movements array saved to movements.json")

