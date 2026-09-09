use super::{
    HeaderEvidence, InspectionFailure, InspectionLimits, InspectionOutcome, InspectionReport,
};
use crate::{
    artifact::SourceArtifact,
    elf::{header, program_headers::ProgramHeaders},
};

/// One explicit request. All-or-failure evidence; never silently keeps an observed prefix.
pub fn inspect(source: SourceArtifact, limits: InspectionLimits) -> InspectionReport {
    let outcome = match collect(&source, limits) {
        Ok(evidence) => InspectionOutcome::Complete(evidence),
        Err(error) => InspectionOutcome::Failed(error),
    };
    InspectionReport {
        source,
        limits,
        outcome,
    }
}
fn collect(
    source: &SourceArtifact,
    limits: InspectionLimits,
) -> Result<HeaderEvidence, InspectionFailure> {
    // Existing fixed-size decoder also proves the full program table range without iterating entries.
    let header = header::decode(source).map_err(InspectionFailure::Header)?;
    if u64::from(header.program_count) > limits.max_program_headers {
        return Err(InspectionFailure::HeaderBudget {
            declared: header.program_count,
            maximum: limits.max_program_headers,
        });
    }
    let mut programs = Vec::new();
    programs
        .try_reserve_exact(usize::from(header.program_count))
        .map_err(|source| InspectionFailure::Allocation {
            requested_headers: header.program_count,
            source,
        })?;
    for (index, p) in ProgramHeaders::new(source, &header).enumerate() {
        programs.push(p.map_err(|error| InspectionFailure::ProgramHeader {
            index: index as u16,
            error,
        })?);
    }
    Ok(HeaderEvidence { header, programs })
}
