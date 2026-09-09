//! Visible defined candidates; no runtime export registration or address assignment.
use super::evidence::CandidateEvidence;
#[derive(Clone, Copy, Debug)]
pub struct ExportCandidate<'r, 's> {
    pub(super) evidence: &'r CandidateEvidence<'s>,
}
impl<'r, 's> ExportCandidate<'r, 's> {
    pub fn evidence(&self) -> &'r CandidateEvidence<'s> {
        self.evidence
    }
}
