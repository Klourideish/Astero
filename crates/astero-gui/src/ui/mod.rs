//! ImGui session and read-only evidence presentation.
//! Guest mechanisms and linkage classification remain outside this crate.
pub mod debugger;
pub mod diagnostics;
pub mod gpu;
pub mod logs;
pub mod memory;
pub mod modules;
pub mod session;
pub mod threads;
mod toolkit;
pub(crate) use toolkit::Toolkit;
