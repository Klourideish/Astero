//! Source-bound dynamic reports and descriptor pairing; successful observation never grants admission.
mod descriptors;
mod inspect;
mod model;
pub use inspect::observe;
pub use model::{
    DynamicDescriptors, DynamicObservation, DynamicTable, ObservationLimits, PltDescriptor,
    PltRelocationKind, SymbolTableDescriptor, TableDescriptor,
};

pub(in crate::elf::dynamic) mod raw;
pub use raw::RawTable;
