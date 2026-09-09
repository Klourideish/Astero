//! Bounded dynamic relocation records and reference evidence; no application semantics.
pub mod descriptor;
pub mod error;
pub mod observation;
pub mod plt;
pub mod rela;
pub mod symbol_reference;
pub use observation::{
    RelocationIterator, RelocationLimits, RelocationObservation, RelocationTables,
};
