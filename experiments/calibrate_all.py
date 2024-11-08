from openlch import HAL
import time
import sys

def get_servo_positions(hal: HAL) -> list:
    servo_positions = hal.servo.get_positions()
    positions = [pos for _, pos in servo_positions[:10]]
    print(f"[INFO]: GET servo positions: {positions}")
    return positions

def set_servo_positions(positions: list, hal: HAL) -> None:
    print(f"[INFO]: SET servo positions: {positions}")
    servo_positions = [(i, pos) for i, pos in enumerate(positions[:10])]
    hal.servo.set_positions(servo_positions)


"""
TODO:
- incrase torque for calibration
- add specific positions for differnet servos when calibrating (sequence)
"""


def set_servo_position(servo_id: int, position: float, hal: HAL) -> None:
    print(f"[INFO]: Setting servo {servo_id} to position {position}")
    hal.servo.set_positions([(servo_id - 1, position)])
    time.sleep(0.5)

def calibrate_all_servos(hal: HAL) -> None:
    # Calibration sequence with specific positions
    sequence = [
        (13, 1.0),  # max position
        (14, 1.0),  # max position
        (12, 0.0),
        (15, 0.0),
        (11, 0.0),
        (16, 0.0),
        (4, 1.0),   # max position
        (9, 1.0),   # max position
        (5, 0.0),
        (10, 0.0),
        (3, 0.0),
        (8, 0.0),
        (2, 0.0),
        (7, 0.0),
        (1, 0.0),
        (6, 0.0),
    ]

    try:
        for servo_id, position in sequence:
            print(f"[INFO]: Started servo {servo_id} calibration =====")
            
            while True:
                status = hal.servo.get_calibration_status()
                if not status.get('is_calibrating', False):
                    break
                time.sleep(1)
            
            hal.servo.start_calibration(servo_id)

        
    except Exception as e:
        print(f"[ERROR]: Calibration failed: {str(e)}")
        cancel_all_calibrations(hal)
        raise e

def cancel_all_calibrations(hal: HAL) -> None:
    print("[INFO]: Canceling all previous calibrations...") 
    for i in range(1, 16):
        try:
            hal.servo.cancel_calibration(i)
            time.sleep(0.1)
        except Exception as e:
            if "No calibration is currently in progress" in str(e):
                continue
            raise e


if __name__ == "__main__":
    hal = HAL()
    
    try:
        print(hal.servo.scan())

        cancel_all_calibrations(hal)

        print("[INFO]: Setting torque...")
        hal.servo.set_torque_enable([(i, True) for i in range(1, 17)])
        time.sleep(1)
        hal.servo.set_torque([(i, 20.0) for i in range(1, 17)])
        time.sleep(1)

        calibrate_all_servos(hal)

        print("[INFO]: Setting positions to 0...")
        time.sleep(1)

        hal.servo.set_positions([(i, 0.0) for i in range(1, 17)])
 
    except KeyboardInterrupt:
        print("\n[INFO]: Keyboard interrupt detected. Canceling calibrations...")
        cancel_all_calibrations(hal)
        sys.exit(0)
    except Exception as e:
        print(f"[ERROR]: An error occurred: {str(e)}")
        cancel_all_calibrations(hal)
        sys.exit(1)
