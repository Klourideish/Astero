use super::*;
use crate::{
    artifact::BoundSourceRange,
    elf::dynamic::{
        candidates::workload::LinkageOutcome,
        identity::{
            ContextEvidence, DescriptorKind, IdentityOutcome, NameEvidence,
            Ps5IdentityEvidenceReport,
        },
        symbol_table::{Binding, Section, SymbolType, Visibility},
    },
};
/// Only unambiguous artifact-local context declarations. IDs are never compared across sources.
fn context(
    r: &Ps5IdentityEvidenceReport,
    c: ContextEvidence,
    library: bool,
    definition: bool,
) -> Option<(BoundSourceRange, u16)> {
    let ContextEvidence::Hypothesis { dynamic_index, .. } = c else {
        return None;
    };
    let IdentityOutcome::Complete(e) = r.outcome() else {
        return None;
    };
    let d = e
        .descriptors()
        .iter()
        .find(|d| d.dynamic_index == dynamic_index)?;
    let valid = if library {
        if definition {
            d.kind == DescriptorKind::ExportLibrary
        } else {
            d.kind == DescriptorKind::ImportLibrary
        }
    } else if definition {
        d.kind == DescriptorKind::Module
    } else {
        matches!(
            d.kind,
            DescriptorKind::Module | DescriptorKind::NeededModule
        )
    };
    if valid {
        Some((d.name?, d.version_bits?))
    } else {
        None
    }
}
fn key(
    r: &Ps5IdentityEvidenceReport,
    index: u64,
    definition: bool,
) -> Option<(u64, BoundSourceRange, BoundSourceRange, u16, u16)> {
    let IdentityOutcome::Complete(e) = r.outcome() else {
        return None;
    };
    let s = e.symbols().iter().find(|s| s.symbol_index == index)?;
    let NameEvidence::Encoded {
        nid,
        library,
        module,
    } = s.evidence
    else {
        return None;
    };
    let (library, lv) = context(r, library, true, definition)?;
    let (module, mv) = context(r, module, false, definition)?;
    Some((nid, library, module, lv, mv))
}
fn bytes(r: &Ps5IdentityEvidenceReport, range: BoundSourceRange) -> &[u8] {
    r.linkage()
        .source()
        .read(&range)
        .expect("identity report owns source token")
}
pub(super) fn collect(p: &mut GuestLoadPlan, b: &mut Budget) -> Result<(), PlanningError> {
    for (i, provider) in p.providers.iter().enumerate() {
        if !matches!(provider.evidence.outcome(), IdentityOutcome::Complete(_)) {
            b.push(&mut p.blockers, Blocker::ProviderEvidence { provider: i })?;
            continue;
        }
        let inspected = crate::elf::inspect::bounded::inspect(
            provider.evidence.linkage().source().clone(),
            crate::elf::inspect::bounded::InspectionLimits {
                max_program_headers: provider
                    .evidence
                    .linkage()
                    .limits()
                    .hash
                    .dynamic
                    .max_program_headers,
            },
        );
        let crate::elf::inspect::bounded::InspectionOutcome::Complete(headers) =
            inspected.outcome()
        else {
            b.push(&mut p.blockers, Blocker::ProviderEvidence { provider: i })?;
            continue;
        };
        if headers.header().machine != 62 {
            b.push(&mut p.blockers, Blocker::ProviderEvidence { provider: i })?;
            continue;
        }
        if headers
            .program_headers()
            .iter()
            .filter(|h| h.kind == 1)
            .any(|h| {
                h.file_size > h.memory_size
                    || h.virtual_address.checked_add(h.memory_size).is_none()
                    || provider
                        .evidence
                        .linkage()
                        .source()
                        .checked_range(h.file_offset, h.file_size)
                        .is_err()
            })
        {
            b.push(&mut p.blockers, Blocker::ProviderEvidence { provider: i })?;
            continue;
        }
        for (_, s) in provider.evidence.linkage().symbols().entries() {
            let f = s.fields();
            if f.index == 0
                || !matches!(f.section, Section::Index(_))
                || !matches!(f.binding, Binding::Global | Binding::Weak)
                || !matches!(
                    f.symbol_type,
                    SymbolType::Function | SymbolType::Object | SymbolType::NoType
                )
                || !matches!(f.visibility, Visibility::Default | Visibility::Protected)
            {
                continue;
            }
            let covered = headers
                .program_headers()
                .iter()
                .filter(|h| {
                    h.kind == 1
                        && h.file_size <= h.memory_size
                        && f.value >= h.virtual_address
                        && f.value
                            .checked_add(f.size.max(1))
                            .zip(h.virtual_address.checked_add(h.memory_size))
                            .is_some_and(|(end, limit)| end <= limit)
                        && (f.symbol_type != SymbolType::Function
                            || (h.flags & 1 != 0 && f.value - h.virtual_address < h.file_size))
                })
                .count();
            if covered != 1 {
                continue;
            }
            if let Some((nid, library, module, library_version, module_version)) =
                key(&provider.evidence, f.index, true)
            {
                b.push(
                    &mut p.candidates,
                    ProviderCandidate {
                        target: ProviderTarget::ArtifactSymbol {
                            provider: i,
                            symbol: f.index,
                            value: crate::metadata::VirtualAddress(f.value),
                        },
                        nid,
                        library,
                        module,
                        library_version,
                        module_version,
                    },
                )?;
            }
        }
    }
    let LinkageOutcome::Complete(e) = p.input.linkage().outcome() else {
        return b.push(&mut p.blockers, Blocker::EvidenceUnavailable);
    };
    for (i, d) in e.needed().iter().enumerate() {
        let name = bytes(&p.input, d.source);
        let mut supplied = Vec::new();
        for (j, provider) in p.providers.iter().enumerate() {
            if provider.dependency_alias.as_deref() == Some(name)
                && matches!(provider.evidence.outcome(), IdentityOutcome::Complete(_))
            {
                b.push(&mut supplied, j)?;
            }
        }
        if supplied.len() != 1 {
            b.push(
                &mut p.blockers,
                Blocker::Dependency {
                    index: i,
                    matches: supplied.len(),
                },
            )?;
        }
        for provider in &supplied {
            b.push(
                &mut p.blockers,
                Blocker::ProviderRequiresOwnLoadPlan {
                    provider: *provider,
                },
            )?;
        }
        b.push(
            &mut p.dependencies,
            DependencyPlan {
                name: d.source,
                supplied,
            },
        )?;
    }
    for (_, s) in p.input.linkage().symbols().entries() {
        let f = s.fields();
        if f.index == 0 || f.section != Section::Undefined {
            continue;
        }
        // Unreferenced undefined entries are preserved upstream, not mandatory link work.
        if !e.relocations().iter().any(|r| {
            u64::from(r.record.symbol_index()) == f.index && r.record.relocation_type() != 0
        }) {
            continue;
        }
        let k = key(&p.input, f.index, false);
        let mut matches = Vec::new();
        if matches!(f.binding, Binding::Global | Binding::Weak)
            && matches!(
                f.symbol_type,
                SymbolType::NoType | SymbolType::Function | SymbolType::Object
            )
            && matches!(f.visibility, Visibility::Default | Visibility::Protected)
            && let Some((nid, library, module, library_version, module_version)) = k
        {
            for (i, c) in p.candidates.iter().enumerate() {
                let ProviderTarget::ArtifactSymbol {
                    provider, symbol, ..
                } = c.target
                else {
                    continue;
                };
                let pr = &p.providers[provider].evidence;
                let pf = pr
                    .linkage()
                    .symbols()
                    .entries()
                    .find(|(_, s)| s.fields().index == symbol)
                    .map(|(_, s)| s.fields());
                if c.nid == nid
                    && library_version == c.library_version
                    && module_version == c.module_version
                    && bytes(&p.input, library) == bytes(pr, c.library)
                    && bytes(&p.input, module) == bytes(pr, c.module)
                    && pf.is_some_and(|pf| {
                        pf.symbol_type == f.symbol_type
                            || f.symbol_type == SymbolType::NoType
                            || pf.symbol_type == SymbolType::NoType
                    })
                {
                    b.push(&mut matches, i)?;
                }
            }
        }
        let resolution = if k.is_none() {
            PlannedResolution::ContextUnavailable
        } else {
            match matches.len() {
                0 => PlannedResolution::Unresolved,
                1 => PlannedResolution::Selected(matches[0]),
                _ => PlannedResolution::Ambiguous(matches),
            }
        };
        if let PlannedResolution::Selected(ci) = resolution
            && let ProviderTarget::ArtifactSymbol { provider, .. } = p.candidates[ci].target
        {
            b.push(
                &mut p.blockers,
                Blocker::ProviderRequiresOwnLoadPlan { provider },
            )?;
        }
        if !matches!(resolution, PlannedResolution::Selected(_)) {
            b.push(&mut p.blockers, Blocker::Reference { symbol: f.index })?;
        }
        b.push(
            &mut p.references,
            ReferencePlan {
                symbol: f.index,
                nid: k.map(|k| k.0),
                resolution,
            },
        )?;
    }
    Ok(())
}
