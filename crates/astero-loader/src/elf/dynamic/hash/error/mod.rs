//! Hash-specific failures retain source identity and virtual location.
use crate::{
    artifact::{SourceError, SourceId},
    elf::{address_translation::TranslationError, dynamic::error::DynamicError},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashKind {
    SysV,
    Gnu,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HashFailure {
    Overflow,
    Translation(TranslationError),
    Source(SourceError),
    WorkLimit,
    ZeroBuckets,
    ZeroSymbols,
    InvalidBloomSize(u32),
    InvalidSymbolOffset(u32),
    InvalidReference {
        index: u64,
        value: u32,
        count: u32,
    },
    Cycle {
        bucket: u32,
    },
    InvalidBucketLayout {
        bucket: u32,
        expected: u64,
        observed: u32,
    },
    MissingChainTerminator {
        symbol: u64,
        error: TranslationError,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HashError {
    Dynamic(DynamicError),
    Mapping(TranslationError),
    At {
        source: SourceId,
        kind: HashKind,
        address: u64,
        failure: HashFailure,
    },
    MissingSymbolDescriptor,
    ConflictingEvidence {
        source: SourceId,
        sysv: u64,
        gnu: u64,
        gnu_exact: bool,
    },
    SymbolExtentOverflow {
        count: u64,
        entry_size: u64,
    },
    SymbolExtent {
        count: u64,
        error: TranslationError,
    },
}
impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HashError {}
