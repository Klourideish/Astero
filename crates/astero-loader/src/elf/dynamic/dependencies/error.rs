use crate::{
    dependencies::DependencyNameError,
    elf::dynamic::{error::DynamicError, string_table::StringTableError},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyObservationError {
    Dynamic(DynamicError),
    StringTable(StringTableError),
    TableUnavailable {
        entry_index: u64,
        offset: u64,
    },
    Lookup {
        entry_index: u64,
        error: StringTableError,
    },
    InvalidName {
        entry_index: u64,
        offset: u64,
        error: DependencyNameError,
    },
}
impl std::fmt::Display for DependencyObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ELF dependency observation: {self:?}")
    }
}
impl std::error::Error for DependencyObservationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dynamic(e) => Some(e),
            Self::StringTable(e) | Self::Lookup { error: e, .. } => Some(e),
            Self::InvalidName { error, .. } => Some(error),
            _ => None,
        }
    }
}
