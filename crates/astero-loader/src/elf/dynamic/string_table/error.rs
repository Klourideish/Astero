use crate::artifact::{BoundSourceRange, SourceError};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringTableError {
    Source(SourceError),
    OffsetOutOfBounds {
        offset: u64,
        table: BoundSourceRange,
    },
    MissingTerminator {
        offset: u64,
        table: BoundSourceRange,
    },
    ScanLimit {
        offset: u64,
        table: BoundSourceRange,
        limit: u64,
    },
}
impl std::fmt::Display for StringTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dynamic string table: {self:?}")
    }
}
impl std::error::Error for StringTableError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(e) => Some(e),
            _ => None,
        }
    }
}
