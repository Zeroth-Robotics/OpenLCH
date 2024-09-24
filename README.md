# Low Cost Humanoid (WIP)

<div align="center">

[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/kscalelabs/onshape/blob/main/LICENSE)
<!-- [![Discord](https://img.shields.io/discord/1280974143936004238)](https://discord.gg/kN8jXdt7Rx) -->
<!-- [![Wiki](https://img.shields.io/badge/wiki-humanoids-black)](https://humanoids.wiki) -->
</div>

Stompy Micro is an open-source, low-cost humanoid robot designed for dynamic movements (like running and boxing) and speech to action. We made the V0 of this humanoid robot at a [hackathon](https://github.com/jingxiangmo/low_cost_humanoid/blob/0ab372ece6673fc3f66a62588d88ebfb2695d9be/README.md) on 2024/08/31. We're now actively working on V1, redesigning the mechanical and eletrical components, with a focus on reinforcement learning for locomotion. 

The goal of this project is to democratize humanoid robotics by offering open-source designs and affordable components, making it accessible to researchers, hobbyists, and educators. This project is inspired by the design of [Robotis OP3](https://emanual.robotis.com/docs/en/platform/op3/introduction/) and the accessibility of [Alex Koch's robot arms](https://github.com/AlexanderKoch-Koch/low_cost_robot).


<div style="display: flex; justify-content: space-between;">
    <div style="width: 48%;">
        <img src="/public/waving.png" alt="Robot Waving" style="width: 100%; height: 450px; object-fit: cover;">
        <p> V0 </p>
    </div>
    <div style="width: 48%;">
        <img src="/public/CAD.png" alt="CAD Model" style="width: 100%; height: 450px; object-fit: cover;">
        <p> V1 Legs </p>
    </div>
</div>


## Mechatronics

Specifications:
| Height | Weight | DoF |
|:--|:--|:--|
| 50cm | 15lb | 16 (5 DoF per leg, 3 DoF per arm) |

### CAD

**OnShape**: https://cad.onshape.com/documents/cacc96f8a7850b951e7aa69a/w/3a0a4ee9d8251956ba5e5e92/e/b92662619a7718ffa83530f2

**URDF/MJCF**: https://kscale.store/file/5b9b5eecb7ffcab1

Want to try the URDF model? Try here: https://kscale.store/file/5b9b5eecb7ffcab1



## BoM
| Part         | Description                                                                   | Link                                                                                                                                             | Quantity | Total Cost (USD) | Date Decided |
|:--           |:--                                                                            |:--                                                                                                                                              |:--       |:--               |:--           |
| Serial BusServos       | STS3250                                 | [Link](https://www.alibaba.com/product-detail/50KG-High-Torque-HV-Robot-Servo_1601045497742.html)                | x16      | 224              | 9/10         |
| Controller   | Milk-V                                              | [Link](https://milkv.io/duo-s)                                                                                                             | x1       | 10               | TBD          |
| Servoboard   | Serial Bus Servo Driver Board                                                  | [Link](https://www.waveshare.com/product/bus-servo-adapter-a.htm)                                                                   | x1       | 5                |              |
| IMU          | 3-Axis Gyroscope, 3-Axis Accelerometer, 3-Axis Magnetometer                    | [Link](https://ozzmaker.com/product/berryimu-accelerometer-gyroscope-magnetometer-barometricaltitude-sensor/)                                 | x1       | 45               | 9/20         |
| Camera | A010 RGBD TOF 3D Depth vision camera                                           | [Link](https://www.amazon.com/Sipeed-MaixSense-Vision-Camera-MS-A075V/dp/B0BPSSFLGH?th=1)                                                      | x1       |                  | TBD          |
| Battery      | RC Lipos                                                                      | [Link](https://www.amazon.com/KBT-1200mAh-Rechargeable-Replacement-Compatible/dp/B0C23Y3VZK?source=ps-sl-shoppingads-lpcontext&ref_=fplfs&smid=A3FKMD6P089KQA&th=1) | x1       |                  | Proposed     |
| 12V to 5V    | 12V to 5V, 3 amp capacity (may need connectors)                               | [Link](https://www.digikey.com/en/products/detail/dfrobot/DFR0571/9559261?utm_adgroup=&utm_source=google&utm_medium=cpc&utm_campaign=PMax%20Shopping_Product_Low%20ROAS%20Categories&utm_term=&utm_content=&utm_id=go_cmp-20243063506_adg-_ad-__dev-m_ext-_prd-9559261_sig-Cj0KCQjwxsm3BhDrARIsAMtVz6OMuYeF6xr0kLeY_OpvuVUEMmsyxZNsa2Y6567T93VBpmQ31ocUh2kaAkzOEALw_wcB&gad_source=1&gbraid=0AAAAADrbLlgUgtqZiYHKHVpeN-YpI-cro&gclid=Cj0KCQjwxsm3BhDrARIsAMtVz6OMuYeF6xr0kLeY_OpvuVUEMmsyxZNsa2Y6567T93VBpmQ31ocUh2kaAkzOEALw_wcB) | x1       | 3                | 9/24         |
| Microphone   | N/A                                                                  |                                                 N/A                                                                                                  | x1       |                  |              |


## Assembly
WIP


## Software

### Embedded
WIP

### Runtime
WIP

We'll be using [Milk-V Duo](https://milkv.io/duo-s) as the main controller for the robot. 


## Simulation

<div style="display: flex; justify-content: space-between;">
    <div style="width: 48%;">
        <img src="/public/isaac.png" alt="Isaac Simulation" style="width: 100%; height: auto; object-fit: cover;">
        <p>Isaac Simulation</p>
    </div>
    <div style="width: 48%;">
        <img src="/public/urdf.png" alt="URDF Model" style="width: 100%; height: auto; object-fit: cover;">
        <p>URDF Model</p>
    </div>
</div>

### NVIDIA IsaacSim
We use NVIDIA IsaacSim to simulate, train, and test the robot for locomotion. Our simulation is based on the K-Scale simulation library.

Link:https://github.com/jingxiangmo/sim/tree/master
Docs: https://docs.kscale.dev/software/simulation/isaac

### PyBullet
Currently the URDF model also support PyBullet using K-Scale OnShape library: https://docs.kscale.dev/software/onshape

## ML
### Locomotion
#### RL (PPO)
We use RL to train the robot to stand walk. The training is done in IsaacSim with the K-Scale simulation and training library: https://github.com/jingxiangmo/sim/tree/master.

### Manipulation
#### E-VLA (WIP)
Integration of E-VLA will be in V2. For more details, please refer to the [E-VLA documentation](https://docs.kscale.dev/software/models/evla).

#### K-Lang (WIP)
Integration of K-Lang will be in V2. For more details, please refer to the [K-Lang documentation](https://docs.kscale.dev/software/klang/intro).



## License
This project is licensed under the MIT License.


## Current Contributors

- **Kelsey Pool** - mechanical & design
- **Denys Bezmenov** - eletrical & embeddd
- **Jingxiang Mo** - electrical, software, & ML
- **Henri Lemoine** - ML


<details>
<summary>Hackathon Team</summary>

<div style="display: flex; justify-content: space-between;">
  <img src="/public/hackathon_team_1.png" alt="Hackathon Team Photo" style="width: 48%;">
</div>

- **Kelsey Pool** - Mechanical design
- **Denys Bezmenov** - Eletrical and software control
- **Jingxiang Mo** - Mechanical assembly, electrical, and software control
- **Baaqer Farhat** - Mechanical assembly, software

Acknowledgements:
- **Jacob Zietek** - AI/ML and simulation help 
- **Saad Sharief** - Teleoperation help

</details>

Last updated: 2024/09/24