//! Native bridge ownership. Public execution is restricted to built-in synthetic probes.
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
mod platform;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use platform::{Bridge, BridgeError, NativeExit, SyntheticProbe};
