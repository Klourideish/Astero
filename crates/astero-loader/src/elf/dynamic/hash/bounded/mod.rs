//! Explicit metadata/count evidence only; never a symbol enumeration request.
mod collect;
mod model;
pub use super::{HashLimits, HashObservation};
pub use collect::observe;
pub use model::{HashMetadataFailure, HashMetadataLimits, HashMetadataOutcome, HashMetadataReport};
