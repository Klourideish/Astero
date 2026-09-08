//! Planned ui ownership within astero-gui.
//! Structural home only; no additional functionality or capability is implemented.
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
