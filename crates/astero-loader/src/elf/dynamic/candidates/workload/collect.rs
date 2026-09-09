use super::*;
use crate::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded,
        candidates::structural::{
            self, CandidateRole, ClassificationLimits, ClassificationOutcome,
        },
        hash::bounded as hash,
        observation,
        relocations::RelocationTables,
        symbol_table::{
            Binding, SymbolType, Visibility,
            bounded::{self as symbols, SymbolOutcome},
        },
    },
};
use std::sync::Arc;
pub fn observe(source: SourceArtifact, limits: LinkageLimits) -> LinkageReport {
    let proof = Arc::new(hash::observe(source.clone(), limits.hash));
    let input = Arc::new(symbols::observe(source.clone(), proof, limits.symbols));
    let symbols = Arc::new(structural::classify(
        input,
        ClassificationLimits {
            max_classifications: limits.symbols.max_symbols,
        },
    ));
    let outcome = match symbols.outcome() {
        ClassificationOutcome::Unavailable => LinkageOutcome::Unavailable,
        ClassificationOutcome::Failed(_) => LinkageOutcome::Failed(LinkageFailure::Symbols),
        ClassificationOutcome::Complete(_) => match collect(&source, &symbols, limits) {
            Ok(e) => LinkageOutcome::Complete(e),
            Err(e) => LinkageOutcome::Failed(e),
        },
    };
    LinkageReport {
        source,
        limits,
        symbols,
        outcome,
    }
}
fn collect(
    source: &SourceArtifact,
    classified: &structural::SymbolClassificationReport,
    limits: LinkageLimits,
) -> Result<LinkageEvidence, LinkageFailure> {
    use LinkageFailure as F;
    let SymbolOutcome::Complete(symbols) = classified.input().outcome() else {
        return Err(F::Symbols);
    };
    let Some((headers, raw)) =
        bounded::collect::discover(source, limits.hash.dynamic).map_err(F::Discovery)?
    else {
        return Err(F::Symbols);
    };
    for e in raw.entries() {
        if let crate::elf::dynamic::tags::DynamicTag::Unknown(tag @ (17 | 18 | 19 | 35 | 36 | 37)) =
            e.tag
        {
            return Err(F::UnsupportedRelocationDescriptor {
                dynamic_index: e.index,
                tag,
            });
        }
    }
    let table =
        observation::from_raw(source, headers.program_headers(), raw).map_err(F::Dynamic)?;
    let tables = RelocationTables::from_dynamic(&table).map_err(F::Relocation)?;
    let stream = tables
        .enumerate_raw(limits.max_relocations)
        .map_err(F::Relocation)?;
    let mut uses = Vec::new();
    uses.try_reserve_exact(symbols.len())
        .map_err(|_| F::Allocation {
            records: symbols.len() as u64,
        })?;
    for s in symbols {
        uses.push(SymbolUse {
            symbol_index: s.fields().index,
            references: 0,
            ordinary: 0,
            plt: 0,
            role: ReferenceRole::StructuralOnly,
        });
    }
    let mut relocations = Vec::new();
    for result in stream {
        let raw = result.map_err(F::Relocation)?;
        let index = raw.record.symbol_index() as usize;
        let Some(symbol) = symbols.get(index) else {
            return Err(F::SymbolIndex {
                raw: Box::new(raw),
                count: symbols.len() as u64,
            });
        };
        // Complete source-bound M21 evidence proves membership and symbol observation once, without rescanning names.
        if symbol.fields().index != index as u64
            || symbol.fields().source.source_id() != source.identity()
        {
            return Err(F::SymbolIndex {
                raw: Box::new(raw),
                count: symbols.len() as u64,
            });
        }
        let u = &mut uses[index];
        u.references += 1;
        u.ordinary += u64::from(raw.dynamic_index.is_some());
        u.plt += u64::from(raw.plt_index.is_some());
        relocations.try_reserve(1).map_err(|_| F::Allocation {
            records: relocations.len() as u64 + 1,
        })?;
        relocations.push(raw);
    }
    for ((role, s), u) in classified.entries().zip(&mut uses) {
        if role.role() == CandidateRole::UndefinedCandidate && u.references > 0 {
            let f = s.fields();
            let eligible = matches!(f.binding, Binding::Global | Binding::Weak)
                && f.visibility == Visibility::Default
                && f.other & !7 == 0
                && matches!(
                    f.symbol_type,
                    SymbolType::NoType
                        | SymbolType::Object
                        | SymbolType::Function
                        | SymbolType::Tls
                )
                && s.name_bytes().is_some_and(|n| !n.is_empty());
            u.role = if eligible {
                ReferenceRole::ExternalReferenceCandidate
            } else {
                ReferenceRole::AmbiguousReference
            };
        }
    }
    let needed = super::names::collect(&table, symbols, limits.symbols)?;
    let counts = LinkageCounts {
        symbols: symbols.len() as u64,
        relocations: relocations.len() as u64,
        symbol_associated: relocations
            .iter()
            .filter(|r| r.record.symbol_index() != 0)
            .count() as u64,
        null_references: relocations
            .iter()
            .filter(|r| r.record.symbol_index() == 0)
            .count() as u64,
        external_candidates: uses
            .iter()
            .filter(|u| u.role == ReferenceRole::ExternalReferenceCandidate)
            .count() as u64,
        definitions: classified
            .entries()
            .filter(|(r, _)| r.role() == CandidateRole::DefinitionCandidate)
            .count() as u64,
        ambiguous_references: uses
            .iter()
            .filter(|u| u.role == ReferenceRole::AmbiguousReference)
            .count() as u64,
    };
    Ok(LinkageEvidence {
        counts,
        relocations,
        uses,
        needed,
    })
}
