//! Binary parsing, load plans, relocations and import metadata.
//! M0 foundation; consult the crate README before extending ownership.
pub mod admission;
pub mod artifact;
pub mod dependencies;
pub mod elf;
pub mod exports;
pub mod imports;
pub mod load_plan;
pub mod metadata;
pub mod modules;
pub mod relocations;
pub mod self_format;

// Compatibility path retained from M0/M1; implementation has one physical owner.
pub use load_plan as loading;
