//! Explicit M30 closure: never transfers to the prepared image. Function landings are
//! distinct from unresolved data. The immutable M26 plan and M29 evidence stay intact.
use super::*;
use astero_kernel::execution::host::{Bridge, BridgeError};
use astero_kernel::execution::preparation::boundary::import_stub;
use astero_loader::{elf::dynamic::symbol_table::SymbolType, load_plan::link::Action};
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{self, GuestRange, NativeImage, NativeLimits, NativeRegion, Protection},
};
use std::sync::{Arc, Mutex};
/// Explicit hypothesis for workloads whose entry stub owns DT_INIT. Not firmware TCB proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupPolicy {
    PreserveUnknowns,
    ExperimentalEntryOwnedInit,
}
#[derive(Debug)]
pub enum ClosureError {
    Bridge(BridgeError),
    Entry(EntryError),
    Native {
        operation: &'static str,
        error: astero_memory::mapping::windows_native::NativeError,
    },
    Write {
        address: u64,
        error: astero_memory::mapping::windows_native::NativeError,
    },
    Budget,
    Registry(RegistryError),
    ExistingProviders {
        count: usize,
    },
    Allocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MustClose {
    PendingWrite {
        relocation: usize,
        kind: u32,
        symbol: u32,
    },
    Entry(EntryBlocker),
    RelroPending,
}
#[derive(Debug)]
pub struct ClosureReport {
    pub source: astero_loader::artifact::SourceId,
    pub pending_before: usize,
    pub treated: usize,
    pub pending_after: usize,
    pub object_trap_writes: usize,
    pub guarded_objects: usize,
    pub object_trap_bytes: u64,
    pub landings: usize,
    pub providers: usize,
    pub startup_matches: usize,
    pub bridge_validated: bool,
    pub remaining: Vec<MustClose>,
    pub relro_finalized: usize,
    pub relro_pending: usize,
    pub relro_hole_bytes: u64,
    pub landing_bytes: u64,
    pub policy: StartupPolicy,
    pub unresolved_function_landings: usize,
    pub early_runtime: Vec<&'static str>,
}
/// Owns installed RX landings and validated bridge; no real-artifact transfer is exposed.
pub struct ClosureGuest {
    guest: PreparedGuest,
    bridge: Bridge,
    landings: NativeImage,
    traps: super::traps::ObjectTraps,
    keys: Vec<Option<ProviderKey>>,
    startup: Arc<Mutex<astero_libs::libc::startup::StartupState>>,
    report: ClosureReport,
    foundation: Option<Arc<super::foundation::Foundation>>,
    synchronization: Option<std::sync::Arc<astero_kernel::synchronization::owned::Synchronization>>,
    sync_timing: Option<astero_timing::scheduler::TimingEngine>,
    runtime: Option<Arc<super::workers::Runtime>>,
}
impl Drop for ClosureGuest {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.stop_and_join();
        }
    }
}
/// Future execution must consume this authority, not a loose readiness Boolean.
/// No public constructor; unresolved closure never manufactures the capability.
pub struct EntryReadyGuest {
    owner: ClosureGuest,
    call_budget: usize,
}
impl EntryReadyGuest {
    /// Explicit per-run admission/retention bound; no change to wall-clock supervision.
    pub fn with_call_budget(mut self, maximum: usize) -> Result<Self, BridgeError> {
        if !(1..=65_536).contains(&maximum) {
            return Err(BridgeError::Validation);
        }
        self.call_budget = maximum;
        Ok(self)
    }
    pub fn report(&self) -> &ClosureReport {
        &self.owner.report
    }
}
impl ClosureGuest {
    pub fn startup_data(&self) -> Option<&astero_hle::providers::data::DataExport> {
        self.foundation.as_ref().map(|f| &f.guard)
    }
    pub fn startup_observer(
        &self,
    ) -> Option<astero_memory::mapping::windows_native::NativeObserver> {
        self.foundation.as_ref().map(|f| f.image.observer())
    }
    pub fn read_startup_data(
        &self,
    ) -> Result<Vec<u8>, astero_memory::mapping::windows_native::NativeError> {
        let f = self
            .foundation
            .as_ref()
            .ok_or(astero_memory::mapping::windows_native::NativeError::Unreadable)?;
        f.image.read(GuestAddress(f.guard.address), f.guard.size)
    }

