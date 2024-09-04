# Low Cost Humanoid

This robot was built in 24 hours during the 2024 August 31st K-Scale Robotics Hackathon. Our goal was to create a very low cost ($270) and open source humanoid platform based on [Robotis OP3](https://emanual.robotis.com/docs/en/platform/op3/introduction/) to be as accessible as [Alex Koch's robot arms](https://github.com/AlexanderKoch-Koch/low_cost_robot) in the future as a humanoid research platform. This project is under development.


<div style="display: flex; justify-content: space-between;">
    <img src="/public/waving.png" alt="Robot Waving" style="width: 48%; height: auto;">
    <img src="/public/CAD.png" alt="CAD Model" style="width: 48%; height: auto;">
</div>

The other goal was to win the [humanoid boxing robot competition](https://x.com/TomPJacobs/status/1830430806952820868), between [Ben Bolte](https://x.com/benjamin_bolte/status/1830447989292478682) and our team during the hackathon.


## Mechanical and Electrical Components
### CAD
OnShape: https://cad.onshape.com/documents/cacc96f8a7850b951e7aa69a/v/c2b8fb3a999aade7349c887f/e/d466d644261b5146fb5a1714?renderMode=0&uiState=66d6c8c2a2a8a0713efb762e


### Required Materials

| Part                      | Quantity |  Cost  | Buying Link                                                                                                                                                                                                                                                                                                                                                                                                                                                | Notes                         |
|---------------------------|----------|--------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------|
| ESP32-S3                   | 1        | $15    | [Link](https://www.amazon.com/Espressif-ESP32-S3-DevKitC-1-N8R2-Development-Board/dp/B09D3S7T3M)                                                                                                                                                                                                                                                                                                                                                            | Websocket communication        |
| LX-16A Serial Bus Servo    | 13       | $16.99 | [Link](https://www.amazon.com/LX-16A-Bearing-Durable-Steering-17KG-CM/dp/B073WR3SK9/ref=sr_1_6?crid=WC7HZXR7UKBU&dib=eyJ2IjoiMSJ9.i13SR2IU-jfIL2UtrFqiQ7sVUCZepe4MpSK5k55mb42RoE8tl6Ww7MooGCM5l_q3iOoEghcveU0A3WrOYJb1j83LmCtTCjODtzg4GBkd-jEq9Gs4K4kHsGujAKZDT5u0xm1rESTxS0jKw0QxtmjwdUO9W70NU0fawlygZfqLwcaf-k6eJgsykhwX1olZj-vEx7izcqiWm8WQxoFwA5CpGEB8FUoXG6guN-Q4YkNK4ceELk9DuKCPujQVaM5Pu46hqDX_tiw7QSLw6braYREvD3ydSYzWgNYBgw7HVB6zG70.U34lKdFVgdq-lR8iFGrCzS1HMrJcEpjauLK2zGcXgtc)                                                                                                                                                          | Motor actuation              |
| Serial Bus Servo Connector | 1        | $9.99  | [Link](https://www.amazon.com/LewanSoul-Equipped-Position-Temperature-Voltage/dp/B0CKMVR2ZS/ref=sr_1_1?crid=ZI1544ULDTEG&dib=eyJ2IjoiMSJ9._UnZXyY7COsnXNPrWW9ZCNPN6sI8p2E3zknDLeCKIXlcv7H3wpfZxwynKj7zY8C8XJ7kc23bpyOFDVgn0RhBgB6fLsyURNT-xAVAaCnneNo.Z5M_4EeXamX9OtRnFIIC3fSnES06YTYmQIM8OtXzgFk&dib_tag=se&keywords=lx-16a+debug+board&qid=1725349938&sprefix=lx-16a+debug+boar%2Caps%2C149&sr=8-1)                                                                                                                                                                       |               |
| 7.4V Battery (Zeee 2S 2200mAh) | 1        | $15    | [Link](https://www.amazon.com/Zeee-Connector-Helicopter-Airplane-Quadcopter/dp/B0C2V8DT8W/ref=sr_1_13?crid=LH3FCLGS8IZN&dib=eyJ2IjoiMSJ9.Ot6_WiaFcXekhA5pWEacRlaA6DxFdUVnblKvcVCHLgveifEz2icWSVsy3flt06iO7ejyq0btSFcL-Bi6zKd1Fr0ls5m8QhgXDFr1cZOzQ_uEqAAI_OzSsRZT01HIk3MDiwhj0cYwiUiQnLNsFkzNTZP-qDreUo4gtSl-vhznGOkKBlZozu9Cuz4-32eUxxeQY45RwTcICHxjrJlxt7ueqv-vdS98KTW6JEBYqWc2xx0F-QYpmhdf-M4xI8RtyVhyA1Zqf8nKnmfSZGhKyK3C1Y8iGmTFg7tcuWyX-2OrLIs.w0Jtz8COYFqD3fo0FLnLnoSiC1lSi-4KO96su_XIMSU&dib_tag=se&keywords=zeee+battery+small&qid=1725350134&sprefix=zeee+battery+smal%2Caps%2C132&sr=8-13) | Power supply                 |
| 4 x 3/8 in. Fasteners         | 40       | $6.33  | [Link](https://www.amazon.com/Stainless-Screws-Phillips-Threaded-TPOHH/dp/B0D2N2PS66/ref=sr_1_3?crid=30ZXVQXUFO6HC&dib=eyJ2IjoiMSJ9.SXyAHQLP9sPQ49rlmsVlsHTigXaN_K-R8xMBEQxyUm4odnhutnG_MITkQvT-WrJObIyZ62JJEgreEoX1MyDovbFQClSCcxnOe7HeQUqe-Rl2_y--kecLWrPwtyEpsEbD1r5_luED766hSGpq7fGbXA2ScyaWJNteyy5TotBy7iGZZCUWZJ602sM4g6f3KqiPw4wbLwa1aVIVcBoD9J3LV_vZVGlfRdMjLcweXYv9638.9d8pLzY472Gtcwp0ZazhnbfxfJyl8pha_tA2oPYyhlE&dib_tag=se&keywords=4+x+3%2F8+in.&qid=1725350293&sprefix=4+x+3%2F8+in.%2Caps%2C210&sr=8-3) |          |
| M2 x 4mm Fasteners            | 13       | $4.5   | [Link](https://www.amazon.com/Deal4GO-10-Pack-Replacement-Phillips-Heatsink/dp/B0B373RNJP?source=ps-sl-shoppingads-lpcontext&ref_=fplfs&psc=1&smid=A1BQWGBLWM4VYT)                                                                                                                                                                                                                                                                                        |   |

**Total Cost:** $271.69


## Software Components

The goal of the software control was to allow the user to control the servos via a wireless websocket connection (using the ESP32 as a WiFi module) to do high-level control of the robot (e.g., WASD keys for forward, backward, left, and right movement).

### Setup

`/server_control` contains the websocket server control code for the ESP32.
`/wire_control` contains the motor servo control code.

```shell
# setup virtual environment
python3.10 -m venv venv # requires python 3.10
source venv/bin/activate
pip install -r requirements.txt
```

### Websocket Control
Controlling servos via websocket:
```shell
websocat ws://192.168.8.165:80/ws
```

Accept commands:
```shell
{"servo_states": [800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800, 800]}
# where each value is 0-1000 for each servo from 1 to 13
#response:
{"response":"ok"}

{"servo_states": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}
```

and 

``` shell
{"get_states": 1}
# response:
{"servo_1":1,"servo_2":802,"servo_3":802,"servo_4":801,"servo_5":801,"servo_6":801,"servo_7":802,"servo_8":801,"servo_9":801,"servo_10":802,"servo_11":801,"servo_12":801,"servo_13":-1}
```


## Future Work
We plan to clean up the CAD and the codebase and add the following feature in the up coming weeks to offer an open source alternative to the Robotis OP3 platform:

- [ ] Classic controller
- [ ] Simulation (MuJoCo)
- [ ] Teleoperation
- [ ] PPO standing and walking policy



## Hackathon Team
<div style="display: flex; justify-content: space-between;">
  <img src="/public/hackathon_team_1.png" alt="Hackathon Team Photo" style="width: 48%;">
  <video width="48%" controls>
    <source src="https://github.com/user-attachments/assets/cbc4ee92-7218-4fed-80e0-7263a1674b4f" type="video/mp4">
    Your browser does not support the video tag.
  </video>
</div>

*(we named him "David")*


- **Kelsey Pool** - Mechanical design
- **Denys Bezmenov** - Eletrical and software control
- **Jingxiang Mo** - Mechanical assembly, electrical, and software control
- **Baaqer Farhat** - Mechanical assembly, software

Acknowledgements:
- **Jacob Zietek** - AI/ML and simulation help 
- **Saad Sharief** - Teleoperation help
