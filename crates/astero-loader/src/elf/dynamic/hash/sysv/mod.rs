//! SysV array shape and chain-index integrity; no name lookup.
mod inspect;
mod model;
pub(in crate::elf::dynamic::hash) use inspect::inspect;
pub use model::SysVHash;
