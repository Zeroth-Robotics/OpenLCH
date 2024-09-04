import sys, time
import serial
import lewansoul_lx16a

SERIAL_PORT = '/dev/cu.usbserial-10'

ctrl = lewansoul_lx16a.ServoController(
    serial.Serial(SERIAL_PORT, 115200, timeout=1),
)

def servo_info(id):
    print("Servo id: {}".format(id))
    print("Position: {}".format(ctrl.get_position(id)))
    print("Temperature: {}, limit: {}".format(ctrl.get_temperature(id), ctrl.get_max_temperature_limit(id)))
    print("Led error: {}".format(ctrl.get_led_errors(id)))

if __name__ == '__main__':
    args = sys.argv[1:]
    args_num = len(args)
    if args_num == 0:
        try:
            id = ctrl.get_servo_id()
            if id != 255:
                servo_info(id)
        except:
            print('Error!')
    elif args[0] == 'info':
        servo_info(int(args[1]))
    elif args[0] == 'assign' and args_num == 3:
        ctrl.set_servo_id(int(args[1]), int(args[2]))
    elif args[0] == 'test' and args_num == 2:
        id = int(args[1])
        ctrl.move(id, 0, 500)
        time.sleep(1)
        ctrl.move(id, 1000, 500)
        time.sleep(1)
        ctrl.move(id, 500, 500)
    elif args[0] == 'test1' and args_num == 2:
        id = int(args[1])
        ctrl.move(id, 0, 0)
        time.sleep(1)
        ctrl.move(id, 1000, 0)
    elif args[0] == 'test2' and args_num == 1:
        res = []
        for i in range(13):
            res.append(ctrl.get_position(i + 1))
        print(res)
    elif args[0] == 'set_pos' and args_num == 3:
        ctrl.move(int(args[1]), int(args[2]), 500)
    elif args[0] == 'set_pos_offset' and args_num == 3:
        id = int(args[1])
        ctrl.set_position_offset(id, int(args[2]))
        time.sleep(0.1)
        ctrl.save_position_offset(id)
    elif args[0] == 'set_pos1' and args_num == 3:
        ctrl.move(int(args[1]), int(args[2]), 0)
    elif args[0] == 'set_temp_limit' and args_num == 3:
        ctrl.set_max_temperature_limit(int(args[1]), int(args[2]))
    elif args[0] == 'reset' and args_num == 2:
        id = int(args[1])
        ctrl.set_position_offset(id, 0)
        time.sleep(0.1)
        ctrl.save_position_offset(id)
        time.sleep(0.1)
        ctrl.set_max_temperature_limit(id, 85)
        time.sleep(0.1)
        ctrl.set_led_errors(id, 7)
        time.sleep(0.1)
        ctrl.led_on(id)
        time.sleep(0.1)
        ctrl.set_servo_id(id, 1)
    elif args[0] == 'scan':
        print("Found servo ids:")
        for i in range(1, 255):
            try:
                ctrl.get_position(i, 0.03)
                print(i)
            except:
                pass
    elif args[0] == 'zzz':
        # ctrl.move(10, 0, 0)
        # ctrl.move(6, 0, 0)
        # time.sleep(1)
        # ctrl.move(10, 1000, 0)
        # ctrl.move(6, 1000, 0)

        start_time = time.time()

        id = 1
        for i in range(1, 1000):
            ctrl.move(id, 200, 500)
            time.sleep(.4)
            ctrl.move(id, 550, 500)
            time.sleep(.4)

            elapsed_time = time.time() - start_time
            print("Temperature: %d, limit: %d, elapsed: %d:%02d" % (ctrl.get_temperature(id), ctrl.get_max_temperature_limit(id), (int)(elapsed_time/60), (int)(elapsed_time%60)))
    else:
        print("\
Usage:\n\
python3 lx_setup.py info 1\n\
python3 lx_setup.py assign 1 10\n\
python3 lx_setup.py test 1\n\
python3 lx_setup.py set_pos 1 500\n\
python3 lx_setup.py set_temp_limit 1 50\n\
python3 lx_setup.py reset 1\n\
python3 lx_setup.py scan\n")
