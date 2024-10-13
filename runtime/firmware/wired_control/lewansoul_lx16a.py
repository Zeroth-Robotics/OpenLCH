__all__ = [
    'ServoController',
    'TimeoutError',

    'SERVO_ERROR_OVER_TEMPERATURE',
    'SERVO_ERROR_OVER_VOLTAGE',
    'SERVO_ERROR_LOCKED_ROTOR',
]


from serial.serialutil import Timeout
from functools import partial
import threading
import logging


SERVO_ID_ALL = 0xfe

SERVO_MOVE_TIME_WRITE = 1
SERVO_MOVE_TIME_READ = 2
SERVO_MOVE_TIME_WAIT_WRITE = 7
SERVO_MOVE_TIME_WAIT_READ = 8
SERVO_MOVE_START = 11
SERVO_MOVE_STOP = 12
SERVO_ID_WRITE = 13
SERVO_ID_READ = 14
SERVO_ANGLE_OFFSET_ADJUST = 17
SERVO_ANGLE_OFFSET_WRITE = 18
SERVO_ANGLE_OFFSET_READ = 19
SERVO_ANGLE_LIMIT_WRITE = 20
SERVO_ANGLE_LIMIT_READ = 21
SERVO_VIN_LIMIT_WRITE = 22
SERVO_VIN_LIMIT_READ = 23
SERVO_TEMP_MAX_LIMIT_WRITE = 24
SERVO_TEMP_MAX_LIMIT_READ = 25
SERVO_TEMP_READ = 26
SERVO_VIN_READ = 27
SERVO_POS_READ = 28
SERVO_OR_MOTOR_MODE_WRITE = 29
SERVO_OR_MOTOR_MODE_READ = 30
SERVO_LOAD_OR_UNLOAD_WRITE = 31
SERVO_LOAD_OR_UNLOAD_READ = 32
SERVO_LED_CTRL_WRITE = 33
SERVO_LED_CTRL_READ = 34
SERVO_LED_ERROR_WRITE = 35
SERVO_LED_ERROR_READ = 36


SERVO_ERROR_OVER_TEMPERATURE = 1
SERVO_ERROR_OVER_VOLTAGE = 2
SERVO_ERROR_LOCKED_ROTOR = 4


def lower_byte(value):
    return int(value) % 256


def higher_byte(value):
    return int(value / 256) % 256


def word(low, high):
    return int(low) + int(high)*256


def clamp(range_min, range_max, value):
    return min(range_max, max(range_min, value))


class TimeoutError(RuntimeError):
    pass


LOGGER = logging.getLogger('lewansoul.servos.lx16a')


class Servo(object):
    def __init__(self, controller, servo_id):
        self.__dict__.update({
            '_controller': controller,
            'servo_id': servo_id,
        })

    def __hasattr__(self, name):
        return hasattr(self._controller, name)

    def __getattr__(self, name):
        attr = getattr(self._controller, name)
        if callable(attr):
            attr = partial(attr, self.servo_id)
        return attr


