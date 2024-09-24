import json
import numpy as np

# Define the number of servos
NUM_SERVOS = 12

# Static movement states with positions and transition times
raw_movements = {
    "move_forward": [
        {"pos": [913, -63, -37, 907, -45, 71, 212, 526, -50, -10, 960, 0, 0], "trans_time": 600},
        {"pos": [913, 0, 0, 910, 142, -93, 212, 526, 0, 56, 968, -80, 0], "trans_time": 600},
        {"pos": [913, 0, 0, 913, 134, -103, 213, 526, 0, 113, 969, -145, 0], "trans_time": 600},
        {"pos":  [913, 0, 0, 920, 20, 9, 213, 526, 0, 107, 968, -131, 0], "trans_time": 600},
        {"pos":  [913, -30, -37, 901, 6, 4, 212, 526, 0, -21, 960, -13, 10], "trans_time": 600},
       
        # {"pos": [913, -63, -37, 922, -8, 44, 213, 526, -50, 4, 961, -16, -1], "trans_time": 300},

    ],
    "move_left": [
        
        # {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
        # {"pos": [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 961, -1, 112], "trans_time": 300},
        # {"pos":  [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 960, -1, 300], "trans_time": 300},

        {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
        {"pos": [619, 254, 1193, 907, 2, 20, 446, 483, 100, -10, 960, -1, 0], "trans_time": 300},
        # {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
        # {"pos": [603, 250, -194, 907, 2, 21, 446, 431, 99, -3, 960, -1, 2], "trans_time": 300},



        
        # {"pos": [1081, 147, -83, 907, 126, -19, 82, 393, 166, 44, 959, 0, 0], "trans_time": 300},
        # {"pos": [1081, 147, -83, 907, 216, -188, 82, 394, 166, 199, 960, -194, 0], "trans_time": 300},
        # {"pos": [896, 59, -194, 857, 342, 1194, 240, 394, 165, 288, 921, 1193, 0], "trans_time": 300},

    ],
    "move_fuckedup": [
        {"pos": [913, -63, -37, 907, -45, 71, 212, 526, -50, -10, 960, 0, 0], "trans_time": 600},
        {"pos": [913, 0, 0, 910, 142, -93, 212, 526, 0, 56, 968, -80, 0], "trans_time": 600},
        {"pos": [913, 0, 0, 913, 134, -103, 213, 526, 0, 113, 969, -145, 0], "trans_time": 600},
        {"pos":  [913, 0, 0, 920, 20, 9, 213, 526, 0, 107, 968, -131, 0], "trans_time": 600},
        {"pos":  [913, -30, -37, 901, 6, 4, 212, 526, 0, -21, 960, -13, 10], "trans_time": 600},
    ],
     "dabbing": [
        
        # {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
        # {"pos": [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 961, -1, 112], "trans_time": 300},
        # {"pos":  [668, 64, -160, 907, -11, 20, 446, 431, 99, -31, 960, -1, 300], "trans_time": 300},

        {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},
        {"pos": [619, 254, 1193, 907, 2, 20, 446, 483, 100, -10, 960, -1, 0], "trans_time": 300},
        {"pos": [668, 64, -160, 907, -11, 20, 446, 432, 99, -31, 962, -1, 3], "trans_time": 300},


        
        # {"pos": [1081, 147, -83, 907, 126, -19, 82, 393, 166, 44, 959, 0, 0], "trans_time": 300},
        # {"pos": [1081, 147, -83, 907, 216, -188, 82, 394, 166, 199, 960, -194, 0], "trans_time": 300},
        # {"pos": [896, 59, -194, 857, 342, 1194, 240, 394, 165, 288, 921, 1193, 0], "trans_time": 300},

    ],
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

