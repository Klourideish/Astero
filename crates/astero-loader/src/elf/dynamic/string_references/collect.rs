use super::{
    StringEncoding, StringReferenceFailure, StringReferenceLimits,
    StringReferenceObservationReport, StringReferenceOutcome, StringReferenceRecord,
};
use crate::{
    artifact::SourceArtifact,
    elf::dynamic::{
        descriptors::{self, DescriptorOutcome, DescriptorValue},
        string_table::DynamicStringTable,
        tags::DynamicTag,
    },
};
/// Each call is explicit. Only supported raw references trigger M6 lookup; no dependency adapter is called.
pub fn observe(
    source: SourceArtifact,
    limits: StringReferenceLimits,
) -> StringReferenceObservationReport {
    let outcome = match collect(&source, limits) {
        Ok(records) if records.is_empty() => StringReferenceOutcome::Unavailable,
        Ok(records) => StringReferenceOutcome::Complete(records),
        Err(e) => StringReferenceOutcome::Failed(e),
    };
    StringReferenceObservationReport {
        source,
        limits,
        outcome,
    }
}
fn collect(
    source: &SourceArtifact,
    limits: StringReferenceLimits,
) -> Result<Vec<StringReferenceRecord>, StringReferenceFailure> {
    let prerequisite = descriptors::observe(source.clone(), limits.descriptors);
    if matches!(prerequisite.outcome(), DescriptorOutcome::Failed(_)) {
        return Err(StringReferenceFailure::Prerequisite(Box::new(prerequisite)));
    }
    let Some(raw) = prerequisite.raw() else {
        return Ok(Vec::new());
    };
    let count = raw
        .entries()
        .iter()
        .filter(|e| e.tag == DynamicTag::Needed)
        .count();
    if count as u64 > limits.max_string_references {
        return Err(StringReferenceFailure::ReferenceBudget {
            observed_references: count as u64,
            maximum: limits.max_string_references,
        });
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    let descriptor = match prerequisite.outcome() {
        DescriptorOutcome::Complete(records) => records
            .iter()
            .find(|r| matches!(r.value(), DescriptorValue::Strings(_))),
        _ => None,
    };
    let table = descriptor
        .map(|r| DynamicStringTable::from_descriptor(source, r))
        .transpose()
        .map_err(StringReferenceFailure::Table)?
        .flatten();
    let mut records = Vec::new();
    records
        .try_reserve_exact(count)
        .map_err(|_| StringReferenceFailure::Allocation {
            requested_references: count as u64,
        })?;
    let mut remaining = limits.strings.max_total_scan_bytes;
    for entry in raw.entries().iter().filter(|e| e.tag == DynamicTag::Needed) {
        let strings = table
            .as_ref()
            .ok_or(StringReferenceFailure::TableUnavailable { reference: *entry })?;
        let effective = remaining.min(limits.strings.max_scan_bytes_per_reference);
        let view = strings.lookup(entry.value, effective).map_err(|error| {
            StringReferenceFailure::Lookup {
                reference: *entry,
                completed_references: records.len() as u64,
                effective_scan_limit: effective,
                remaining_total_scan_bytes: remaining,
                error,
            }
        })?;
        remaining -= view.scanned_bytes(); // M6 success proves the charge fits the effective limit.
        let encoding = if view.as_bytes().is_empty() {
            StringEncoding::Empty
        } else if view.as_utf8().is_ok() {
            StringEncoding::Utf8
        } else {
            StringEncoding::RawBytes
        };
        records.push(StringReferenceRecord {
            entry: *entry,
            source: source.clone(),
            range: view.source_range(),
            encoding,
        });
    }
    Ok(records)
}
