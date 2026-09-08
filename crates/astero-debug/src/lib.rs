//! Read-only session inspection and explicit unsupported guest operations.
pub mod breakpoints;
pub mod context;
pub mod faults;
pub mod gpu;
pub mod memory;
pub mod modules;
pub mod nids;
pub mod sampling;
pub mod session;
pub mod snapshots;
pub mod symbols;
pub mod threads;
pub mod tracing;
pub mod watchpoints;

// Compatibility path retained from M0/M1; implementation has one physical owner.
pub use session::{capabilities, control, inspection};