    pub fn import_key(&self, ordinal: u32) -> Option<&ProviderKey> {
        self.keys.get(ordinal as usize).and_then(|k| k.as_ref())
    }
    pub fn trapped_object(&self, address: u64) -> Option<&super::traps::ObjectTrap> {
        self.traps
            .records
            .iter()
            .find(|t| address >= t.range.start.0 && address - t.range.start.0 < t.range.size)
    }
    /// Exercises the same exact-key dispatch through Astero-owned assembly only.
    pub fn validate_native_dispatch(
        &mut self,
        ordinal: u32,
    ) -> Result<astero_kernel::execution::host::NativeExit, BridgeError> {
        let keys = &self.keys;
        let registry = &self.guest.registry;
        self.bridge.synthetic(
            astero_kernel::execution::host::SyntheticProbe::ImportOrdinal(ordinal),
            &mut |i, frame| {
                keys.get(i as usize)
                    .and_then(|k| k.as_ref())
                    .is_some_and(|k| {
                        matches!(
                            registry.invoke_host_model(k, frame),
                            Ok(astero_hle::dispatch::prepared::CallResult::Returned)
                        )
                    })
            },
        )
    }
    pub fn object_traps(&self) -> &[super::traps::ObjectTrap] {
        &self.traps.records
    }
    pub fn traps_active(&self) -> bool {
        self.traps
            .image
            .as_ref()
            .is_some_and(|i| i.snapshot().active)
    }
    pub fn report(&self) -> &ClosureReport {
        &self.report
    }
    pub fn prepared(&self) -> &PreparedGuest {
        &self.guest
    }
    pub fn entry_ready(&self) -> bool {
        self.report.remaining.is_empty()
    }
    pub fn try_ready(self) -> Result<EntryReadyGuest, Box<Self>> {
        if self.entry_ready() {
            Ok(EntryReadyGuest {
                owner: self,
                call_budget: super::foundation::MAX_PROVIDER_CALLS,
            })
        } else {
            Err(Box::new(self))
        }
    }
    pub fn retained_callback_count(&self) -> usize {
        self.startup
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .callbacks()
            .count()
    }
    pub fn native_landing_count(&self) -> usize {
        self.keys.len()
    }
    pub fn bridge_validated(&self) -> bool {
        self.bridge.validated()
    }
    pub fn landing_mapping_active(&self) -> bool {
        self.landings.snapshot().active
    }
}
/// Runtime byte cap includes the landing pages, placed deterministically after TLS.
/// Fixed callback capacity 256 is the named M30 startup policy, not a full libc exit ABI.
pub fn close_entry(
    guest: PreparedGuest,
    max_runtime_bytes: u64,
    policy: StartupPolicy,
) -> Result<ClosureGuest, ClosureError> {
    close_impl(guest, max_runtime_bytes, policy, false)
}
/// M32 migrated startup capability; retains the preparation-only stop boundary.
pub fn close_startup(
    guest: PreparedGuest,
    max_runtime_bytes: u64,
    policy: StartupPolicy,
) -> Result<ClosureGuest, ClosureError> {
    close_impl(guest, max_runtime_bytes, policy, true)
}
fn close_impl(
    mut guest: PreparedGuest,
    max_runtime_bytes: u64,
    policy: StartupPolicy,
    migrate: bool,
) -> Result<ClosureGuest, ClosureError> {
    if !guest.registry.is_empty() {
        return Err(ClosureError::ExistingProviders {
            count: guest.registry.len(),
        });
    }
    let mut bridge = Bridge::new().map_err(ClosureError::Bridge)?;
    bridge.validate().map_err(ClosureError::Bridge)?;
    let plan = guest.image.plan().clone();
    let source = plan.headers().source();
    let mut keys = Vec::new();
    let mut patches = Vec::new();
    let mut remaining = Vec::new();
    let n = plan.relocations().len();
    keys.try_reserve(n).map_err(|_| ClosureError::Allocation)?;
    patches
        .try_reserve(n)
        .map_err(|_| ClosureError::Allocation)?;
    remaining
        .try_reserve(n + 16)
        .map_err(|_| ClosureError::Allocation)?;
    let page = guest.image.snapshot().geometry.page_size;
    let gran = guest.image.snapshot().geometry.allocation_granularity;
    let tls = &guest.thread.layout().tls;
    let base = tls
        .start
        .0
        .checked_add(tls.size)
        .and_then(|v| v.checked_add(gran - 1))
        .ok_or(ClosureError::Budget)?
        & !(gran - 1);
    let available = max_runtime_bytes
        .checked_sub(guest.thread.layout().reserved_bytes)
        .ok_or(ClosureError::Budget)?;
    let mut stubs = Vec::new();
    stubs
        .try_reserve(
            n.checked_mul(24)
                .ok_or(ClosureError::Budget)?
                .min(usize::try_from(available).map_err(|_| ClosureError::Budget)?),
        )
        .map_err(|_| ClosureError::Allocation)?;
    for (i, r) in plan.relocations().iter().enumerate() {
        if r.action == Action::None || (r.value.is_some() && r.resolution.is_none()) {
            continue;
        }
        let sym = r.raw.record.symbol_index();
        let symbol = plan
            .input()
            .linkage()
            .symbols()
            .entries()
            .find(|(_, s)| s.fields().index == sym as u64)
            .map(|(_, s)| s);
        // Only proven function slots with zero addend and an owned writable target. Do not
        // redirect OBJECT/TLS/NOTYPE into executable stubs or fabricate data semantics.
        if r.width != 8
            || !matches!(
                r.action,
                Action::JumpSlot64 | Action::GlobDat64 | Action::Absolute64
            )
            || r.raw.record.addend != 0
            || symbol.is_none_or(|s| s.fields().symbol_type != SymbolType::Function)
            || r.place.is_none()
        {
            remaining.push(MustClose::PendingWrite {
                relocation: i,
                kind: r.raw.record.relocation_type(),
                symbol: sym,
            });
            continue;
        }
        let key = plan.reference_identity(sym as u64).and_then(|(nid, l, m)| {
            Some(ProviderKey {
                nid,
                library: source.read(&l).ok()?.to_vec(),
                module: source.read(&m).ok()?.to_vec(),
            })
        });
        let ordinal = u32::try_from(keys.len()).map_err(|_| ClosureError::Budget)?;
        let address = base
            .checked_add(stubs.len() as u64)
            .ok_or(ClosureError::Budget)?;
        if (stubs.len() as u64)
            .checked_add(24)
            .is_none_or(|n| n > available)
        {
            return Err(ClosureError::Budget);
        }
        stubs.extend(import_stub(ordinal, bridge.import_landing()).ok_or(ClosureError::Budget)?);
        keys.push(key);
        patches.push((r.place.unwrap().0, address));
    }
    let size = (stubs.len() as u64)
        .max(1)
        .checked_add(page - 1)
        .ok_or(ClosureError::Budget)?
        & !(page - 1);
    if guest
        .thread
        .layout()
        .reserved_bytes
        .checked_add(size)
        .is_none_or(|v| v > max_runtime_bytes)
    {
        return Err(ClosureError::Budget);
    }
    stubs
        .try_reserve(size as usize - stubs.len())
        .map_err(|_| ClosureError::Allocation)?;
    stubs.resize(size as usize, 0xcc);
    let (landings, _) = windows_native::realize(
        &[NativeRegion {
            range: GuestRange {
                start: GuestAddress(base),
                size,
            },
            bytes: &stubs,
            protection: Protection {
                read: true,
                write: false,
                execute: true,
            },
        }],
        NativeLimits {
            max_reserved_bytes: size,
            max_committed_bytes: size,
        },
    );
    let landings = landings.map_err(|e| ClosureError::Native {
        operation: "landings",
        error: e,
    })?;
    let mut groups = std::collections::BTreeMap::<u32, (i64, i64)>::new();
    let mut object_writes = Vec::new();
    object_writes
        .try_reserve(remaining.len())
        .map_err(|_| ClosureError::Allocation)?;
    if policy == StartupPolicy::ExperimentalEntryOwnedInit {
        for b in &remaining {
            if let MustClose::PendingWrite {
                relocation, symbol, ..
            } = b
            {
                let r = &plan.relocations()[*relocation];
                let object = plan.input().linkage().symbols().entries().any(|(_, s)| {
                    s.fields().index == u64::from(*symbol)
                        && s.fields().symbol_type == SymbolType::Object
                });
                if object
                    && r.width == 8
                    && r.place.is_some()
                    && matches!(r.action, Action::Absolute64 | Action::GlobDat64)
                {
                    let addend = if r.action == Action::GlobDat64 {
                        0
                    } else {
                        r.raw.record.addend
                    };
                    let bounds = groups.entry(*symbol).or_insert((0, 0));
                    bounds.0 = bounds.0.min(addend);
                    bounds.1 = bounds.1.max(addend);
                    object_writes.push((*relocation, *symbol, addend));
                }
            }
        }
    }
    let trap_start = base
        .checked_add(size)
        .and_then(|v| v.checked_add(gran - 1))
        .ok_or(ClosureError::Budget)?
        & !(gran - 1);
    let traps = super::traps::build(
        trap_start,
        page,
        available.checked_sub(size).ok_or(ClosureError::Budget)?,
        &groups,
    )?;
    let foundation_base = trap_start
        .checked_add(traps.bytes)
        .and_then(|v| v.checked_add(gran - 1))
        .ok_or(ClosureError::Budget)?
        & !(gran - 1);
    let foundation = if migrate {
        Some(Arc::new(super::foundation::Foundation::build(
            foundation_base,
            page,
            available
                .checked_sub(size)
                .and_then(|v| v.checked_sub(traps.bytes))
                .ok_or(ClosureError::Budget)?,
        )?))
    } else {
        None
    };
    for &(index, symbol, addend) in &object_writes {
        let t = traps
            .records
            .iter()
            .find(|t| t.symbol == symbol)
            .ok_or(ClosureError::Budget)?;
        let value = u64::try_from(i128::from(t.anchor) + i128::from(addend))
            .map_err(|_| ClosureError::Budget)?;
        let key = plan
            .reference_identity(symbol as u64)
            .and_then(|(nid, l, m)| {
                Some(ProviderKey {
                    nid,
                    library: source.read(&l).ok()?.to_vec(),
                    module: source.read(&m).ok()?.to_vec(),
                })
            });
        let value = if let Some(f) = &foundation {
            if key.as_ref() == Some(&f.guard.key) {
                f.guard
                    .address_with_addend(addend)
                    .ok_or(ClosureError::Budget)?
            } else {
                value
            }
        } else {
            value
        };
        patches.push((plan.relocations()[index].place.unwrap().0, value));
    }
    remaining.retain(|b|!matches!(b,MustClose::PendingWrite{relocation,..} if object_writes.iter().any(|(i,_,_)|i==relocation)));

    for &(address, value) in &patches {
        guest
            .image
            .write(address, &value.to_le_bytes())
            .map_err(|error| ClosureError::Write { address, error })?;
    }
    guest
        .thread
        .as_ref()
        .install_return(bridge.return_landing())
        .map_err(|e| ClosureError::Native {
            operation: "return slot",
            error: e,
        })?;
    guest.context.gpr[4] = bridge.return_landing();
    let executable: Vec<_> = guest
        .image
        .pages()
        .iter()
        .filter(|p| p.protection.execute)
        .map(|p| (p.range.start.0, p.range.size))
        .collect();
    let (mut registrations, startup) = astero_libs::libc::startup::owned_registrations(
        256,
        executable.clone(),
        migrate.then_some(bridge.return_landing()),
    );
    let mut synchronization = None;
    let mut sync_timing = None;
    if migrate {
        let timing =
            astero_timing::scheduler::TimingEngine::real(astero_timing::scheduler::Config {
                max_pending: 128,
                max_snapshot_entries: 128,
            })
            .map_err(|_| ClosureError::Allocation)?;
        let service = std::sync::Arc::new(
            astero_kernel::synchronization::owned::Synchronization::new(
                timing.scheduler(),
                1024,
                128,
            )
            .map_err(|_| ClosureError::Allocation)?,
        );
        registrations.extend(astero_libs::pthread::exports::registrations(
            service.clone(),
            astero_kernel::synchronization::owned::Thread(1),
        ));
        synchronization = Some(service);
        sync_timing = Some(timing);
        registrations.push(astero_libs::libc::startup::cxa_registration(
            startup.clone(),
            executable,
        ));
        registrations.extend(astero_libs::libc::process::registrations(
            guest.thread.layout().thread_pointer + 8,
            guest.bootstrap.procparam.as_ref().map_or(0, |p| p.start.0),
        ));
        registrations.extend(astero_libs::libc::primitives::registrations_with_errno(
            b"libc",
            b"libc",
            Some(guest.thread.layout().thread_pointer + 8),
        ));
        registrations.extend(astero_libs::libc::primitives::registrations_with_errno(
            b"libkernel",
            b"libkernel",
            Some(guest.thread.layout().thread_pointer + 8),
        ));
    }
    guest.registry = PreparedRegistry::new(registrations, 256).map_err(ClosureError::Registry)?;
    let mut finalized = 0;
    let mut relro_pending = 0;
    let mut relro_hole_bytes = 0;
    for r in &guest.bootstrap.relro {
        if r.size == 0 {
            continue;
        }
        let end = r.start.0.checked_add(r.size).ok_or(ClosureError::Budget)?;
        let pending = remaining.iter().any(|b| match b {
            MustClose::PendingWrite { relocation, .. } => {
                let p = &plan.relocations()[*relocation];
                p.place
                    .is_none_or(|a| a.0 < end && a.0.saturating_add(p.width as u64) > r.start.0)
            }
            _ => false,
        });
        if pending || !r.start.0.is_multiple_of(page) || !r.size.is_multiple_of(page) {
            relro_pending += 1;
            continue;
        }
        let pages: Vec<_> = guest
            .image
            .pages()
            .iter()
            .filter(|p| p.range.start.0 >= r.start.0 && p.range.start.0 < end)
            .cloned()
            .collect();
        relro_hole_bytes += r.size - pages.len() as u64 * page;
        for p in pages {
            guest
                .image
                .seal_read_only(p.range)
                .map_err(|error| ClosureError::Native {
                    operation: "RELRO",
                    error,
                })?;
        }
        finalized += 1;
    }
    for b in &guest.blockers {
        if let EntryBlocker::UnknownProgram { kind, .. } = b
            && matches!(*kind, 0x6fffff00 | 0x6fffff01)
        {
            continue;
        }
        if policy == StartupPolicy::ExperimentalEntryOwnedInit
            && matches!(
                b,
                EntryBlocker::InitializersUnprepared | EntryBlocker::TcbLayoutUnproven
            )
        {
            continue;
        }
        // Only close mechanisms actually supplied. TCB/startup/program semantics stay explicit.
        if matches!(
            b,
            EntryBlocker::TcbLayoutUnproven
                | EntryBlocker::EntryUnavailable
                | EntryBlocker::EntryNotExecutable
                | EntryBlocker::InitializersUnprepared
                | EntryBlocker::UnknownProgram { .. }
        ) {
            remaining.push(MustClose::Entry(b.clone()));
        }
    }
    let ranges: Vec<_> = guest
        .image
        .pages()
        .iter()
        .filter(|p| p.protection.execute)
        .map(|p| (p.range.start.0, p.range.size))
        .chain(std::iter::once((base, size)))
        .collect();
    bridge
        .prepare_ranges(&ranges)
        .map_err(ClosureError::Bridge)?;
    if relro_pending != 0 {
        remaining.push(MustClose::RelroPending);
    }
    let runtime = if let (Some(f), Some(sync), Some(timing)) =
        (&foundation, &synchronization, &sync_timing)
    {
        let runtime = super::workers::Runtime::new(
            &guest,
            f.clone(),
            sync.clone(),
            timing.scheduler(),
            startup.clone(),
            &bridge,
            keys.clone(),
        )?;
        guest.registry = runtime
            .registry(
                astero_kernel::synchronization::owned::Thread(1),
                &guest.thread,
                Arc::new(Mutex::new(None)),
            )
            .map_err(ClosureError::Registry)?;
        Some(runtime)
    } else {
        None
    };
    let startup_matches = keys
        .iter()
        .flatten()
        .filter(|k| guest.registry.find(k).is_ok())
        .map(|k| k.nid)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let report = ClosureReport {
        source: source.identity(),
        object_trap_writes: object_writes.len(),
        guarded_objects: traps.records.len(),
        object_trap_bytes: traps.bytes,
        pending_before: guest.image.staging_snapshot().pending_relocations,
        treated: patches.len(),
        pending_after: remaining
            .iter()
            .filter(|b| matches!(b, MustClose::PendingWrite { .. }))
            .count(),
        landings: keys.len(),
        providers: guest.registry.len(),
        startup_matches,
        bridge_validated: bridge.validated(),
        remaining,
        relro_finalized: finalized,
        relro_pending,
        relro_hole_bytes,
        landing_bytes: size,
        unresolved_function_landings: keys
            .iter()
            .filter(|k| k.as_ref().is_none_or(|k| guest.registry.find(k).is_err()))
            .count(),
        early_runtime: vec![
            "Unimplemented function calls stop with ordinal/key evidence",
            "Guarded OBJECT addresses are unresolved; dereference must stop, never provide object contents",
            "Experimental TCB self-pointer/slack and _init_env no-op; not firmware-complete TLS/environment",
            "Caller selected entry-owned DT_INIT; constructors and callbacks have not executed",
            "No arbitrary-guest security sandbox or asynchronous stop guarantee",
        ],
        policy,
    };
    Ok(ClosureGuest {
        guest,
        bridge,
        landings,
        traps,
        keys,
        startup,
        report,
        foundation,
        synchronization,
        sync_timing,
        runtime,
    })
}

