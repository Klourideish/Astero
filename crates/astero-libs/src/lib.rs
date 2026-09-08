//! Guest library exports and delegation to owning services.
//! M0 foundation; consult the crate README before extending ownership.
pub mod agc;
pub mod audio;
pub mod families;
pub mod filesystem;
pub mod kernel;
pub mod libc;
pub mod network;
pub mod nids;
pub mod pthread;
pub mod runtime;
pub mod sysmodule;
pub mod videoout;

/// Provider contract shared with the HLE registry.
pub use astero_hle::providers::Provider;
