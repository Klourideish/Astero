use super::{
    DescriptorFailure, DescriptorLimits, DescriptorObservationReport, DescriptorOutcome,
    DescriptorRecord, DescriptorUnavailable, DescriptorValue,
};
use crate::{
    artifact::SourceArtifact,
    elf::{
        address_translation::AddressTranslator,
        dynamic::{
            bounded::collect::discover,
            entries::DynamicEntry,
            error::DynamicError,
            observation::{RawTable, descriptors},
            tags::DynamicTag,
        },
        inspect::bounded::HeaderEvidence,
    },
};
use DynamicTag::{StrSz, StrTab, SymEnt, SymTab};
/// One explicit request, with no dependency on a prior frontend operation.
pub fn observe(source: SourceArtifact, limits: DescriptorLimits) -> DescriptorObservationReport {
    let (raw, outcome) = match discover(&source, limits.dynamic) {
        Err(e) => (
            None,
            DescriptorOutcome::Failed(DescriptorFailure::Discovery(e)),
        ),
        Ok(None) => (
            None,
            DescriptorOutcome::Unavailable(DescriptorUnavailable::NoDynamicTable),
        ),
        Ok(Some((headers, raw))) => {
            let outcome = match collect(&source, &headers, &raw, limits.max_descriptors) {
                Ok(records) if records.is_empty() => {
                    DescriptorOutcome::Unavailable(DescriptorUnavailable::NoSupportedDescriptors)
                }
                Ok(records) => DescriptorOutcome::Complete(records),
                Err(e) => DescriptorOutcome::Failed(e),
            };
            (Some(raw), outcome)
        }
    };
    DescriptorObservationReport {
        source,
        limits,
        raw,
        outcome,
    }
}
fn supported(tag: DynamicTag) -> bool {
    matches!(tag, StrTab | StrSz | SymTab | SymEnt)
}
fn fields(raw: &RawTable, tags: [DynamicTag; 2]) -> Result<[DynamicEntry; 2], DescriptorFailure> {
    let get = |tag| {
        raw.entries().iter().find(|e| e.tag == tag).copied().ok_or(
            DescriptorFailure::Interpretation(DynamicError::IncompleteDescriptor {
                present: tags[0],
                missing: tag,
            }),
        )
    };
    Ok([get(tags[0])?, get(tags[1])?])
}
fn collect(
    source: &SourceArtifact,
    headers: &HeaderEvidence,
    raw: &RawTable,
    maximum: u64,
) -> Result<Vec<DescriptorRecord>, DescriptorFailure> {
    let strings = raw
        .entries()
        .iter()
        .any(|e| matches!(e.tag, StrTab | StrSz));
    let symbols = raw
        .entries()
        .iter()
        .any(|e| matches!(e.tag, SymTab | SymEnt));
    let count = u64::from(strings) + u64::from(symbols);
    if count > maximum {
        return Err(DescriptorFailure::Budget {
            attempted_families: count,
            maximum,
        });
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    let translator = AddressTranslator::from_program_headers(
        source,
        headers.program_headers().iter().cloned().map(Ok),
    )
    .map_err(|error| {
        DescriptorFailure::Interpretation(DynamicError::Translation { tag: None, error })
    })?;
    // Existing authoritative pairing and extent checks, limited to the explicit supported set.
    let selected = descriptors::collect(
        raw.entries().iter().filter(|e| supported(e.tag)),
        &translator,
    )
    .map_err(DescriptorFailure::Interpretation)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(count as usize)
        .map_err(|_| DescriptorFailure::Allocation {
            requested_descriptors: count,
        })?;
    if let Some(value) = selected.strings {
        records.push(DescriptorRecord {
            fields: fields(raw, [StrTab, StrSz])?,
            value: DescriptorValue::Strings(value),
        });
    }
    if let Some(value) = selected.symbols {
        records.push(DescriptorRecord {
            fields: fields(raw, [SymTab, SymEnt])?,
            value: DescriptorValue::Symbols(value),
        });
    }
    Ok(records)
}