/// First-entry policy permits one _init_env and two atexit calls, then stops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FirstEntryStop {
    Returned,
    UnresolvedFunction,
    StartupAllowanceExhausted,
    GuardedObject,
    AccessViolation,
    IllegalInstruction,
    SupervisorExpired,
    BridgeFailure,
    ProviderStopped,
    ProviderRefused,
}
#[derive(Clone, Debug)]
pub struct StartupCall {
    pub result: Result<astero_hle::dispatch::prepared::CallResult, RegistryError>,
    pub ordinal: u32,
    pub key: ProviderKey,
    pub arguments: [u64; 6],
    pub scalar_arguments: [[u8; 16]; 2],
    pub scalar_returned: [u8; 16],
    pub returned: u64,
}
#[derive(Debug)]
pub struct FirstEntryReport {
    pub threads: Vec<astero_kernel::threading::thread::lifecycle::Record>,
    pub worker_exits: Vec<(
        astero_kernel::synchronization::owned::Thread,
        astero_kernel::execution::host::NativeExit,
    )>,
    pub worker_calls: Vec<(astero_kernel::synchronization::owned::Thread, StartupCall)>,
    pub worker_active_reservations: u64,
    pub worker_release_errors: Vec<u32>,
    pub source: astero_loader::artifact::SourceId,
    pub initial: InitialContext,
    pub stop: FirstEntryStop,
    pub native: astero_kernel::execution::host::NativeExit,
    pub startup_calls: Vec<StartupCall>,
    pub unresolved: Option<ProviderKey>,
    pub guarded_object: Option<super::traps::ObjectTrap>,
    pub guarded_relocations:
        Vec<astero_loader::elf::dynamic::relocations::observation::RawRelocation>,
    pub synchronization: Option<astero_kernel::synchronization::owned::Snapshot>,
    pub callback_count: usize,
    pub provider_failure: Option<astero_hle::calls::memory::AccessError>,
    pub registry_failure: Option<RegistryError>,
    pub elapsed_micros: u128,
    pub heap: Option<astero_memory::allocation::heap::HeapSnapshot>,
    pub users: Option<astero_kernel::process::users::Snapshot>,
    pub guest_timing: Option<astero_kernel::timing::sleep::Snapshot>,
    pub audio: Option<astero_audio::output::service::Snapshot>,
    pub formatting: Vec<astero_libs::libc::formatting::exports::Observation>,
    pub output: Vec<u8>,
    pub access_budget: Option<astero_hle::calls::budget::AccessSnapshot>,
    pub data_export: Option<astero_hle::providers::data::DataExport>,
}
impl EntryReadyGuest {
    /// Consumes authority on the dedicated owning thread. Does not invoke initializers/fini.
    fn execute_first(mut self, millis: u64) -> Result<FirstEntryReport, BridgeError> {
        let call_budget = self.call_budget;
        let owner = &mut self.owner;
        if let Some(runtime) = &owner.runtime {
            runtime.configure_call_budget(call_budget)?;
        }
        let initial = owner.guest.context.clone();
        let source = owner.report.source;
        let keys = &owner.keys;
        let registry = &owner.guest.registry;
        let mut calls = Vec::new();
        calls
            .try_reserve_exact(call_budget)
            .map_err(|_| BridgeError::Validation)?;
        let mut counts = [0, 0];
        let mut exhausted = false;
        let mut stopped = false;
        let mut refused = false;
        let mut provider_failure = None;
        let mut registry_failure = None;
        if let (Some(sync), Some(timing)) = (&owner.synchronization, &owner.sync_timing) {
            let deadline = astero_timing::time::Deadline::after(
                timing
                    .scheduler()
                    .now()
                    .map_err(|_| BridgeError::Validation)?,
                astero_timing::time::Span::from_millis(millis)
                    .map_err(|_| BridgeError::Validation)?,
            )
            .map_err(|_| BridgeError::Validation)?;
            sync.arm(deadline).map_err(|_| BridgeError::Validation)?;
            if let Some(runtime) = &owner.runtime {
                runtime.table.arm(deadline);
                runtime
                    .guards
                    .arm(deadline)
                    .map_err(|_| BridgeError::Validation)?;
                runtime.audio.arm(deadline);
                runtime.guest_timing.arm(deadline);
            }
        }
        let started = std::time::Instant::now();
        let native = owner.bridge.execute_prepared(
            owner.guest.image.native_owner(),
            &owner.guest.thread,
            &initial,
            millis,
            &mut |ordinal, frame| {
                let Some(key) = keys.get(ordinal as usize).and_then(|k| k.as_ref()) else {
                    return false;
                };
                if calls.len() >= call_budget {
                    exhausted = true;
                    return false;
                }
                if owner.foundation.is_none() {
                    let slot = match key.nid {
                        astero_libs::libc::startup::INIT_ENV_NID => 0,
                        astero_libs::libc::startup::ATEXIT_NID => 1,
                        _ => return false,
                    };
                    if counts[slot] >= if slot == 0 { 1 } else { 2 } {
                        exhausted = true;
                        return false;
                    }
                    counts[slot] += 1;
                }
                let args = frame.arguments;
                let result = if let Some(runtime) = &owner.runtime {
                    if runtime.table.stopped() {
                        stopped = true;
                        return false;
                    }
                    if !runtime.admit_call() {
                        exhausted = true;
                        return false;
                    }
                    registry.invoke(key, frame, &mut runtime.access())
                } else if let Some(f) = &mut owner.foundation {
                    let mut access = super::foundation::Access {
                        guest: &owner.guest,
                        foundation: f,
                    };
                    registry.invoke(key, frame, &mut access)
                } else {
                    registry.invoke_host_model(key, frame)
                };
                match result {
                    Ok(astero_hle::dispatch::prepared::CallResult::Returned) => {}
                    Ok(astero_hle::dispatch::prepared::CallResult::StopRequested) => {
                        stopped = true;
                    }
                    Ok(
                        astero_hle::dispatch::prepared::CallResult::Unsupported
                        | astero_hle::dispatch::prepared::CallResult::FormatFailure { .. },
                    ) => {
                        refused = true;
                    }
                    Ok(astero_hle::dispatch::prepared::CallResult::AccessFailure(e)) => {
                        refused = true;
                        provider_failure = Some(e);
                    }
                    Err(RegistryError::Missing) => return false,
                    Err(e) => {
                        registry_failure = Some(e);
                        refused = true;
                    }
                }
                calls.push(StartupCall {
                    result,
                    ordinal,
                    key: key.clone(),
                    arguments: args,
                    scalar_arguments: [frame.xmm[0], frame.xmm[1]],
                    scalar_returned: frame.xmm0,
                    returned: frame.rax,
                });
                !stopped && !refused
            },
        )?;
        let object = owner.trapped_object(native.fault_address).cloned();
        let stop = match native.reason {
            0 => FirstEntryStop::Returned,
            2 if stopped => FirstEntryStop::ProviderStopped,
            2 if refused => FirstEntryStop::ProviderRefused,
            2 if exhausted => FirstEntryStop::StartupAllowanceExhausted,
            2 => FirstEntryStop::UnresolvedFunction,
            4 => FirstEntryStop::SupervisorExpired,
            3 if object.is_some() => FirstEntryStop::GuardedObject,
            3 if native.exception == 0xc000001d => FirstEntryStop::IllegalInstruction,
            3 if native.exception == 0xc0000005 => FirstEntryStop::AccessViolation,
            _ => FirstEntryStop::BridgeFailure,
        };
        let unresolved = if stop == FirstEntryStop::UnresolvedFunction {
            owner.import_key(native.import_ordinal).cloned()
        } else {
            None
        };
        let mut guarded_relocations = Vec::new();
        if let Some(t) = &object {
            for r in owner
                .guest
                .image
                .plan()
                .relocations()
                .iter()
                .filter(|r| r.raw.record.symbol_index() == t.symbol)
            {
                guarded_relocations
                    .try_reserve(1)
                    .map_err(|_| BridgeError::Validation)?;
                guarded_relocations.push(r.raw.clone());
            }
        }
        let callback_count = owner.retained_callback_count();
        if let Some(runtime) = &owner.runtime {
            use astero_kernel::threading::thread::lifecycle::{Outcome, ThreadEnd};
            runtime.table.complete_main(Outcome {
                value: native.value,
                reason: match stop {
                    FirstEntryStop::Returned => ThreadEnd::Returned,
                    FirstEntryStop::AccessViolation
                    | FirstEntryStop::IllegalInstruction
                    | FirstEntryStop::GuardedObject => ThreadEnd::NativeFault,
                    FirstEntryStop::SupervisorExpired => ThreadEnd::Interrupted,
                    _ => ThreadEnd::ProviderStop,
                },
            });
            runtime.stop_and_join();
        }
        if let Some(sync) = &owner.synchronization {
            sync.shutdown();
        }
        let synchronization = owner.synchronization.as_ref().map(|s| s.snapshot());
        Ok(FirstEntryReport {
            threads: owner
                .runtime
                .as_ref()
                .map_or_else(Vec::new, |r| r.table.snapshot()),
            worker_exits: owner.runtime.as_ref().map_or_else(Vec::new, |r| {
                r.exits.lock().unwrap_or_else(|p| p.into_inner()).clone()
            }),
            worker_calls: owner.runtime.as_ref().map_or_else(Vec::new, |r| {
                r.calls.lock().unwrap_or_else(|p| p.into_inner()).clone()
            }),
            worker_active_reservations: owner.runtime.as_ref().map_or(0, |r| {
                r.observers
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .iter()
                    .map(|o| o.active_reservations())
                    .sum()
            }),
            worker_release_errors: owner.runtime.as_ref().map_or_else(Vec::new, |r| {
                r.observers
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .iter()
                    .map(|o| o.release_error())
                    .filter(|c| *c != 0)
                    .collect()
            }),
            source,
            initial,
            stop,
            native,
            startup_calls: calls,
            unresolved,
            guarded_object: object,
            guarded_relocations,
            callback_count,
            synchronization,
            provider_failure,
            registry_failure,
            elapsed_micros: started.elapsed().as_micros(),
            users: owner
                .foundation
                .as_ref()
                .map(|f| f.users.lock().unwrap_or_else(|p| p.into_inner()).snapshot()),
            guest_timing: owner.runtime.as_ref().map(|r| r.guest_timing.snapshot()),
            audio: owner.runtime.as_ref().map(|r| r.audio.snapshot()),
            formatting: owner
                .foundation
                .as_ref()
                .map(|f| f.formatting.snapshot())
                .unwrap_or_default(),
            output: owner
                .foundation
                .as_ref()
                .map(|f| {
                    f.output
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .bytes()
                        .to_vec()
                })
                .unwrap_or_default(),
            access_budget: owner
                .foundation
                .as_ref()
                .map(|f| f.access_budget.snapshot()),
            heap: owner
                .foundation
                .as_ref()
                .map(|f| f.heap.lock().unwrap_or_else(|p| p.into_inner()).snapshot()),
            data_export: owner.foundation.as_ref().map(|f| f.guard.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ExecutionReport {
    pub first: FirstEntryReport,
    pub thread_joined: bool,
    pub active_reservations: u64,
    pub release_errors: Vec<u32>,
}
/// Preparation happens on its final owning thread; !Send native resources never migrate.
/// Only the factory's privately constructed EntryReadyGuest can enter the bridge.
pub fn execute_first_entry<F>(prepare: F, millis: u64) -> Result<ExecutionReport, String>
where
    F: FnOnce() -> Result<EntryReadyGuest, String> + Send + 'static,
{
    if !(1..=500).contains(&millis) {
        return Err("execution limit must be 1..=500 ms".into());
    }
    std::thread::Builder::new()
        .name("astero-first-guest".into())
        .spawn(move || {
            let ready = prepare()?;
            let mut observers = ready.owner.guest.thread.observers().to_vec();
            observers.push(ready.owner.guest.image.native_owner().observer());
            observers.push(ready.owner.landings.observer());
            if let Some(i) = &ready.owner.traps.image {
                observers.push(i.observer());
            }
            if let Some(f) = &ready.owner.foundation {
                observers.push(f.image.observer());
            }
            let first = ready
                .execute_first(millis)
                .map_err(|e| format!("Bridge: {e:?}"))?;
            let active_reservations = observers
                .iter()
                .map(|o| o.active_reservations())
                .sum::<u64>()
                + first.worker_active_reservations;
            let mut release_errors: Vec<_> = observers
                .iter()
                .map(|o| o.release_error())
                .filter(|e| *e != 0)
                .collect();
            release_errors.extend(&first.worker_release_errors);
            Ok(ExecutionReport {
                first,
                thread_joined: true,
                active_reservations,
                release_errors,
            })
        })
        .map_err(|e| format!("Guest thread start: {e}"))?
        .join()
        .map_err(|_| "Guest thread panicked".to_string())?
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainmentExit {
    Clean,
    WorkerFailure(Option<i32>),
    TimeoutKilled,
}
/// Additional process containment, never reported as clean native recovery.
pub fn contain_worker(
    child: std::process::Child,
    maximum_ms: u64,
) -> Result<ContainmentExit, String> {
    struct OwnedChild(std::process::Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if !matches!(self.0.try_wait(), Ok(Some(_))) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    let mut owned = OwnedChild(child);
    let child = &mut owned.0;
    use astero_timing::{
        scheduler::{Config, Label, TimingEngine},
        time::Span,
    };
    let engine = TimingEngine::real(Config {
        max_pending: 2,
        max_snapshot_entries: 2,
    })
    .map_err(|e| format!("Containment timer: {e:?}"))?;
    let scheduler = engine.scheduler();
    let end = scheduler
        .after(
            Span::from_millis(maximum_ms).map_err(|e| format!("{e:?}"))?,
            Label::new("entry containment").unwrap(),
        )
        .map_err(|e| format!("{e:?}"))?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return Ok(if status.success() {
                ContainmentExit::Clean
            } else {
                ContainmentExit::WorkerFailure(status.code())
            });
        }
        if end.poll().is_some() {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
            return Ok(ContainmentExit::TimeoutKilled);
        }
        scheduler
            .after(
                Span::from_millis(10).unwrap(),
                Label::new("worker completion").unwrap(),
            )
            .map_err(|e| format!("{e:?}"))?
            .wait();
    }
}
