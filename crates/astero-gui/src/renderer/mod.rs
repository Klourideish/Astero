//! GUI-only host rendering, independent of astero-gpu.
// Only the private Ash FFI boundary may opt in to unsafe.
#[allow(unsafe_code)]
pub(crate) mod vulkan;
