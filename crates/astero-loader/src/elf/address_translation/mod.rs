//! Source-backed ELF virtual-range translation; no runtime load bias or memory mappings.
mod error;
mod translate;
pub use error::TranslationError;
pub use translate::AddressTranslator;
