use super::{DynamicFailure, DynamicLimits, DynamicObservationReport, DynamicOutcome};
use crate::{
    artifact::SourceArtifact,
    elf::{
        dynamic::{ObservationLimits, observation::raw::observe_raw},
        inspect::bounded::{self, InspectionLimits, InspectionOutcome},
    },
};
/// A separate request. Header decoding is discovery only, never a frontend inspection or linkage operation.
pub fn observe(source: SourceArtifact, limits: DynamicLimits) -> DynamicObservationReport {
    let headers = bounded::inspect(
        source.clone(),
        InspectionLimits {
            max_program_headers: limits.max_program_headers,
        },
    );
    let outcome = match headers.into_outcome() {
        InspectionOutcome::Failed(e) => DynamicOutcome::Failed(DynamicFailure::Headers(e)),
        InspectionOutcome::Complete(e) => match observe_raw(
            &source,
            e.program_headers(),
            ObservationLimits {
                max_entries: limits.max_dynamic_entries,
            },
        ) {
            Ok(Some(table)) => DynamicOutcome::Complete(table),
            Ok(None) => DynamicOutcome::Unavailable,
            Err(e) => DynamicOutcome::Failed(DynamicFailure::Table(e)),
        },
    };
    DynamicObservationReport {
        source,
        limits,
        outcome,
    }
}
