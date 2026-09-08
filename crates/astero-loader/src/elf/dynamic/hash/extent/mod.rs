//! Provenance-bearing exact symbol bounds. Detached claims cannot authorize enumeration.
mod derive;
mod model;
pub(in crate::elf::dynamic::hash) use derive::derive;
pub use model::{CountClaim, ExtentEvidence, TrustedSymbolExtent};
