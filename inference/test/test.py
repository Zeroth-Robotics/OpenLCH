from openlch import HAL
import time

def get_servo_positions(hal: HAL) -> list:
    servo_positions = hal.servo.get_positions()
    positions = [pos for _, pos in servo_positions[:10]]
    print(f"[INFO]: GET servo positions: {positions}")
    return positions

def set_servo_positions(positions : list, hal : HAL) -> None:
    print(f"[INFO]: SET servo positions: {positions}")
    servo_positions = [(i, pos) for i, pos in enumerate(positions[:10])]
    hal.servo.set_positions(servo_positions)


def calibrate_all_servos(hal : HAL) -> None:
    for i in range(1, 16): 
            print(f"[INFO]: Started servo {i - 1} calibration =====")
            
            while True:
                status = hal.servo.get_calibration_status()
                if not status.get('is_calibrating', False):
                    break
                time.sleep(1)
            
            hal.servo.start_calibration(i)



if __name__ == "__main__":
    hal = HAL()


    print(hal.servo.scan())
    calibrate_all_servos(hal)

    # print("[INFO]: Setting servos 1-10 to 0...")
    # zero_positions = [0.0] * 10
    # set_servo_positions(zero_positions, hal)
    
    # time.sleep(1)
    

    # current_positions = get_servo_positions(hal)

  
        

    # print("[INFO]: Getting servo positions...")
    # get_servo_positions(hal)
    # set_servo_positions([0.0] * 10, hal)
