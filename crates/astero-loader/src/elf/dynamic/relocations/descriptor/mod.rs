//! Private trusted extents adapted only from M5-owned descriptors.
mod inspect;
mod model;
pub(in crate::elf::dynamic::relocations) use inspect::discover;
pub use model::{DescriptorOrigin, RelocationFormat, TableKind, TrustedRelocationExtent};
