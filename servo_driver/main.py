from hiwonder_lsbc_v16 import ServoController
from time import sleep, time

if __name__ == '__main__':
    controller = ServoController(vid=0x0483, pid=0x5750)
    controller.move_servos([(3, 800)], 1000)
    print(f'voltage: {controller.get_battery_voltage()}')
    # print(f'pos {1}: {controller.read_servo_angles(1, 1)}')
    sleep(1.5)
    controller.disable_motors(1, 3)
    while True:
        start_time = time()
        angles = controller.read_servo_angles(list(range(1, 14)))
        print(f'time: {int((time()-start_time)*1000)}ms, pos: {angles}')
        sleep(0.05)
