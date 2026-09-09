//! Discovery, bounded hash-byte access and explicit shared work accounting.
mod bytes;
mod inspect;
mod model;
pub(in crate::elf::dynamic::hash) use bytes::HashReader;
pub use inspect::observe;
pub use model::{HashLimits, HashObservation};

pub(in crate::elf::dynamic::hash) use inspect::collect;
