


pub struct Robot {
    motors: Vec<Motor>,
    imu: IMU,
    num_motors: usize,
}


impl Robot {
    pub fn new() -> Self {
        Self {
            motors: vec![],
            imu: IMU::new(),
            num_motors: 0,
        }
    }

    pub fn joint_states(&self) -> JointStates {
        // get joint states from motors


    }

    pub fn joint_commands(&self) -> JointCommands {
        // get joint commands from motors

    }

    pub fn imu_state(&self) -> IMUState {
        // get imu from imu

    }
    
}