class ServoController(object):
    def __init__(self, serial, timeout=1):
        self._serial = serial
        self._timeout = timeout
        self._lock = threading.RLock()

    def _command(self, servo_id, command, *params):
        length = 3 + len(params)
        checksum = 255-((servo_id + length + command + sum(params)) % 256)
        LOGGER.debug('Sending servo control packet: %s', [
            0x55, 0x55, servo_id, length, command, *params, checksum
        ])
        with self._lock:
            self._serial.write(bytearray([
                0x55, 0x55, servo_id, length, command, *params, checksum
            ]))

    def _wait_for_response(self, servo_id, command, timeout=None):
        timeout = Timeout(timeout or self._timeout)

        def read(size=1):
            self._serial.timeout = timeout.time_left()
            data = self._serial.read(size)
            if len(data) != size:
                raise TimeoutError()
            return data

        while True:
            data = []
            data += read(1)
            if data[-1] != 0x55:
                continue
            data += read(1)
            if data[-1] != 0x55:
                continue
            data += read(3)
            sid = data[2]
            length = data[3]
            cmd = data[4]
            if length > 7:
                LOGGER.error('Invalid length for packet %s', list(data))
                continue

            data += read(length-3) if length > 3 else []
            params = data[5:]
            data += read(1)
            checksum = data[-1]
            if 255-(sid + length + cmd + sum(params)) % 256 != checksum:
                LOGGER.error('Invalid checksum for packet %s', list(data))
                continue

            if cmd != command:
                LOGGER.warning('Got unexpected command %s response %s',
                               cmd, list(data))
                continue

            if servo_id != SERVO_ID_ALL and sid != servo_id:
                LOGGER.warning('Got command response from unexpected servo %s', sid)
                continue

            return [sid, cmd, *params]

    def _query(self, servo_id, command, timeout=None):
        with self._lock:
            self._command(servo_id, command)
            return self._wait_for_response(servo_id, command, timeout=timeout)

    def servo(self, servo_id):
        return Servo(self, servo_id)

    def get_servo_id(self, servo_id=SERVO_ID_ALL, timeout=None):
        response = self._query(servo_id, SERVO_ID_READ, timeout=timeout)
        return response[2]

    def set_servo_id(self, servo_id, new_servo_id):
        self._command(servo_id, SERVO_ID_WRITE, new_servo_id)

    def move(self, servo_id, position, time=0):
        position = clamp(0, 1000, position)
        time = clamp(0, 30000, time)

        self._command(
            servo_id, SERVO_MOVE_TIME_WRITE,
            lower_byte(position), higher_byte(position),
            lower_byte(time), higher_byte(time),
        )

    def get_prepared_move(self, servo_id, timeout=None):
        """Returns servo position and time tuple"""
        response = self._query(servo_id, SERVO_MOVE_TIME_WAIT_READ, timeout=timeout)
        return word(response[2], response[3]), word(response[4], response[5])

    def move_prepare(self, servo_id, position, time=0):
        position = clamp(0, 1000, position)
        time = clamp(0, 30000, time)

        self._command(
            servo_id, SERVO_MOVE_TIME_WAIT_WRITE,
            lower_byte(position), higher_byte(position),
            lower_byte(time), higher_byte(time),
        )

    def move_start(self, servo_id=SERVO_ID_ALL):
        self._command(servo_id, SERVO_MOVE_START)

    def move_stop(self, servo_id=SERVO_ID_ALL):
        self._command(servo_id, SERVO_MOVE_STOP)

    def get_position_offset(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_ANGLE_OFFSET_READ, timeout=timeout)
        deviation = response[2]
        if deviation > 127:
            deviation -= 256
        return deviation

    def set_position_offset(self, servo_id, deviation):
        deviation = clamp(-125, 125, deviation)
        if deviation < 0:
            deviation += 256
        self._command(servo_id, SERVO_ANGLE_OFFSET_ADJUST, deviation)

    def save_position_offset(self, servo_id):
        self._command(servo_id, SERVO_ANGLE_OFFSET_WRITE)

    def get_position_limits(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_ANGLE_LIMIT_READ, timeout=timeout)
        return word(response[2], response[3]), word(response[4], response[5])

    def set_position_limits(self, servo_id, min_position, max_position):
        min_position = clamp(0, 1000, min_position)
        max_position = clamp(0, 1000, max_position)
        self._command(
            servo_id, SERVO_ANGLE_LIMIT_WRITE,
            lower_byte(min_position), higher_byte(min_position),
            lower_byte(max_position), higher_byte(max_position),
        )

    def get_voltage_limits(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_VIN_LIMIT_READ, timeout=timeout)
        return word(response[2], response[3]), word(response[4], response[5])

    def set_voltage_limits(self, servo_id, min_voltage, max_voltage):
        min_voltage = clamp(4500, 12000, min_voltage)
        max_voltage = clamp(4500, 12000, max_voltage)
        self._command(
            servo_id, SERVO_VIN_LIMIT_WRITE,
            lower_byte(min_voltage), higher_byte(min_voltage),
            lower_byte(max_voltage), higher_byte(max_voltage),
        )

    def get_max_temperature_limit(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_TEMP_MAX_LIMIT_READ, timeout=timeout)
        return response[2]

    def set_max_temperature_limit(self, servo_id, max_temperature):
        max_temperature = clamp(50, 100, max_temperature)
        self._command(servo_id, SERVO_TEMP_MAX_LIMIT_WRITE, max_temperature)

    def get_temperature(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_TEMP_READ, timeout=timeout)
        return response[2]

    def get_voltage(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_VIN_READ, timeout=timeout)
        return word(response[2], response[3])

    def get_position(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_POS_READ, timeout=timeout)
        position = word(response[2], response[3])
        if position > 32767:
            position -= 65536
        return position

    def get_mode(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_OR_MOTOR_MODE_READ, timeout=timeout)
        return response[2]

    def get_motor_speed(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_OR_MOTOR_MODE_READ, timeout=timeout)
        if response[2] != 1:
            return 0
        speed = word(response[4], response[5])
        if speed > 32767:
            speed -= 65536
        return speed

    def set_servo_mode(self, servo_id):
        self._command(
            servo_id, SERVO_OR_MOTOR_MODE_WRITE, 0, 0, 0, 0,
        )

    def set_motor_mode(self, servo_id, speed=0):
        speed = clamp(-1000, 1000, speed)
        if speed < 0:
            speed += 65536
        self._command(
            servo_id, SERVO_OR_MOTOR_MODE_WRITE, 1, 0,
            lower_byte(speed), higher_byte(speed),
        )

    def is_motor_on(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_LOAD_OR_UNLOAD_READ, timeout=timeout)
        return response[2] == 1

    def motor_on(self, servo_id):
        self._command(servo_id, SERVO_LOAD_OR_UNLOAD_WRITE, 1)

    def motor_off(self, servo_id):
        self._command(servo_id, SERVO_LOAD_OR_UNLOAD_WRITE, 0)

    def is_led_on(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_LED_CTRL_READ, timeout=timeout)
        return response[2] == 0

    def led_on(self, servo_id):
        self._command(servo_id, SERVO_LED_CTRL_WRITE, 0)

    def led_off(self, servo_id):
        self._command(servo_id, SERVO_LED_CTRL_WRITE, 1)

    def get_led_errors(self, servo_id, timeout=None):
        response = self._query(servo_id, SERVO_LED_ERROR_READ, timeout=timeout)
        return response[2]

    def set_led_errors(self, servo_id, error):
        error = clamp(0, 7, error)
        self._command(servo_id, SERVO_LED_ERROR_WRITE, error)
