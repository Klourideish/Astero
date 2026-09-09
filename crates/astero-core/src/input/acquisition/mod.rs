//! Acquisition-only application entry point; no inspection or session attachment.
pub use astero_loader::artifact::{
    SourceArtifact,
    filesystem::{AcquisitionError, AcquisitionLimits, Failure, Operation},
};
use std::path::Path;

/// Delegate to the single loader-owned reader. Success is not admission or loading.
pub fn acquire(path: &Path, limits: AcquisitionLimits) -> Result<SourceArtifact, AcquisitionError> {
    astero_loader::artifact::filesystem::acquire(path, limits)
}
