use super::*;
use crate::elf::dynamic::{
    bounded,
    candidates::workload::{LinkageOutcome, LinkageReport},
    observation,
    string_table::DynamicStringTable,
    tags::DynamicTag,
};
use std::sync::Arc;
/// Explicitly consume one existing complete M23 report. No symbols or relocations are re-enumerated.
pub fn observe(linkage: Arc<LinkageReport>, limits: IdentityLimits) -> Ps5IdentityEvidenceReport {
    let outcome = match linkage.outcome() {
        LinkageOutcome::Unavailable => IdentityOutcome::Unavailable,
        LinkageOutcome::Failed(_) => IdentityOutcome::Failed(IdentityFailure::LinkagePrerequisite),
        LinkageOutcome::Complete(_) => match collect(&linkage, limits) {
            Ok(e) => IdentityOutcome::Complete(e),
            Err(e) => IdentityOutcome::Failed(e),
        },
    };
    Ps5IdentityEvidenceReport {
        linkage,
        limits,
        outcome,
    }
}
fn collect(
    linkage: &LinkageReport,
    limits: IdentityLimits,
) -> Result<IdentityEvidence, IdentityFailure> {
    use IdentityFailure as F;
    let LinkageOutcome::Complete(previous) = linkage.outcome() else {
        return Err(F::LinkagePrerequisite);
    };
    let raw = linkage
        .symbols()
        .input()
        .proof()
        .raw()
        .ok_or(F::LinkagePrerequisite)?;
    let sce = |tag| matches!(tag, DynamicTag::Unknown(0x61000000..=0x6100ffff));
    let count =
        raw.entries().iter().filter(|e| sce(e.tag)).count() as u64 + previous.counts().symbols;
    if count > limits.max_identity_records {
        return Err(F::Budget {
            count,
            maximum: limits.max_identity_records,
        });
    }
    // Reuse existing bounded descriptor translation; never interpret payloads other than bounded names.
    let source = linkage.source();
    let Some((headers, table)) =
        bounded::collect::discover(source, linkage.limits().hash.dynamic).map_err(F::Discovery)?
    else {
        return Err(F::LinkagePrerequisite);
    };
    let table =
        observation::from_raw(source, headers.program_headers(), table).map_err(F::Dynamic)?;
    let strings = DynamicStringTable::from_dynamic(&table).map_err(|error| F::String {
        dynamic_index: 0,
        error,
    })?;
    let name_limits = linkage.limits().symbols;
    let mut attempts = previous.needed().len() as u64;
    let mut spent = previous
        .needed()
        .iter()
        .map(|n| n.source.extent().size + 1)
        .sum::<u64>();
    for (_, s) in linkage.symbols().entries() {
        if let Some(n) = s.name_bytes() {
            attempts += 1;
            spent += n.len() as u64 + 1;
        }
    }
    let mut remaining = name_limits.max_total_name_scan_bytes - spent;
    let mut descriptors = Vec::new();
    for e in raw.entries().iter().filter(|e| sce(e.tag)) {
        let DynamicTag::Unknown(tag) = e.tag else {
            unreachable!()
        };
        let kind = metadata::kind(tag);
        let (id, version_bits, name) = if kind == DescriptorKind::Unsupported {
            (None, None, None)
        } else {
            attempts += 1;
            if attempts > name_limits.max_name_lookups {
                return Err(F::NameLookupBudget {
                    attempted: attempts,
                    maximum: name_limits.max_name_lookups,
                });
            }
            let name = strings
                .as_ref()
                .ok_or(F::StringsUnavailable {
                    dynamic_index: e.index,
                })?
                .lookup(
                    e.value & 0xffff_ffff,
                    name_limits.max_name_scan_bytes.min(remaining),
                )
                .map_err(|error| F::String {
                    dynamic_index: e.index,
                    error,
                })?;
            remaining -= name.scanned_bytes();
            (
                Some((e.value >> 48) as u16),
                Some((e.value >> 32) as u16),
                Some(name.source_range()),
            )
        };
        descriptors
            .try_reserve(1)
            .map_err(|_| F::Allocation { records: count })?;
        descriptors.push(IdentityDescriptor {
            dynamic_index: e.index,
            tag,
            raw: e.value,
            kind,
            evidence: if kind == DescriptorKind::Unsupported {
                MetadataEvidence::Uninterpreted
            } else {
                MetadataEvidence::ExperimentalPacking
            },
            id,
            version_bits,
            name,
        });
    }
    let mut symbols = Vec::new();
    for (_, s) in linkage.symbols().entries() {
        symbols
            .try_reserve(1)
            .map_err(|_| F::Allocation { records: count })?;
        symbols.push(EncodedSymbol {
            symbol_index: s.fields().index,
            name: s.name_range(),
            evidence: metadata::correlate(s.name_bytes(), &descriptors),
        });
    }
    Ok(IdentityEvidence {
        descriptors,
        symbols,
    })
}
