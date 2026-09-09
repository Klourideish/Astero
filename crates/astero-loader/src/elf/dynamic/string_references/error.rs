use crate::elf::dynamic::{
    descriptors::{DescriptorObservationReport, DescriptorOutcome},
    entries::DynamicEntry,
    string_table::StringTableError,
};
#[derive(Debug)]
pub enum StringReferenceFailure {
    /// Retains the entire failed prerequisite report, including its immutable raw audit evidence.
    Prerequisite(Box<DescriptorObservationReport>),
    ReferenceBudget {
        observed_references: u64,
        maximum: u64,
    },
    TableUnavailable {
        reference: DynamicEntry,
    },
    Table(StringTableError),
    Lookup {
        reference: DynamicEntry,
        completed_references: u64,
        effective_scan_limit: u64,
        remaining_total_scan_bytes: u64,
        error: StringTableError,
    },
    Allocation {
        requested_references: u64,
    },
}
impl std::fmt::Display for StringReferenceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Prerequisite(report) = self {
            return write!(f, "String prerequisite failed: {:?}", report.outcome());
        }
        write!(f, "String reference observation: {self:?}")
    }
}
impl std::error::Error for StringReferenceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Prerequisite(r) => match r.outcome() {
                DescriptorOutcome::Failed(e) => Some(e),
                _ => None,
            },
            Self::Table(e) | Self::Lookup { error: e, .. } => Some(e),
            _ => None,
        }
    }
}
