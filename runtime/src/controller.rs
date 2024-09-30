

struct StandingControllerPID {
    robot: Robot,
    config: Config,
}

struct StandingControllerPPO {
    model: OnnxInfer,
    robot: Robot,
    config: Config,
}




impl StandingControllerPID {


}

impl StandingControllerPPO {
    pub fn new(model: OnnxInfer, robot: Robot, config: Config) -> Self {
        Self {
            model,
            robot,
            config,
        }
    }
    

    pub fn get_state(&self) -> Result<()> {
        let state = self.robot.joint_states();
        let imu = self.robot.imu_state();
        let action = self.model.infer(state, imu)?;
        Ok(())
    }

    pub fn send_commands(&self) -> Result<()> {
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        Ok(())
    }

    
}

