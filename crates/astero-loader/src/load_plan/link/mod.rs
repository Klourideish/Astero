//! Immutable multi-artifact planning. No mappings, dispatch or filesystem search.
mod collect;
mod model;
mod providers;
mod relocations;
mod segments;
pub use collect::plan;
pub use model::*;
