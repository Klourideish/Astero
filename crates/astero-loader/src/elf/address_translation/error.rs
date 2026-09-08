use crate::{artifact::SourceError, elf::ElfError};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranslationError {
    Header(ElfError),
    InvalidLoadSize {
        segment: usize,
        file_size: u64,
        memory_size: u64,
    },
    LoadRangeOverflow {
        segment: usize,
        address: u64,
        size: u64,
    },
    Source {
        segment: usize,
        error: SourceError,
    },
    RequestOverflow {
        address: u64,
        size: u64,
    },
    Unmapped {
        address: u64,
        size: u64,
    },
    Ambiguous {
        first: usize,
        second: usize,
        address: u64,
        size: u64,
    },
    CrossesMapping {
        segment: usize,
        address: u64,
        size: u64,
    },
    ZeroFill {
        segment: usize,
        address: u64,
        size: u64,
    },
    CrossesSourceBoundary {
        segment: usize,
        address: u64,
        size: u64,
    },
}
impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ELF address translation: {self:?}")
    }
}
impl std::error::Error for TranslationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Header(e) => Some(e),
            Self::Source { error, .. } => Some(error),
            _ => None,
        }
    }
}
