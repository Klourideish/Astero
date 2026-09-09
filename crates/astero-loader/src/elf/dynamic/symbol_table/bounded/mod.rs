//! Explicit exact-proof consumer; no linkage classification or runtime state.
mod collect;
mod error;
mod model;
pub use crate::elf::dynamic::hash::bounded::HashMetadataReport;
pub use collect::observe;
pub use error::SymbolObservationFailure;
pub use model::{SymbolObservationLimits, SymbolObservationReport, SymbolOutcome, SymbolRecord};
