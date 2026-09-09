//! Explicit selected descriptor metadata; no table payload reads or runtime state.
mod collect;
mod model;
pub use collect::observe;
pub use model::{
    DescriptorFailure, DescriptorFamily, DescriptorLimits, DescriptorObservationReport,
    DescriptorOutcome, DescriptorRecord, DescriptorUnavailable, DescriptorValue,
};
