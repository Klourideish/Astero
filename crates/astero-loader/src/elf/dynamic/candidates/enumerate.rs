use super::{
    classification::classify,
    error::CandidateError,
    evidence::{CandidateEvidence, CandidateObservation},
};
use crate::elf::dynamic::{
    hash::extent::TrustedSymbolExtent,
    relocations::{RelocationLimits, RelocationTables, observation::RawRelocation},
    symbol_table::{EnumerationLimits, SymbolEnumeration, SymbolError, SymbolTable},
};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateLimits {
    pub symbols: EnumerationLimits,
    pub relocations: RelocationLimits,
}
/// Symbol-index ordered, lazy and fail-stop. A caller stopping early has only a prefix.
pub struct CandidateEnumeration<'s, 'e> {
    symbols: SymbolEnumeration<'s, 'e>,
    extent: &'s TrustedSymbolExtent,
    associations: BTreeMap<u64, Vec<RawRelocation>>,
}
/// Preflights trusted symbol count and completely validates the budgeted relocation inventory.
/// No API accepts caller-constructed symbol/relocation observations as trusted candidates.
pub fn enumerate<'s, 'e>(
    symbols: &'s SymbolTable<'e>,
    relocations: &RelocationTables,
    limits: CandidateLimits,
) -> Result<CandidateEnumeration<'s, 'e>, CandidateError> {
    let extent = symbols
        .extent()
        .ok_or(CandidateError::Symbol(SymbolError::CountUnavailable))?;
    if extent.source_range().source_id() != relocations.source().identity() {
        return Err(CandidateError::SourceMismatch {
            symbols: extent.source_range().source_id(),
            relocations: relocations.source().identity(),
        });
    }
    let enumeration = symbols
        .enumerate(limits.symbols)
        .map_err(CandidateError::Symbol)?;
    let mut associations: BTreeMap<u64, Vec<RawRelocation>> = BTreeMap::new();
    for relocation in relocations
        .enumerate(symbols, limits.relocations)
        .map_err(CandidateError::Relocation)?
    {
        let raw = relocation.map_err(CandidateError::Relocation)?.raw;
        associations
            .entry(u64::from(raw.record.symbol_index()))
            .or_default()
            .push(raw);
    }
    Ok(CandidateEnumeration {
        symbols: enumeration,
        extent,
        associations,
    })
}
impl<'s> Iterator for CandidateEnumeration<'s, '_> {
    type Item = Result<CandidateObservation<'s>, CandidateError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.symbols.next().map(|result| {
            let symbol = result.map_err(CandidateError::Symbol)?;
            let relocations = self.associations.remove(&symbol.index).unwrap_or_default();
            let evidence = CandidateEvidence {
                symbol,
                extent: self.extent,
                relocations,
            };
            let classification = classify(&evidence);
            Ok(CandidateObservation {
                evidence,
                classification,
            })
        })
    }
}
impl std::iter::FusedIterator for CandidateEnumeration<'_, '_> {}
