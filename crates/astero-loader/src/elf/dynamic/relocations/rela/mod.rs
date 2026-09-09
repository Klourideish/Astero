//! Explicit little-endian ELF64 RELA decoding; numeric types have no executor.
mod decode;
pub use decode::RelaRecord;
pub(in crate::elf::dynamic::relocations) use decode::decode;
