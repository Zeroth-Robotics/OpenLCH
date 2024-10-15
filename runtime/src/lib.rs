pub mod hal;

#[cfg(all(target_arch = "riscv64", target_os = "linux", feature = "risc"))]
pub mod hal_risc;

#[cfg(any(target_os = "macos", all(target_os = "linux", not(feature = "risc"))))]
pub mod hal_serial;

// Re-export the appropriate HAL implementation
#[cfg(all(target_arch = "riscv64", target_os = "linux", feature = "risc"))]
pub use hal_risc as hal_impl;

#[cfg(any(target_os = "macos", all(target_os = "linux", not(feature = "risc"))))]
pub use hal_serial as hal_impl;

// Public API
pub use hal::Servo;
pub use hal_impl::ServoImpl;
