//! Explicit isolated image composition; creates no Session or timing worker.
use astero_loader::load_plan::staging::StagingBackend;
pub use astero_loader::load_plan::staging::{
    BlockerClass, ProtectionEnforcement, RelocationState, StageStatus, StagingError, StagingLimits,
    StagingSnapshot, classify_blocker,
};
use astero_loader::{
    load_plan::{
        MappingIntent,
        link::{GuestLoadPlan, SegmentPlan},
    },
    metadata::VirtualAddress,
};
pub use astero_memory::mapping::MappingObserver;
use astero_memory::mapping::{GuestAddress, MemoryError, OwnedAddressSpace};
use std::sync::Arc;
pub struct ByteBackend(OwnedAddressSpace);
impl StagingBackend for ByteBackend {
    type Error = MemoryError;
    fn reserve_image(&mut self, _m: &[SegmentPlan]) -> Result<(), MemoryError> {
        Ok(())
    }
    fn map(&mut self, m: &MappingIntent) -> Result<(), MemoryError> {
        self.0
            .map_zeroed(GuestAddress(m.range.start.0), m.range.size)
    }
    fn write(&mut self, a: VirtualAddress, b: &[u8]) -> Result<(), MemoryError> {
        self.0.write(GuestAddress(a.0), b)
    }
    fn read(&self, a: VirtualAddress, n: u64) -> Result<&[u8], MemoryError> {
        self.0.read(GuestAddress(a.0), n)
    }
    fn finalize(&mut self, _m: &[SegmentPlan]) -> Result<ProtectionEnforcement, MemoryError> {
        self.0.finalize();
        Ok(ProtectionEnforcement::MetadataOnly)
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}
pub type StagedGuestImage = astero_loader::load_plan::staging::StagedGuestImage<ByteBackend>;
/// Observer returned even on refusal so callers can verify transaction cleanup.
pub fn stage(
    plan: Arc<GuestLoadPlan>,
    limits: StagingLimits,
) -> (
    Result<StagedGuestImage, StagingError<MemoryError>>,
    MappingObserver,
) {
    let backend = OwnedAddressSpace::new(limits.max_mapped_bytes);
    let observer = backend.observer();
    (
        astero_loader::load_plan::staging::stage(plan, ByteBackend(backend), limits),
        observer,
    )
}
