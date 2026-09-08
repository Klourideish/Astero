//! Source-bound hash metadata evidence, never runtime symbol lookup.
pub mod error;
pub mod extent;
pub mod gnu;
pub mod observation;
pub mod sysv;
pub use observation::{HashLimits, HashObservation, observe};
