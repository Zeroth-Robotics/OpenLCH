import asyncio
import json
import websockets
from enum import Enum
from typing import List, Dict, Any

class ServoMode(Enum):
    Position = 0
    ConstantSpeed = 1
    PWMOpenLoop = 2
    StepServo = 3

class ServoDirection(Enum):
    Clockwise = 0
    Counterclockwise = 1

class MemoryLockState(Enum):
    Unlocked = 0
    Locked = 1

class TorqueMode(Enum):
    Disabled = 0
    Enabled = 1
    Stiff = 2

class ServoData:
    def __init__(self, data: Dict[str, Any]):
        self.acceleration = data['acceleration']
        self.async_write_flag = data['async_write_flag']
        self.current_current = data['current_current']
        self.current_load = data['current_load']
        self.current_location = data['current_location']
        self.current_speed = data['current_speed']
        self.current_temperature = data['current_temperature']
        self.current_voltage = data['current_voltage']
        self.lock_mark = data['lock_mark']
        self.mobile_sign = data['mobile_sign']
        self.reserved1 = data['reserved1']
        self.reserved2 = data['reserved2']
        self.running_speed = data['running_speed']
        self.running_time = data['running_time']
        self.servo_status = data['servo_status']
        self.target_location = data['target_location']
        self.torque_limit = data['torque_limit']
        self.torque_switch = data['torque_switch']

    def __repr__(self):
        return f"ServoData(current_location={self.current_location}, current_speed={self.current_speed}, current_temperature={self.current_temperature})"

class ContinuousReadResponse:
    def __init__(self, data: Dict[str, Any]):
        self.servo_data = [ServoData(servo) for servo in data['servo']]
        self.task_run_count = data['task_run_count']

    def __repr__(self):
        return f"ContinuousReadResponse(servos={len(self.servo_data)}, task_run_count={self.task_run_count})"

class WriteMultipleCommand:
    def __init__(self, ids: List[int], positions: List[int], times: List[int], speeds: List[int], only_write_positions: int):
        self.ids = ids
        self.positions = positions
        self.times = times
        self.speeds = speeds
        self.only_write_positions = only_write_positions

    def to_dict(self):
        return {
            "ids": self.ids,
            "positions": self.positions,
            "times": self.times,
            "speeds": self.speeds,
            "only_write_positions": self.only_write_positions
        }

    def __repr__(self):
        return f"WriteMultipleCommand(ids={self.ids}, positions={self.positions}, times={self.times}, speeds={self.speeds}, only_write_positions={self.only_write_positions})"

class ServoClient:
    def __init__(self, uri="ws://192.168.42.1:8080"):
        self.uri = uri
        self.websocket = None

    async def connect(self):
        if self.websocket is None or self.websocket.closed:
            self.websocket = await websockets.connect(self.uri)

    async def disconnect(self):
        if self.websocket and not self.websocket.closed:
            await self.websocket.close()

    async def _send_command(self, command, params):
        try:
            await self.connect()
            message = json.dumps({"command": command, "params": params})
            await self.websocket.send(message)
            response = await self.websocket.recv()
            parsed_response = json.loads(response)
            
            if not parsed_response['success']:
                error_msg = parsed_response.get('error', 'Unknown error')
                print(f"Error in command {command}: {error_msg}")
                return None
            
            return parsed_response.get('data')
        except websockets.exceptions.WebSocketException as e:
            print(f"WebSocket error: {e}")
            await self.disconnect()
            return None
        except json.JSONDecodeError as e:
            print(f"JSON decoding error: {e}")
            return None
        except Exception as e:
            print(f"Unexpected error: {e}")
            await self.disconnect()
            return None

    async def move_servo(self, id: int, position: int, time: int, speed: int):
        params = {
            "id": id,
            "position": position,
            "time": time,
            "speed": speed
        }
        return await self._send_command("Move", params)

    async def set_mode(self, id: int, mode: ServoMode):
        params = {
            "id": id,
            "mode": mode.value
        }
        return await self._send_command("SetMode", params)

    async def set_speed(self, id: int, speed: int, direction: ServoDirection):
        params = {
            "id": id,
            "speed": speed,
            "direction": direction.value
        }
        return await self._send_command("SetSpeed", params)

    async def read_info(self, id: int):
        params = {
            "id": id
        }
        return await self._send_command("ReadInfo", params)

    async def read_pid(self, id: int):
        params = {
            "id": id
        }
        return await self._send_command("ReadPID", params)

    async def set_pid(self, id: int, p: int, i: int, d: int):
        params = {
            "id": id,
            "p": p,
            "i": i,
            "d": d
        }
        return await self._send_command("SetPID", params)

    async def set_memory_lock(self, id: int, state: MemoryLockState):
        params = {
            "id": id,
            "state": state.value
        }
        return await self._send_command("SetMemoryLock", params)

    async def read_angle_limits(self, id: int):
        params = {
            "id": id
        }
        return await self._send_command("ReadAngleLimits", params)

    async def set_torque_mode(self, id: int, mode: TorqueMode):
        params = {
            "id": id,
            "mode": mode.value
        }
        return await self._send_command("SetTorqueMode", params)

    async def scan(self, id: int):
        params = {
            "id": id
        }
        return await self._send_command("Scan", params)

    async def write_multiple(self, command: WriteMultipleCommand):
        params = {
            "cmd": command.to_dict()
        }
        return await self._send_command("WriteMultiple", params)

    async def read_continuous(self) -> ContinuousReadResponse:
        data = await self._send_command("ReadContinuous", None)
        if data:
            return ContinuousReadResponse(data)
        return None

# Example usage
async def main():
    client = ServoClient()

    odd = True
    move_timer = 0
    target_interval = 1/50  # 50Hz
    try:
        while True:
            loop_start = asyncio.get_event_loop().time()

            # Read continuous data
            response = await client.read_continuous()
            if response:
                print("Read continuous response:", response.servo_data[8])  # Print data for servo 9 (index 8)

            # Move servo once per second
            if move_timer >= 50:  # 50 iterations at 50Hz = 1 second
                await client.write_multiple(WriteMultipleCommand(
                    ids=list(range(1, 17)),
                    positions=[3500]*16 if odd else [3000]*16,
                    times=[0]*16,
                    speeds=[0]*16,
                    only_write_positions=1
                ))
                odd = not odd
                move_timer = 0
            else:
                move_timer += 1

            # Calculate remaining time and sleep if possible
            elapsed_time = asyncio.get_event_loop().time() - loop_start
            remaining_time = target_interval - elapsed_time

            if remaining_time > 0:
                await asyncio.sleep(remaining_time)
            else:
                print(f"Warning: Loop running late by {-remaining_time:.6f} seconds")

    finally:
        await client.disconnect()

if __name__ == "__main__":
    asyncio.run(main())
