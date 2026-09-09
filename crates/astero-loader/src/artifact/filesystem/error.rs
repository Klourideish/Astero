use crate::artifact::SourceError;
use std::{
    collections::TryReserveError,
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    InspectPath,
    Open,
    InspectHandle,
    Allocate,
    Read,
    ProbeEnd,
    RecheckHandle,
    ConstructSource,
}

#[derive(Debug)]
pub enum Failure {
    Io(io::Error),
    NotRegularFile,
    SizeLimit {
        observed: u64,
        maximum: u64,
    },
    LengthUnrepresentable {
        observed: u64,
    },
    Allocation {
        requested: usize,
        source: TryReserveError,
    },
    ReadBudget {
        maximum: u64,
        completed_bytes: u64,
    },
    ShortRead {
        expected: u64,
        actual: u64,
    },
    GrewDuringRead {
        expected: u64,
        observed_at_least: u64,
    },
    SizeChanged {
        before: u64,
        after: u64,
    },
    Source(SourceError),
}

/// Requested native path and failing operation survive every adapter failure.
#[derive(Debug)]
pub struct AcquisitionError {
    pub path: PathBuf,
    pub operation: Operation,
    pub failure: Failure,
}
impl AcquisitionError {
    pub(super) fn new(path: &Path, operation: Operation, failure: Failure) -> Self {
        Self {
            path: path.to_owned(),
            operation,
            failure,
        }
    }
}
impl std::fmt::Display for AcquisitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "file acquisition {:?} at {:?}: {:?}",
            self.operation, self.path, self.failure
        )
    }
}
impl std::error::Error for AcquisitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            Failure::Io(e) => Some(e),
            Failure::Allocation { source, .. } => Some(source),
            Failure::Source(e) => Some(e),
            _ => None,
        }
    }
}
