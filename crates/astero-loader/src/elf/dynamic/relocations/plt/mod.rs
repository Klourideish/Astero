//! JMPREL provenance and supported aligned RELA tail alias relationships.
mod alias;
pub use alias::TailAlias;
pub(in crate::elf::dynamic::relocations) use alias::classify;
