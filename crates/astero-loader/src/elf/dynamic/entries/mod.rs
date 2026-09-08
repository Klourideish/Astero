//! Ordinary ELF64 dynamic entry decoding inside the validated table extent.
mod decode;
pub use decode::DynamicEntry;
pub(in crate::elf::dynamic) use decode::decode;
