//! Guest mappings, protections, allocation and checked access.
//! M0 foundation; consult the crate README before extending ownership.
pub mod access;
pub mod address;
pub mod allocation;
pub mod mapping;
pub mod protection;
pub mod regions;

// Compatibility path retained from M0/M1; implementation has one physical owner.
pub use mapping as mappings;
