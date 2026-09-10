//! Explicit realization of an existing M27 image; no restaging, relocation or execution.
pub use astero_memory::mapping::windows_native::{
    NativeError, NativeLimits, NativeObserver, NativePage, NativeSnapshot,
};
#[cfg(all(windows, target_arch = "x86_64"))]
mod compose;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use compose::*;
