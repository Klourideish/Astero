//! Independent host GUI. Session APIs remain toolkit-independent.
mod toolkit;
pub mod view_model;
mod windowing;
// Ash exposes Vulkan's unsafe FFI. Only this private module may opt in.
#[allow(unsafe_code)]
mod vulkan;
pub use windowing::run;
