import sys, time, spline
import serial
import lewansoul_lx16a

SERIAL_PORT = '/dev/ttyUSB0'

controller = lewansoul_lx16a.ServoController(
    serial.Serial(SERIAL_PORT, 115200, timeout=1),
)

# max speed is a 0.64 sec per 1000 points
time_per_point = 0.64 / 1000
speed_factor = 0.1

class Joint:
    def __init__(self, id):
        self.id = id
        self.prev_pos = -1
        try:
            self.start = self.end = controller.get_position(self.id)
        except lewansoul_lx16a.TimeoutError:
            print("TimeoutError, servo id: ", self.id)
            exit(0)

    def Start(self, end):
        self.start = self.end
        self.end = end
        self.period = abs(self.end - self.start) * time_per_point / speed_factor
        self.start_time = time.time()
        return True

    def get_next_pos(self):
        t = time.time() - self.start_time
        if t >= self.period: return None

        t /= float(self.period)
        return int(spline.GetSimpleHermiteY(t, self.start, self.end))
        #return self.start + t * (self.end - self.start)

    def move_to(self, pos, t=0):
        if pos != self.prev_pos:
            self.prev_pos = pos
            try:
                controller.move(self.id, pos, t)
            except:
                pass

    def get_pos(self):
        try:
            return controller.get_position(self.id)
        except:
            return None

class Controller:
    def __init__(self):
        self.ma = []

    def balance_time(self, joints):
        max_time = 0
        for m in joints:
            if m.period > max_time:
                max_time = m.period

        for m in joints:
            m.period = max_time

    def get_ma(self, pos):
        self.ma.append(pos)
        if len(self.ma) > 3:
            self.ma.pop(0)
        sum = 0.0
        for s in self.ma:
            sum += s
        return int(sum * 1000 / len(self.ma))

    def move_joints(self, joints):
        self.balance_time(joints)

        t = time.time()

        while True:
            time_diff = time.time() - t
            if time_diff > 0.5:
                print("Execution failed....")
                exit(0)
            t = time.time()
            ma = self.get_ma(time_diff)

            for m in joints:
                curr_pos = m.get_pos()
                p = m.get_next_pos()
                if p:
                    m.move_to(p, ma)
                else:
                    if curr_pos:
                        if abs(curr_pos - m.end) > 20:
                            #print('servo {}: to end {}'.format(m.id, abs(curr_pos - m.end)))
                            m.move_to(m.end)
                        else:
                            joints.remove(m)
                time.sleep(0.005)

            if len(joints) == 0:
                break

    def move(self, full_seq):
        for m in full_seq:
            i = 1
            joints = []
            for pos in m:
                if pos:
                    j = Joint(i)
                    if not j.Start(pos):
                        print("Failed to init, servo id:", i)
                        return
                    joints.append(j)
                i += 1
            self.move_joints(joints)

def print_pos(last_id=9):
    s = ""
    for id in range(1, last_id+1):
        if len(s) > 0:
            s += ", "
        try:
            s += '{}'.format(controller.get_position(id))
        except:
            s += 'None'
    print(s)
    return s

def motors_off(last_id=9):
    for id in range(1, last_id+1):
        controller.motor_off(id)

def show_pos():
    while 1:
        print_pos()
        time.sleep(1)

def is_valid_pos(pos):
    return pos >=0 and pos <= 1000

def is_valid_pos_array(pos_array):
    for p in pos_array.split(", "):
        try:
            if not is_valid_pos(int(p)):
                return False
        except:
            print("Some servo is failed!!!")
            exit(0)
    return True

def record_seq():
    with open('seq.txt', 'w') as f:
        while True:
            val = input("Press Enter to write position (or any char to exit)...")
            if val != '': break

            pos = print_pos()
            if is_valid_pos_array(pos):
                f.write(pos + "\n")
            else:
                print("Invalid position. Try again...")

def run_seq():
    full_seq = []

    with open('seq.txt', 'r') as f:
        for line in f.readlines():
            pos = []
            for v in line.split(", "):
                pos.append(int(v))
            full_seq.append(pos)

    c = Controller()
    c.move(full_seq)

if __name__ == '__main__':
    args = sys.argv[1:]
    args_num = len(args)
    if args_num > 0:
        if args[0] == 'record':
            record_seq()
        if args[0] == 'play':
            if args_num > 1:
                speed_factor = float(args[1])
            run_seq()
        if args[0] == 'monitor':
            show_pos()
    else:
        print_pos()

    time.sleep(1)
    motors_off()
