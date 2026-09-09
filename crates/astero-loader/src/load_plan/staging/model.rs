use crate::{
    artifact::SourceError,
    load_plan::{
        MappingIntent,
        link::{Blocker, GuestLoadPlan},
    },
    metadata::VirtualAddress,
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StagingLimits {
    pub max_mapped_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectionEnforcement {
    MetadataOnly,
    HostEnforced,
}
/// Trusted host adapter contract: fresh isolated storage, zeroed mappings, checked writes,
/// no guest callbacks; release all mappings on drop. A future native backend must reserve the
/// exact planned guest addresses or refuse, not silently change bias. No CPU fetch API exists.
pub trait StagingBackend: Sized {
    type Error: std::error::Error;
    /// Reserve the complete guest placement before any segment writes. A native backend can
    /// reserve a page/granularity-aligned envelope here and handle shared boundary pages.
    fn reserve_image(
        &mut self,
        mappings: &[crate::load_plan::link::SegmentPlan],
    ) -> Result<(), Self::Error>;
    fn map(&mut self, intent: &MappingIntent) -> Result<(), Self::Error>;
    fn write(&mut self, address: VirtualAddress, bytes: &[u8]) -> Result<(), Self::Error>;
    fn read(&self, address: VirtualAddress, size: u64) -> Result<&[u8], Self::Error>;
    fn finalize(
        &mut self,
        mappings: &[crate::load_plan::link::SegmentPlan],
    ) -> Result<ProtectionEnforcement, Self::Error>;
    fn clear(&mut self);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockerClass {
    StageBlocking,
    ResolutionBlocking,
    ExecutionBlocking,
    DeferredDiagnostic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationState {
    Applied,
    Pending,
    NoWrite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageStatus {
    Staged,
    StagedWithPendingWork,
    Released,
}
#[derive(Debug)]
pub enum StagingError<E> {
    Blocked {
        index: usize,
        blocker: Blocker,
    },
    Budget {
        required: u64,
        maximum: u64,
    },
    Arithmetic,
    Source {
        segment: usize,
        error: SourceError,
    },
    InvalidMapping {
        segment: usize,
    },
    InvalidRelocation {
        index: usize,
    },
    Allocation,
    Backend {
        operation: &'static str,
        index: usize,
        error: E,
    },
}
impl<E: std::fmt::Debug> std::fmt::Display for StagingError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "staging: {self:?}")
    }
}
impl<E: std::error::Error + 'static> std::error::Error for StagingError<E> {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StagingSnapshot {
    pub status: StageStatus,
    pub mapped_segments: usize,
    pub mapped_bytes: u64,
    pub copied_bytes: u64,
    pub zero_filled_bytes: u64,
    pub applied_relocations: usize,
    pub pending_relocations: usize,
    pub unresolved_references: usize,
    pub protections: ProtectionEnforcement,
    /// Always false in M27, including a plan with no blockers.
    pub ready_for_execution: bool,
}
/// Private storage; only immutable diagnostics/reads and idempotent release are exposed.
pub struct StagedGuestImage<B: StagingBackend> {
    pub(super) plan: Arc<GuestLoadPlan>,
    pub(super) backend: B,
    pub(super) relocations: Vec<RelocationState>,
    pub(super) snapshot: StagingSnapshot,
}
impl<B: StagingBackend> StagedGuestImage<B> {
    pub fn plan(&self) -> &Arc<GuestLoadPlan> {
        &self.plan
    }
    pub fn snapshot(&self) -> StagingSnapshot {
        self.snapshot
    }
    pub fn relocations(&self) -> &[RelocationState] {
        &self.relocations
    }
    pub fn read(&self, address: VirtualAddress, size: u64) -> Result<&[u8], B::Error> {
        self.backend.read(address, size)
    }
    pub fn release(&mut self) {
        self.backend.clear();
        self.snapshot.status = StageStatus::Released;
        self.snapshot.mapped_segments = 0;
        self.snapshot.mapped_bytes = 0;
    }
}
impl<B: StagingBackend> Drop for StagedGuestImage<B> {
    fn drop(&mut self) {
        self.backend.clear();
    }
}
