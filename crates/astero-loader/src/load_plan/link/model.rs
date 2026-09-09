use crate::{
    artifact::{BoundSourceRange, SourceError},
    elf::{
        dynamic::{identity::Ps5IdentityEvidenceReport, relocations::observation::RawRelocation},
        inspect::bounded::InspectionReport,
        program_headers::ProgramHeader,
    },
    load_plan::MappingIntent,
    metadata::VirtualAddress,
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanningLimits {
    pub max_providers: u64,
    pub max_plan_records: u64,
}
/// Future adapter contract: documentary provider identity only. M26 does not accept these
/// as installed handlers; a later HLE adapter must validate registration and address authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HleProviderDeclaration {
    pub declaration_id: u64,
    pub nid: u64,
    pub library: Vec<u8>,
    pub module: Vec<u8>,
    pub evidence_label: String,
}
/// Caller declaration, never an HLE registration or a filesystem search result.
#[derive(Clone, Debug)]
pub struct ProviderInput {
    pub evidence: Arc<Ps5IdentityEvidenceReport>,
    pub dependency_alias: Option<Vec<u8>>,
    pub image_bias: Option<VirtualAddress>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderTarget {
    ArtifactSymbol {
        provider: usize,
        symbol: u64,
        value: VirtualAddress,
    },
    FutureHleDeclaration,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderCandidate {
    pub target: ProviderTarget,
    pub nid: u64,
    pub library: BoundSourceRange,
    pub module: BoundSourceRange,
    pub library_version: u16,
    pub module_version: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedResolution {
    Selected(usize),
    Unresolved,
    Ambiguous(Vec<usize>),
    ContextUnavailable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferencePlan {
    pub symbol: u64,
    pub nid: Option<u64>,
    pub resolution: PlannedResolution,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyPlan {
    pub name: BoundSourceRange,
    pub supplied: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentPlan {
    pub program_index: usize,
    pub raw: ProgramHeader,
    pub mapping: MappingIntent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Absolute64,
    PcRelative32,
    GlobDat64,
    JumpSlot64,
    Relative64,
    Unsupported(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocationPlan {
    pub raw: RawRelocation,
    pub place: Option<VirtualAddress>,
    pub action: Action,
    pub value: Option<u64>,
    pub width: u8,
    pub resolution: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    EvidenceUnavailable,
    ProviderEvidence {
        provider: usize,
    },
    ProviderRequiresOwnLoadPlan {
        provider: usize,
    },
    Machine(u16),
    ObjectType(u16),
    NoSegments,
    Segment {
        index: usize,
        reason: SegmentProblem,
    },
    SegmentOverlap {
        first: usize,
        second: usize,
    },
    EntryUnavailable,
    EntryOutsideExecutable,
    Bootstrap {
        tag: i64,
        value: u64,
    },
    ProgramSemantics {
        index: usize,
        kind: u32,
    },
    Dependency {
        index: usize,
        matches: usize,
    },
    Reference {
        symbol: u64,
    },
    Relocation {
        index: usize,
        reason: RelocationProblem,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentProblem {
    Size,
    Alignment,
    Overflow,
    Flags,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationProblem {
    Unsupported,
    TargetOutsideImage,
    SymbolUnavailable,
    Arithmetic,
    Overlap,
    NonzeroRelativeSymbol,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    ReadyForLoad,
    Experimental,
    Blocked,
}
#[derive(Debug)]
pub enum PlanningError {
    Budget {
        required: u64,
        maximum: u64,
    },
    ProviderBudget {
        count: u64,
        maximum: u64,
    },
    Allocation {
        records: u64,
    },
    Source {
        source_id: crate::artifact::SourceId,
        segment: usize,
        error: SourceError,
    },
    Arithmetic,
}
impl std::fmt::Display for PlanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "load planning: {self:?}")
    }
}
impl std::error::Error for PlanningError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source { error, .. } => Some(error),
            _ => None,
        }
    }
}
/// Private construction; getters never expose mutable plan/evidence storage.
/// ```compile_fail
/// use astero_loader::load_plan::link::GuestLoadPlan;
/// fn alter(p: &mut GuestLoadPlan) { p.segments.clear(); }
/// ```
#[derive(Debug)]
pub struct GuestLoadPlan {
    pub(super) input: Arc<Ps5IdentityEvidenceReport>,
    pub(super) providers: Vec<ProviderInput>,
    pub(super) headers: InspectionReport,
    pub(super) limits: PlanningLimits,
    pub(super) bias: VirtualAddress,
    pub(super) entry: Option<VirtualAddress>,
    pub(super) segments: Vec<SegmentPlan>,
    pub(super) dependencies: Vec<DependencyPlan>,
    pub(super) candidates: Vec<ProviderCandidate>,
    pub(super) references: Vec<ReferencePlan>,
    pub(super) relocations: Vec<RelocationPlan>,
    pub(super) blockers: Vec<Blocker>,
    pub(super) experimental: bool,
}
impl GuestLoadPlan {
    pub fn input(&self) -> &Arc<Ps5IdentityEvidenceReport> {
        &self.input
    }
    pub fn providers(&self) -> &[ProviderInput] {
        &self.providers
    }
    pub fn headers(&self) -> &InspectionReport {
        &self.headers
    }
    pub fn limits(&self) -> PlanningLimits {
        self.limits
    }
    pub fn image_bias(&self) -> VirtualAddress {
        self.bias
    }
    pub fn entry(&self) -> Option<VirtualAddress> {
        self.entry
    }
    pub fn segments(&self) -> &[SegmentPlan] {
        &self.segments
    }
    pub fn dependencies(&self) -> &[DependencyPlan] {
        &self.dependencies
    }
    pub fn candidates(&self) -> &[ProviderCandidate] {
        &self.candidates
    }
    pub fn references(&self) -> &[ReferencePlan] {
        &self.references
    }
    pub fn relocations(&self) -> &[RelocationPlan] {
        &self.relocations
    }
    pub fn blockers(&self) -> &[Blocker] {
        &self.blockers
    }
    pub fn readiness(&self) -> Readiness {
        if !self.blockers.is_empty() {
            Readiness::Blocked
        } else if self.experimental {
            Readiness::Experimental
        } else {
            Readiness::ReadyForLoad
        }
    }
}
pub(super) struct Budget {
    used: u64,
    maximum: u64,
}
impl Budget {
    pub fn new(maximum: u64) -> Self {
        Self { used: 0, maximum }
    }
    pub fn push<T>(&mut self, v: &mut Vec<T>, x: T) -> Result<(), PlanningError> {
        self.used = self.used.checked_add(1).ok_or(PlanningError::Arithmetic)?;
        if self.used > self.maximum {
            return Err(PlanningError::Budget {
                required: self.used,
                maximum: self.maximum,
            });
        }
        v.try_reserve(1)
            .map_err(|_| PlanningError::Allocation { records: self.used })?;
        v.push(x);
        Ok(())
    }
}

/// Two focused M24/M26 documentary correlations, not provider registrations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogueCorrelation {
    pub name: &'static str,
    pub evidence: &'static str,
    pub astero_registered: bool,
}
pub fn catalogue_correlation(nid: u64) -> Option<CatalogueCorrelation> {
    match nid {
        0x1f67bcb7949c4067 => Some(CatalogueCorrelation {
            name: "__cxa_finalize",
            evidence: "07_NIDS/by_sysmodule/libSceLibcInternal.sprx/012.md; approved firmware numeric join",
            astero_registered: false,
        }),
        0x3f7df43f774517af => Some(CatalogueCorrelation {
            name: "LIBC_NEED_FLAG_NID (legacy constant; function name unestablished)",
            evidence: "07_NIDS/by_sysmodule/SOURCE_OWNER_kernel/003.md",
            astero_registered: false,
        }),
        _ => None,
    }
}
