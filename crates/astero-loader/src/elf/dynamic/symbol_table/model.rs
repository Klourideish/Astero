use crate::{artifact::BoundSourceRange, elf::dynamic::string_table::StringReference};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Local,
    Global,
    Weak,
    Unknown(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolType {
    NoType,
    Object,
    Function,
    Section,
    File,
    Common,
    Tls,
    Unknown(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Default,
    Internal,
    Hidden,
    Protected,
    Unknown(u8),
}
/// Structural section designation only; no section existence or export eligibility is certified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Undefined,
    Index(u16),
    Absolute,
    Common,
    Extended,
    Reserved(u16),
}
/// Detached value observation with borrowed name bytes. Not a linker/import/export record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicSymbolObservation<'a> {
    pub index: u64,
    pub source: BoundSourceRange,
    pub name_offset: u32,
    pub name: Option<StringReference<'a>>,
    pub info: u8,
    pub other: u8,
    pub binding: Binding,
    pub symbol_type: SymbolType,
    pub visibility: Visibility,
    pub section: Section,
    pub value: u64,
    pub size: u64,
}

/// Raw ELF64 fields and established structural views; no linkage classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolFields {
    pub index: u64,
    pub source: crate::artifact::BoundSourceRange,
    pub name_offset: u32,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
    pub binding: Binding,
    pub symbol_type: SymbolType,
    pub visibility: Visibility,
    pub section: Section,
    pub value: u64,
    pub size: u64,
}
