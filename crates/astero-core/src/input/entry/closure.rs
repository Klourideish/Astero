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
use std::{cell::RefCell, rc::Rc};
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
    startup: Rc<RefCell<astero_libs::libc::startup::StartupState>>,
    report: ClosureReport,
}
/// Future execution must consume this authority, not a loose readiness Boolean.
/// No public constructor; unresolved closure never manufactures the capability.
pub struct EntryReadyGuest {
    owner: ClosureGuest,
}
impl EntryReadyGuest {
    pub fn report(&self) -> &ClosureReport {
        &self.owner.report
    }
}
impl ClosureGuest {
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
            Ok(EntryReadyGuest { owner: self })
        } else {
            Err(Box::new(self))
        }
    }
    pub fn retained_callback_count(&self) -> usize {
        self.startup.borrow().callbacks().count()
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
    mut guest: PreparedGuest,
    max_runtime_bytes: u64,
    policy: StartupPolicy,
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
    for &(index, symbol, addend) in &object_writes {
        let t = traps
            .records
            .iter()
            .find(|t| t.symbol == symbol)
            .ok_or(ClosureError::Budget)?;
        let value = u64::try_from(i128::from(t.anchor) + i128::from(addend))
            .map_err(|_| ClosureError::Budget)?;
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
        .install_return(bridge.return_landing())
        .map_err(|e| ClosureError::Native {
            operation: "return slot",
            error: e,
        })?;
    guest.context.gpr[4] = bridge.return_landing();
    let executable = guest
        .image
        .pages()
        .iter()
        .filter(|p| p.protection.execute)
        .map(|p| (p.range.start.0, p.range.size))
        .collect();
    let (registrations, startup) = astero_libs::libc::startup::registrations(256, executable);
    guest.registry = PreparedRegistry::new(registrations, 2).map_err(ClosureError::Registry)?;
    let startup_matches = keys
        .iter()
        .flatten()
        .filter(|k| guest.registry.find(k).is_ok())
        .map(|k| k.nid)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
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
    })
}
