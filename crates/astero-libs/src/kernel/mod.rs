//! Guest kernel contracts; mechanisms remain in astero-kernel.
#[cfg(all(windows, target_arch = "x86_64"))]
pub mod memory;
pub mod semaphore;
pub mod timing;
