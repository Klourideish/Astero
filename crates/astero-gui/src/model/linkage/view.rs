use astero_core::{
    observation::SessionSnapshot,
    session::inputs::{
        CandidateDetail, Completeness, EvidenceOrigin, LinkageEvidenceReport, ReportCounts,
    },
};
/// Borrows a single already-observed session; does not perform another observation.
pub struct LinkageView<'a> {
    session: &'a SessionSnapshot,
}
impl<'a> LinkageView<'a> {
    pub fn new(session: &'a SessionSnapshot) -> Self {
        Self { session }
    }
    pub fn report(&self) -> Option<&'a LinkageEvidenceReport> {
        self.session.inputs.linkage().map(AsRef::as_ref)
    }
    pub fn provenance(&self) -> &'static str {
        match self.session.inputs.origin() {
            EvidenceOrigin::Synthetic => "Synthetic linkage evidence",
            EvidenceOrigin::Unspecified => "Evidence origin unspecified (not authenticated)",
        }
    }
    pub fn loaded_state(&self) -> &'static str {
        if self.session.loaded_target.is_none() {
            "No guest loaded"
        } else {
            "Loaded-target metadata present"
        }
    }
    pub fn status(&self) -> String {
        match self.report().map(|r| r.completeness()) {
            None => "Linkage evidence unavailable: no report supplied".into(),
            Some(Completeness::Complete) => "Enumeration: Complete".into(),
            Some(Completeness::Partial {
                observed,
                remaining,
                reason,
            }) => format!(
                "Enumeration: Partial - report budget reached; observed {observed}, remaining symbols {remaining:?}; {reason:?}"
            ),
            Some(Completeness::Unavailable(reason)) => {
                format!("Linkage evidence unavailable: {reason:?}")
            }
            Some(Completeness::Failed(error)) => format!("Evidence collection failed: {error:?}"),
        }
    }
    pub fn counts(&self) -> Option<&'a ReportCounts> {
        self.report().map(|r| r.counts())
    }
    pub fn details(&self) -> &'a [CandidateDetail] {
        self.report().map_or(&[], |r| r.details())
    }
    pub fn selected(&self, index: u64) -> Option<&'a CandidateDetail> {
        self.details().iter().find(|row| row.index == index)
    }
}
