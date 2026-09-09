use super::LinkageEvidenceReport;
use astero_loader::{artifact::SourceId, modules::ModuleId};
use std::sync::Arc;
/// Declared observation identity only; neither runtime module nor authenticated PS5 target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceTarget {
    pub source: SourceId,
    pub module: Option<ModuleId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    TargetRequired,
    IdentityMismatch {
        target: EvidenceTarget,
        report: EvidenceTarget,
    },
}
/// Checked once at construction; no replacement or mutable attachment API.
/// ```compile_fail
/// use astero_core::session::inputs::SessionInputs;
/// fn change(inputs: &mut SessionInputs) { inputs.linkage = None; }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionInputs {
    target: Option<EvidenceTarget>,
    linkage: Option<Arc<LinkageEvidenceReport>>,
}
impl SessionInputs {
    pub fn new(
        target: Option<EvidenceTarget>,
        linkage: Option<Arc<LinkageEvidenceReport>>,
    ) -> Result<Self, InputError> {
        if let Some(report) = &linkage {
            let expected = EvidenceTarget {
                source: report.source(),
                module: report.module(),
            };
            let target = target.ok_or(InputError::TargetRequired)?;
            if target != expected {
                return Err(InputError::IdentityMismatch {
                    target,
                    report: expected,
                });
            }
        }
        Ok(Self { target, linkage })
    }
    pub fn target(&self) -> Option<EvidenceTarget> {
        self.target
    }
    pub fn linkage(&self) -> Option<&Arc<LinkageEvidenceReport>> {
        self.linkage.as_ref()
    }
}
