//! Immutable evidence returned only by trusted enumeration; names borrow source bytes.
use super::super::{
    classification::Classification, exports::ExportCandidate, imports::ImportCandidate,
};
use crate::{
    artifact::SourceId,
    elf::dynamic::{
        hash::extent::TrustedSymbolExtent, relocations::observation::RawRelocation,
        symbol_table::DynamicSymbolObservation,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameState {
    Absent,
    Empty,
    Utf8,
    Bytes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationUse {
    NoneObserved,
    Ordinary,
    Plt,
    Both,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateIdentity {
    pub source: SourceId,
    pub symbol_index: u64,
}
/// Private construction prevents detached symbol observations from granting membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateEvidence<'a> {
    pub(in crate::elf::dynamic::candidates) symbol: DynamicSymbolObservation<'a>,
    pub(in crate::elf::dynamic::candidates) extent: &'a TrustedSymbolExtent,
    pub(in crate::elf::dynamic::candidates) relocations: Vec<RawRelocation>,
}
impl<'a> CandidateEvidence<'a> {
    pub fn symbol(&self) -> &DynamicSymbolObservation<'a> {
        &self.symbol
    }
    pub fn extent(&self) -> &TrustedSymbolExtent {
        self.extent
    }
    pub fn relocations(&self) -> &[RawRelocation] {
        &self.relocations
    }
    pub fn identity(&self) -> CandidateIdentity {
        CandidateIdentity {
            source: self.symbol.source.source_id(),
            symbol_index: self.symbol.index,
        }
    }
    pub fn name_state(&self) -> NameState {
        match self.symbol.name {
            None => NameState::Absent,
            Some(n) if n.as_bytes().is_empty() => NameState::Empty,
            Some(n) if n.as_utf8().is_ok() => NameState::Utf8,
            Some(_) => NameState::Bytes,
        }
    }
    pub fn relocation_use(&self) -> RelocationUse {
        let ordinary = self.relocations.iter().any(|r| r.dynamic_index.is_some());
        let plt = self.relocations.iter().any(|r| r.plt_index.is_some());
        match (ordinary, plt) {
            (false, false) => RelocationUse::NoneObserved,
            (true, false) => RelocationUse::Ordinary,
            (false, true) => RelocationUse::Plt,
            (true, true) => RelocationUse::Both,
        }
    }
}
/// Every trusted symbol is retained, including null/internal/unclassified entries.
/// Detached observations cannot replace the trusted evidence or classification.
/// ```compile_fail
/// use astero_loader::elf::dynamic::candidates::{evidence::CandidateObservation, classification::Classification};
/// fn forge(observation: &mut CandidateObservation<'_>) {
///     observation.classification = Classification::ExportCandidate;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateObservation<'a> {
    pub(in crate::elf::dynamic::candidates) evidence: CandidateEvidence<'a>,
    pub(in crate::elf::dynamic::candidates) classification: Classification,
}
impl<'a> CandidateObservation<'a> {
    pub fn evidence(&self) -> &CandidateEvidence<'a> {
        &self.evidence
    }
    pub fn classification(&self) -> Classification {
        self.classification
    }
    pub fn import(&self) -> Option<ImportCandidate<'_, 'a>> {
        (self.classification == Classification::ImportCandidate).then_some(ImportCandidate {
            evidence: &self.evidence,
        })
    }
    pub fn export(&self) -> Option<ExportCandidate<'_, 'a>> {
        (self.classification == Classification::ExportCandidate).then_some(ExportCandidate {
            evidence: &self.evidence,
        })
    }
}
