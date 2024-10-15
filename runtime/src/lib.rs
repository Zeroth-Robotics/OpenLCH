#[cfg(all(target_arch = "riscv64", target_os = "linux", feature = "milkv"))]
pub mod hal_risc;

#[cfg(any(target_os = "macos", all(target_os = "linux", not(feature = "milkv"))))]
pub mod hal_serial;

// Create a public hal module
pub mod hal {
    // Re-export the appropriate HAL implementation
    #[cfg(all(target_arch = "riscv64", target_os = "linux", feature = "milkv"))]
    pub use super::hal_risc::*;

    #[cfg(any(target_os = "macos", all(target_os = "linux", not(feature = "milkv"))))]
    pub use super::hal_serial::*;
}

// Public API
pub use hal::{Servo, MAX_SERVOS, TorqueMode, ServoRegister};
