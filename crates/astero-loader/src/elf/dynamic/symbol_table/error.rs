use crate::{
    artifact::SourceError,
    elf::{
        address_translation::TranslationError,
        dynamic::{error::DynamicError, string_table::StringTableError},
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolError {
    Dynamic(DynamicError),
    TableUnavailable,
    CountUnavailable,
    Hash(crate::elf::dynamic::hash::error::HashError),
    EnumerationLimit { count: u64, limit: u64 },
    IndexOverflow { index: u64 },
    Translation { index: u64, error: TranslationError },
    Source(SourceError),
    StringsUnavailable { index: u64, offset: u32 },
    Name { index: u64, error: StringTableError },
    InvalidNullSymbol,
}
impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SymbolError {}
