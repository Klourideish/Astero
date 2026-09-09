use super::{DynamicFailure, DynamicLimits, DynamicObservationReport, DynamicOutcome};
use crate::{
    artifact::SourceArtifact,
    elf::{
        dynamic::{
            ObservationLimits,
            observation::{RawTable, raw::observe_raw},
        },
        inspect::bounded::{self, HeaderEvidence, InspectionLimits, InspectionOutcome},
    },
};
/// A separate request. Header decoding is discovery only, never a frontend inspection or linkage operation.
pub fn observe(source: SourceArtifact, limits: DynamicLimits) -> DynamicObservationReport {
    let outcome = match discover(&source, limits) {
        Ok(Some((_, table))) => DynamicOutcome::Complete(table),
        Ok(None) => DynamicOutcome::Unavailable,
        Err(error) => DynamicOutcome::Failed(error),
    };
    DynamicObservationReport {
        source,
        limits,
        outcome,
    }
}
/// Shared bounded discovery, so descriptor requests need not repeat header decoding.
pub(in crate::elf::dynamic) fn discover(
    source: &SourceArtifact,
    limits: DynamicLimits,
) -> Result<Option<(HeaderEvidence, RawTable)>, DynamicFailure> {
    let headers = bounded::inspect(
        source.clone(),
        InspectionLimits {
            max_program_headers: limits.max_program_headers,
        },
    );
    let evidence = match headers.into_outcome() {
        InspectionOutcome::Failed(e) => return Err(DynamicFailure::Headers(e)),
        InspectionOutcome::Complete(e) => e,
    };
    let raw = observe_raw(
        source,
        evidence.program_headers(),
        ObservationLimits {
            max_entries: limits.max_dynamic_entries,
        },
    )
    .map_err(DynamicFailure::Table)?;
    Ok(raw.map(|table| (evidence, table)))
}
