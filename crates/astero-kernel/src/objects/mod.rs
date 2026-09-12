//! Kernel object identity, handle lookup and lifetime ownership.
//! Object-specific mechanisms stay with synchronization, threading or filesystem.
#[cfg(all(windows, target_arch = "x86_64"))]
pub mod memory;
