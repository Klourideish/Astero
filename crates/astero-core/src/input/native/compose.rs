use super::*;
use crate::input::staging::{StageStatus, StagedGuestImage, StagingSnapshot};
use astero_loader::load_plan::link::GuestLoadPlan;
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{self, GuestRange, NativeImage, NativeRegion, Protection},
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeState {
    NativeBacked,
    NativeBackedWithPendingWork,
    Released,
}
pub struct NativeBackedGuestImage {
    image: Arc<NativeImage>,
    plan: Arc<GuestLoadPlan>,
    staging: StagingSnapshot,
}
impl NativeBackedGuestImage {
    pub(crate) fn shared_owner(&self) -> Arc<NativeImage> {
        self.image.clone()
    }
    pub(crate) fn native_owner(&self) -> &NativeImage {
        &self.image
    }
    pub fn state(&self) -> NativeState {
        if !self.image.snapshot().active {
            NativeState::Released
        } else if self.staging.pending_relocations > 0
            || self.staging.unresolved_references > 0
            || !self.plan.blockers().is_empty()
        {
            NativeState::NativeBackedWithPendingWork
        } else {
            NativeState::NativeBacked
        }
    }
    pub fn snapshot(&self) -> NativeSnapshot {
        self.image.snapshot()
    }
    pub fn staging_snapshot(&self) -> StagingSnapshot {
        self.staging
    }
    pub fn pages(&self) -> &[NativePage] {
        self.image.pages()
    }
    pub fn plan(&self) -> &Arc<GuestLoadPlan> {
        &self.plan
    }
    pub(crate) fn write(&mut self, address: u64, bytes: &[u8]) -> Result<(), NativeError> {
        self.image.write(GuestAddress(address), bytes)
    }
    pub(crate) fn seal_read_only(&mut self, range: GuestRange) -> Result<(), NativeError> {
        Arc::get_mut(&mut self.image)
            .ok_or(NativeError::SharedOwnership)?
            .seal_read_only(range)
    }
    pub fn release(&mut self) -> Result<(), NativeError> {
        Arc::get_mut(&mut self.image)
            .ok_or(NativeError::SharedOwnership)?
            .release()
    }
    pub fn read(&self, address: u64, size: u64) -> Result<Vec<u8>, NativeError> {
        self.image.read(GuestAddress(address), size)
    }
}
pub fn realize(
    staged: &StagedGuestImage,
    limits: NativeLimits,
) -> (Result<NativeBackedGuestImage, NativeError>, NativeObserver) {
    if staged.snapshot().status == StageStatus::Released {
        return (Err(NativeError::Released), NativeObserver::default());
    }
    let mut regions = Vec::new();
    if regions
        .try_reserve_exact(staged.plan().segments().len())
        .is_err()
    {
        return (Err(NativeError::Allocation), NativeObserver::default());
    }
    for segment in staged.plan().segments() {
        let m = &segment.mapping;
        let bytes = match staged.read(m.range.start, m.range.size) {
            Ok(b) => b,
            Err(_) => {
                return (
                    Err(NativeError::SourceSize {
                        index: segment.program_index,
                    }),
                    NativeObserver::default(),
                );
            }
        };
        regions.push(NativeRegion {
            range: GuestRange {
                start: GuestAddress(m.range.start.0),
                size: m.range.size,
            },
            bytes,
            protection: Protection {
                read: m.permissions.read,
                write: m.permissions.write,
                execute: m.permissions.execute,
            },
        });
    }
    let (result, observer) = windows_native::realize(&regions, limits);
    (
        result.map(|image| NativeBackedGuestImage {
            image: Arc::new(image),
            plan: staged.plan().clone(),
            staging: staged.snapshot(),
        }),
        observer,
    )
}
