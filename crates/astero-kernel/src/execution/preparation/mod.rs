//! Native runtime preparation mechanisms, never guest execution.
pub mod boundary;
pub mod layout;
#[cfg(all(windows, target_arch = "x86_64"))]
pub mod storage;
pub use astero_abi::layouts::entry::{InitialContext, process_arguments};
