//! Native bridge ownership, synthetic probes and checked native-owner execution leases.
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
mod platform;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use platform::{Bridge, BridgeError, BridgeLease, NativeExit, Supervision, SyntheticProbe};
