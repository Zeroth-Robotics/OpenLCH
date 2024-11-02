"""Main script to initialize the robot and start walking."""

import argparse
import logging
import onnxruntime as ort
from robot import Robot
from rl_walk import inference_loop

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--embodiment", type=str, default="stompymicro")
    parser.add_argument("--model_path", type=str, required=True, help="Path to the ONNX model")
    args = parser.parse_args()

    # initalization 
    robot = Robot()
    policy = ort.InferenceSession(args.model_path)

    
    inference_loop(policy, robot)






