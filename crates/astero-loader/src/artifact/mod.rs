//! Artifact contracts; see the loader architecture record.
mod inspection;
pub use inspection::*;
mod source;
pub use source::{BoundSourceRange, SourceArtifact, SourceError, SourceId};
