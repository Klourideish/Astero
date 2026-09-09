use super::{AcquisitionError, Failure, Operation, read::read_exact_extent};
use crate::artifact::SourceArtifact;
use std::{
    fs::{self, File},
    path::Path,
};

/// Caller-selected resource policy, not an ELF/PS5 size rule. There is no implicit default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcquisitionLimits {
    /// Maximum accepted bytes; zero permits an empty file only.
    pub max_bytes: u64,
    /// Includes short reads, Interrupted retries and the one-byte EOF probe.
    pub max_read_calls: u64,
}

/// Acquire one regular file, then STOP. No format detection, inspection or session mutation.
/// Observed final symlinks are rejected before open; this is not a race-proof path sandbox.
pub fn acquire(
    path: impl AsRef<Path>,
    limits: AcquisitionLimits,
) -> Result<SourceArtifact, AcquisitionError> {
    let path = path.as_ref();
    let err = |op, failure| AcquisitionError::new(path, op, failure);
    let before_open =
        fs::symlink_metadata(path).map_err(|e| err(Operation::InspectPath, Failure::Io(e)))?;
    if !before_open.file_type().is_file() {
        return Err(err(Operation::InspectPath, Failure::NotRegularFile));
    }
    let mut file = File::open(path).map_err(|e| err(Operation::Open, Failure::Io(e)))?;
    let metadata = file
        .metadata()
        .map_err(|e| err(Operation::InspectHandle, Failure::Io(e)))?;
    if !metadata.is_file() {
        return Err(err(Operation::InspectHandle, Failure::NotRegularFile));
    }
    // Only opened-handle metadata supplies the candidate length. Read results remain authoritative.
    let expected = metadata.len();
    let bytes = read_exact_extent(&mut file, expected, limits, path)?;
    let after = file
        .metadata()
        .map_err(|e| err(Operation::RecheckHandle, Failure::Io(e)))?;
    check_final_size(path, expected, after.len())?;
    // Escaped display provenance is not a canonical path or identity; never used to reopen input.
    SourceArtifact::new(bytes, Some(format!("filesystem input: {path:?}")))
        .map_err(|e| err(Operation::ConstructSource, Failure::Source(e)))
}
pub(super) fn check_final_size(
    path: &Path,
    before: u64,
    after: u64,
) -> Result<(), AcquisitionError> {
    if before != after {
        return Err(AcquisitionError::new(
            path,
            Operation::RecheckHandle,
            Failure::SizeChanged { before, after },
        ));
    }
    Ok(())
}
