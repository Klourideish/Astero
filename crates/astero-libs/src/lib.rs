//! Guest library exports and delegation to owning services.
//! M0 foundation; consult the crate README before extending ownership.

pub mod families;
pub mod nids;

/// Provider contract shared with the HLE registry.
pub use astero_hle::providers::Provider;
