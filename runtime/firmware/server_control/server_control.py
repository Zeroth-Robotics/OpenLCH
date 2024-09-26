import asyncio
import websockets
import json
import keyboard
import sys

# WebSocket endpoint
WEBSOCKET_URL = 'ws://192.168.8.165:80/ws'

# dictionary to map keys to actions
key_actions = {
    '1': 'attack_position',
    '2': 'standing_position',
    '3': 'emergency_recovery',
    'w': 'forward',
    'a': 'left',
    's': 'backward',
    'd': 'right',
    'k': 'right_uppercut',
    'j': 'left_uppercut',
    'h': 'left_hook',
    'l': 'right_hook'
}

async def send_action(websocket, servo_states):
    data = json.dumps({'servo_states': servo_states})
    print(f"Sending command: {data}")
    sys.stdout.flush() 
    await websocket.send(data)
    print(f'Command sent: {data}')
    sys.stdout.flush()

async def attack_position(websocket):
    # await send_action(websocket, [1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000])
    pass

async def standing_position(websocket):
    # await send_action(websocket, [500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500])
    pass

async def emergency_recovery(websocket):
    # await send_action(websocket, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    pass

async def forward(websocket):
    # await send_action(websocket, [800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800])
    # await asyncio.sleep(0.5)
    # await send_action(websocket, [850, 850, 850, 850, 850, 850, 850, 850, 850, 850, 850, 850, 850])
    pass

async def left(websocket):
    # await send_action(websocket, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    pass

async def backward(websocket):
    # await send_action(websocket, [500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500])
    pass

async def right(websocket):
    # await send_action(websocket, [1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000])
    pass

async def right_uppercut(websocket):
    # await send_action(websocket, [900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900, 900])
    pass

async def left_uppercut(websocket):
    # await send_action(websocket, [600, 600, 600, 600, 600, 600, 600, 600, 600, 600, 600, 600, 600])
    pass

async def left_hook(websocket):
    # await send_action(websocket, [700, 700, 700, 700, 700, 700, 700, 700, 700, 700, 700, 700, 700])
    pass


async def right_hook(websocket):
    # await send_action(websocket, [800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800])
    pass

async def on_key_event(event, websocket):
    if event.name in key_actions:
        action = key_actions[event.name]
        print(f"Key '{event.name}' pressed, executing '{action}' command...")
        sys.stdout.flush() 
        await globals()[action](websocket)
        print(f"Command '{action}' completed.")
        sys.stdout.flush()

async def main():
    async with websockets.connect(WEBSOCKET_URL) as websocket:
        print("Connected to WebSocket server. Press keys to send actions. Press ESC to quit.")
        sys.stdout.flush()
        
        loop = asyncio.get_event_loop()
        while True:
            event = keyboard.read_event(suppress=True)
            if event.event_type == keyboard.KEY_DOWN:
                await on_key_event(event, websocket)
            if event.name == 'esc':
                break
        
        print("Exiting program...")
        sys.stdout.flush()
        
        await websocket.close()

if __name__ == '__main__':
    asyncio.run(main())