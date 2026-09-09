//! Undefined external linkage candidates, including unnamed observations; never resolved imports.
use super::evidence::CandidateEvidence;
#[derive(Clone, Copy, Debug)]
pub struct ImportCandidate<'r, 's> {
    pub(super) evidence: &'r CandidateEvidence<'s>,
}
impl<'r, 's> ImportCandidate<'r, 's> {
    pub fn evidence(&self) -> &'r CandidateEvidence<'s> {
        self.evidence
    }
}
