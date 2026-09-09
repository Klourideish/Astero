//! Trusted index validation and symbol observation, never import/export interpretation.
mod validate;
pub(in crate::elf::dynamic::relocations) use validate::validate;
pub use validate::{SymbolReference, SymbolReferenceKind};
