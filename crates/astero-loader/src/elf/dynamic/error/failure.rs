use crate::{
    artifact::SourceError,
    elf::{ElfError, address_translation::TranslationError, dynamic::tags::DynamicTag},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicError {
    Header(ElfError),
    Allocation {
        entries: u64,
    },
    MultipleTables {
        first: usize,
        second: usize,
    },
    InvalidTableSize {
        segment: usize,
        file_size: u64,
        memory_size: u64,
    },
    TableAddressOverflow {
        segment: usize,
        address: u64,
        size: u64,
    },
    Source {
        segment: usize,
        error: SourceError,
    },
    Translation {
        tag: Option<DynamicTag>,
        error: TranslationError,
    },
    TableOffsetMismatch {
        segment: usize,
        declared: u64,
        translated: u64,
    },
    EntryLimit {
        limit: u64,
    },
    TruncatedEntry {
        index: u64,
        remaining: u64,
    },
    MissingTerminator {
        entries: u64,
    },
    DuplicateTag {
        tag: DynamicTag,
        first: u64,
        second: u64,
    },
    IncompleteDescriptor {
        present: DynamicTag,
        missing: DynamicTag,
    },
    UnsupportedEntrySize {
        tag: DynamicTag,
        observed: u64,
        expected: u64,
    },
    InvalidDescriptorSize {
        tag: DynamicTag,
        size: u64,
        entry_size: u64,
    },
    UnsupportedPltRelocationKind(u64),
    NeededOffsetOutOfBounds {
        index: usize,
        offset: u64,
        string_size: u64,
    },
}
impl std::fmt::Display for DynamicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ELF dynamic observation: {self:?}")
    }
}
impl std::error::Error for DynamicError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Header(e) => Some(e),
            Self::Source { error, .. } => Some(error),
            Self::Translation { error, .. } => Some(error),
            _ => None,
        }
    }
}
