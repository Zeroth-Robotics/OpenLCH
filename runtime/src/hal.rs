use anyhow::Result;

pub trait Servo {
    fn new() -> Result<Self> where Self: Sized;
    fn set_position(&mut self, position: f32) -> Result<()>;
    // Add other common methods here
}
