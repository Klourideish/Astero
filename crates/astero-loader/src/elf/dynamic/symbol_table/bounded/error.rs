use crate::{
    artifact::{SourceError, SourceId},
    elf::dynamic::{descriptors::DescriptorObservationReport, symbol_table::SymbolError},
};
#[derive(Debug)]
pub enum SymbolObservationFailure {
    SourceMismatch {
        source: SourceId,
        proof: SourceId,
    },
    EvidenceFailed,
    Descriptors(Box<DescriptorObservationReport>),
    ExtentMismatch,
    Source(SourceError),
    EntryBudget {
        count: u64,
        maximum: u64,
    },
    Allocation {
        count: u64,
    },
    NameLookupBudget {
        index: u64,
        offset: u32,
        attempted: u64,
        maximum: u64,
    },
    Symbol {
        index: u64,
        completed: u64,
        lookups: u64,
        remaining_scan_bytes: u64,
        error: SymbolError,
    },
}
impl std::fmt::Display for SymbolObservationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Descriptors(r) => write!(f, "symbol descriptors: {:?}", r.outcome()),
            _ => write!(f, "symbol observation: {self:?}"),
        }
    }
}
impl std::error::Error for SymbolObservationFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(e) => Some(e),
            Self::Symbol { error, .. } => Some(error),
            _ => None,
        }
    }
}
