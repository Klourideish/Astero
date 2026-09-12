//! Private Ash FFI boundary. See architecture/gui_framework.md for safety/lifetime rules.
mod context;
mod rendering;
mod swapchain;
pub(crate) use rendering::Graphics;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod pixels;
