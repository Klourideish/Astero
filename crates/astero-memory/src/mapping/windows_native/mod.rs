//! Explicit native identity placement. No entry pointer or guest execution API.
mod layout;
mod model;
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
mod platform;
pub use layout::plan_layout;
pub use model::*;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use platform::{NativeImage, host_geometry, realize};
#[cfg(not(all(windows, target_arch = "x86_64")))]
pub fn host_geometry() -> Result<Geometry, NativeError> {
    Err(NativeError::UnsupportedHost)
}
