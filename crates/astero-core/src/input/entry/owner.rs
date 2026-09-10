use super::*;
use crate::input::native::{NativeBackedGuestImage, NativeState};
use astero_kernel::execution::preparation::{
    boundary::{BoundaryState, RecoveryBoundary},
    layout,
    storage::{StorageError, ThreadStorage},
};
use astero_loader::load_plan::{
    bootstrap::{self, BootstrapEvidence},
    link::Action,
};
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{GuestRange, NativeError, NativeObserver},
};
use astero_timing::scheduler::TimingEngine;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryBlocker {
    EntryUnavailable,
    EntryNotExecutable,
    PendingRelocations { count: usize },
    ReturnLandingMissing,
    ExitHandlerMissing,
    NativeBridgeMissing,
    RecoveryAdapterMissing,
    TlsActivationMissing,
    TcbLayoutUnproven,
    InitializersUnprepared,
    RelroPending { index: usize },
    RelroSharedPage { index: usize },
    UnknownProgram { index: usize, kind: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EarlyRuntimeBlocker {
    Dependencies { count: usize },
    UnresolvedReferences { count: usize },
    ProvidersNotInstalled { count: usize },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelroState {
    Finalized { index: usize },
    PendingWrites { index: usize },
    UnalignedOrShared { index: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryState {
    PreparedBlocked,
    Released,
}
#[derive(Debug)]
pub enum EntryError {
    Released,
    Bootstrap(bootstrap::BootstrapError),
    Layout(PreparationError),
    Storage(StorageError),
    Native(NativeError),
    Source(astero_loader::artifact::SourceError),
    Allocation,
}
impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entry preparation: {self:?}")
    }
}
impl std::error::Error for EntryError {}
/// Owns all runtime allocations and immutable plan. No public native transfer method exists.
/// The M30 adapter must satisfy the reported blockers; a Boolean cannot grant entry authority.
pub struct PreparedGuest {
    pub(super) image: NativeBackedGuestImage,
    pub(super) thread: ThreadStorage,
    pub(super) context: InitialContext,
    pub(super) bootstrap: BootstrapEvidence,
    pub(super) registry: PreparedRegistry,
    pub(super) recovery: RecoveryBoundary,
    pub(super) timing: Option<TimingEngine>,
    pub(super) blockers: Vec<EntryBlocker>,
    pub(super) early: Vec<EarlyRuntimeBlocker>,
    pub(super) relro: Vec<RelroState>,
    pub(super) state: EntryState,
}
impl PreparedGuest {
    pub fn state(&self) -> EntryState {
        self.state
    }
    pub fn entry_ready(&self) -> bool {
        false
    }
    pub fn image(&self) -> &NativeBackedGuestImage {
        &self.image
    }
    pub fn context(&self) -> &InitialContext {
        &self.context
    }
    pub fn stack(&self) -> &ThreadLayout {
        self.thread.layout()
    }
    pub fn bootstrap(&self) -> &BootstrapEvidence {
        &self.bootstrap
    }
    pub fn blockers(&self) -> &[EntryBlocker] {
        &self.blockers
    }
    pub fn early_blockers(&self) -> &[EarlyRuntimeBlocker] {
        &self.early
    }
    pub fn relro(&self) -> &[RelroState] {
        &self.relro
    }
    pub fn registry(&self) -> &PreparedRegistry {
        &self.registry
    }
    pub fn recovery(&self) -> BoundaryState {
        self.recovery.state()
    }
    pub fn has_timing(&self) -> bool {
        self.timing.is_some()
    }
    pub fn read_stack(&self, address: u64, size: u64) -> Result<Vec<u8>, NativeError> {
        self.thread.read_stack(address, size)
    }
    pub fn read_tls(&self, address: u64, size: u64) -> Result<Vec<u8>, NativeError> {
        self.thread.read_tls(address, size)
    }
    pub fn release(&mut self) -> Result<(), NativeError> {
        self.recovery.release();
        // No execution leases exist in M29. Future entry must prohibit teardown while active.
        self.timing.take();
        let a = self.thread.release();
        let b = self.image.release();
        if a.is_ok() && b.is_ok() {
            self.state = EntryState::Released;
        }
        a.and(b)
    }
}
impl Drop for PreparedGuest {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
pub fn prepare(
    mut image: NativeBackedGuestImage,
    limits: RuntimeLimits,
    registry: PreparedRegistry,
    timing: Option<TimingEngine>,
) -> Result<(PreparedGuest, [NativeObserver; 2]), EntryError> {
    if image.state() == NativeState::Released {
        return Err(EntryError::Released);
    }
    let bootstrap = bootstrap::observe(image.plan()).map_err(EntryError::Bootstrap)?;
    let source = image.plan().headers().source();
    let (template, mem, align) = if let Some(t) = &bootstrap.tls {
        (
            source.read(&t.source).map_err(EntryError::Source)?,
            t.memory_size,
            t.alignment,
        )
    } else {
        (&[][..], 0, 0)
    };
    let layout = layout::plan(
        limits,
        image.snapshot().geometry,
        template.len() as u64,
        mem,
        align,
    )
    .map_err(EntryError::Layout)?;
    let (thread, observers) =
        ThreadStorage::build(layout, template).map_err(EntryError::Storage)?;
    let mut blockers = Vec::new();
    blockers
        .try_reserve(
            bootstrap
                .relro
                .len()
                .checked_add(image.plan().blockers().len())
                .and_then(|n| n.checked_add(16))
                .ok_or(EntryError::Allocation)?,
        )
        .map_err(|_| EntryError::Allocation)?;
    let entry = image.plan().entry().map_or(0, |v| v.0);
    if entry == 0 {
        blockers.push(EntryBlocker::EntryUnavailable)
    } else if !image.pages().iter().any(|p| {
        p.protection.execute && entry >= p.range.start.0 && entry - p.range.start.0 < p.range.size
    }) {
        blockers.push(EntryBlocker::EntryNotExecutable)
    }
    let count = image.staging_snapshot().pending_relocations;
    if count != 0 {
        blockers.push(EntryBlocker::PendingRelocations { count })
    }
    blockers.extend([
        EntryBlocker::ReturnLandingMissing,
        EntryBlocker::ExitHandlerMissing,
        EntryBlocker::NativeBridgeMissing,
        EntryBlocker::RecoveryAdapterMissing,
        EntryBlocker::TlsActivationMissing,
        EntryBlocker::TcbLayoutUnproven,
    ]);
    if !bootstrap.dynamic_init.is_empty() {
        blockers.push(EntryBlocker::InitializersUnprepared)
    }
    for b in image.plan().blockers() {
        if let astero_loader::load_plan::link::Blocker::ProgramSemantics { index, kind } = b
            && !matches!(*kind, 7 | 0x6474e552 | 0x61000001)
        {
            blockers.push(EntryBlocker::UnknownProgram {
                index: *index,
                kind: *kind,
            });
        }
    }
    let mut relro = Vec::new();
    relro
        .try_reserve(bootstrap.relro.len())
        .map_err(|_| EntryError::Allocation)?;
    for (index, r) in bootstrap.relro.iter().enumerate() {
        if r.size == 0 {
            continue;
        }
        let end = r.start.0 + r.size;
        let pending = image.plan().relocations().iter().any(|p| {
            p.action != Action::None
                && (p.value.is_none() || p.resolution.is_some())
                && (p.width == 0
                    || p.place.is_none_or(|a| {
                        a.0 < end && a.0.saturating_add(u64::from(p.width).max(1)) > r.start.0
                    }))
        });
        if pending {
            relro.push(RelroState::PendingWrites { index });
            blockers.push(EntryBlocker::RelroPending { index });
            continue;
        }
        let page = image.snapshot().geometry.page_size;
        if !r.start.0.is_multiple_of(page) || !r.size.is_multiple_of(page) {
            relro.push(RelroState::UnalignedOrShared { index });
            blockers.push(EntryBlocker::RelroSharedPage { index });
            continue;
        }
        // Do not remove write from bytes outside the declared interval. Exact-page only.
        image
            .seal_read_only(GuestRange {
                start: GuestAddress(r.start.0),
                size: r.size,
            })
            .map_err(EntryError::Native)?;
        relro.push(RelroState::Finalized { index });
    }
    let context = InitialContext::planned(
        entry,
        thread.layout().rsp,
        thread.layout().params,
        thread.layout().thread_pointer,
    );
    let early = vec![
        EarlyRuntimeBlocker::Dependencies {
            count: image.plan().dependencies().len(),
        },
        EarlyRuntimeBlocker::UnresolvedReferences {
            count: image.staging_snapshot().unresolved_references,
        },
        EarlyRuntimeBlocker::ProvidersNotInstalled {
            count: registry.len(),
        },
    ];
    Ok((
        PreparedGuest {
            image,
            thread,
            context,
            bootstrap,
            registry,
            recovery: RecoveryBoundary::default(),
            timing,
            blockers,
            early,
            relro,
            state: EntryState::PreparedBlocked,
        },
        observers,
    ))
}
