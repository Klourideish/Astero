//! Runtime composition for a bounded shared process and independent supervised native workers.
use super::*;
use astero_hle::calls::memory::{AccessError, GuestMemory};
use astero_kernel::{
    execution::{
        host::{BridgeLease, NativeExit},
        preparation::storage::ThreadStorage,
    },
    synchronization::owned::Synchronization,
    threading::thread::{
        attributes::{AttributeTable, Attributes},
        lifecycle::{Error, Outcome, Result as ThreadResult, Thread, ThreadEnd, ThreadTable},
    },
};
use astero_memory::mapping::windows_native::{NativeImage, NativeObserver};
use std::sync::{Arc, Mutex, Weak};
pub const MAX_WORKERS: usize = 32;
/// Placement is runtime policy in a reserved-purpose address band; OS conflicts refuse creation.
const WORKER_BASE: u64 = 0x240000000;
const WORKER_STRIDE: u64 = 0x1000000;
pub struct Runtime {
    attempts: std::sync::atomic::AtomicUsize,
    pub table: Arc<ThreadTable>,
    attrs: Arc<AttributeTable>,
    image: Arc<NativeImage>,
    main: Arc<ThreadStorage>,
    storage: Mutex<Vec<(Thread, Arc<ThreadStorage>)>>,
    pub observers: Mutex<Vec<NativeObserver>>,
    pub exits: Mutex<Vec<(Thread, NativeExit)>>,
    pub calls: Mutex<Vec<(Thread, super::StartupCall)>>,
    foundation: Arc<super::foundation::Foundation>,
    synchronization: Arc<Synchronization>,
    startup: Arc<Mutex<astero_libs::libc::startup::StartupState>>,
    environment: Arc<Mutex<astero_kernel::process::environment::Environment>>,
    bridge: BridgeLease,
    keys: Vec<Option<ProviderKey>>,
    executable: Vec<(u64, u64)>,
    procparam: u64,
    return_landing: u64,
    template: Vec<u8>,
}
impl Runtime {
    pub fn new(
        guest: &PreparedGuest,
        foundation: Arc<super::foundation::Foundation>,
        synchronization: Arc<Synchronization>,
        scheduler: astero_timing::scheduler::Scheduler,
        startup: Arc<Mutex<astero_libs::libc::startup::StartupState>>,
        bridge: &astero_kernel::execution::host::Bridge,
        keys: Vec<Option<ProviderKey>>,
    ) -> std::result::Result<Arc<Self>, ClosureError> {
        let l = guest.thread.layout();
        let template = guest
            .thread
            .read_tls(l.tls.start.0, l.tls_template_size)
            .map_err(|error| ClosureError::Native {
                operation: "worker TLS template",
                error,
            })?;
        let table = Arc::new(ThreadTable::new(scheduler, MAX_WORKERS));
        table.initialize_main(guest.context.rip, guest.context.gpr[5], l.clone());
        let mut storage = Vec::new();
        let mut observers = Vec::new();
        let mut exits = Vec::new();
        let mut calls = Vec::new();
        storage
            .try_reserve_exact(MAX_WORKERS)
            .map_err(|_| ClosureError::Allocation)?;
        observers
            .try_reserve_exact(MAX_WORKERS * 2)
            .map_err(|_| ClosureError::Allocation)?;
        exits
            .try_reserve_exact(MAX_WORKERS)
            .map_err(|_| ClosureError::Allocation)?;
        calls
            .try_reserve_exact(4096)
            .map_err(|_| ClosureError::Allocation)?;
        Ok(Arc::new(Self {
            attempts: std::sync::atomic::AtomicUsize::new(0),
            table,
            attrs: Arc::new(AttributeTable::new(256)),
            image: guest.image.shared_owner(),
            main: guest.thread.clone(),
            storage: Mutex::new(storage),
            observers: Mutex::new(observers),
            exits: Mutex::new(exits),
            calls: Mutex::new(calls),
            foundation,
            synchronization,
            startup,
            environment: Arc::new(Mutex::new(
                astero_kernel::process::environment::Environment::new(),
            )),
            bridge: bridge.worker_lease().map_err(ClosureError::Bridge)?,
            keys,
            executable: guest
                .image
                .pages()
                .iter()
                .filter(|p| p.protection.execute)
                .map(|p| (p.range.start.0, p.range.size))
                .collect(),
            procparam: guest.bootstrap.procparam.as_ref().map_or(0, |p| p.start.0),
            return_landing: bridge.return_landing(),
            template,
        }))
    }
    pub fn registry(
        self: &Arc<Self>,
        thread: Thread,
        storage: &ThreadStorage,
        exit: Arc<Mutex<Option<u64>>>,
    ) -> std::result::Result<PreparedRegistry, RegistryError> {
        let mut entries = astero_libs::libc::startup::shared_registrations(
            self.startup.clone(),
            self.executable.clone(),
            Some(self.return_landing),
        );
        entries.push(astero_libs::libc::startup::cxa_registration(
            self.startup.clone(),
            self.executable.clone(),
        ));
        let errno = storage.layout().thread_pointer + 8;
        entries.extend(astero_libs::libc::process::shared_registrations(
            errno,
            self.procparam,
            self.environment.clone(),
        ));
        for lib in [b"libc".as_slice(), b"libkernel".as_slice()] {
            entries.extend(astero_libs::libc::primitives::registrations_with_errno(
                lib,
                lib,
                Some(errno),
            ));
        }
        entries.push(astero_libs::libc::output::registration(
            self.foundation.output.clone(),
        ));
        entries.extend(astero_libs::pthread::exports::registrations(
            self.synchronization.clone(),
            thread,
        ));
        entries.extend(astero_libs::pthread::thread::exports::registrations(
            self.attrs.clone(),
            self.table.clone(),
            Arc::new(Spawn {
                runtime: Arc::downgrade(self),
            }),
            thread,
            exit,
        ));
        PreparedRegistry::new(entries, 256)
    }
    pub fn access(&self) -> Access<'_> {
        Access(self)
    }
    pub fn admit_call(&self) -> bool {
        self.attempts
            .fetch_update(
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
                |n| (n < super::foundation::MAX_PROVIDER_CALLS).then(|| n + 1),
            )
            .is_ok()
    }
    pub fn stop_and_join(&self) {
        self.table.request_stop();
        self.synchronization.shutdown();
        self.table.reap_all();
        self.storage
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }
    fn create(
        self: &Arc<Self>,
        attr: Attributes,
        start: u64,
        argument: u64,
        name: Vec<u8>,
        output: u64,
        m: &mut dyn GuestMemory,
    ) -> ThreadResult {
        if !self
            .executable
            .iter()
            .any(|&(a, n)| start >= a && start - a < n)
            || output == 0
        {
            return Err(Error::Invalid);
        }
        // Publication is last; validate output permissions before allocating/starting anything.
        let old = m.read(output, 8).map_err(|_| Error::Invalid)?;
        m.write(output, &old).map_err(|_| Error::Invalid)?;
        let id = self.table.reserve(attr.clone(), start, argument, name)?;
        let prepared = (|| {
            let geometry = astero_memory::mapping::windows_native::host_geometry()
                .map_err(|_| Error::Capacity)?;
            let base = WORKER_BASE
                .checked_add(
                    (id.0 - 2)
                        .checked_mul(WORKER_STRIDE)
                        .ok_or(Error::Capacity)?,
                )
                .ok_or(Error::Capacity)?;
            let initial = self.main.layout();
            let layout = astero_kernel::execution::preparation::layout::plan(
                RuntimeLimits {
                    stack_base: base,
                    stack_bytes: attr.stack_size,
                    tls_base: base + 0x900000,
                    max_runtime_bytes: WORKER_STRIDE,
                },
                geometry,
                initial.tls_template_size,
                initial.tls_memory_size,
                initial.tls_alignment,
            )
            .map_err(|_| Error::Capacity)?;
            let (storage, observers) =
                ThreadStorage::build(layout, &self.template).map_err(|_| Error::Capacity)?;
            self.table.set_storage(id, storage.layout().clone())?;
            storage
                .install_return(self.return_landing)
                .map_err(|_| Error::Fault)?;
            let storage = Arc::new(storage);
            self.observers
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .extend(observers);
            self.storage
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .push((id, storage.clone()));
            let runtime = self.clone();
            let result = self.table.launch(
                id,
                move || {
                    let outcome = runtime.run(id, &storage, start, argument);
                    runtime
                        .storage
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .retain(|(i, _)| *i != id);
                    outcome
                },
                || {
                    m.write(output, &id.0.to_le_bytes())
                        .map_err(|_| Error::Invalid)
                },
            );
            if result.is_err() {
                self.storage
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .retain(|(i, _)| *i != id);
            }
            result
        })();
        if prepared.is_err() {
            self.table.abandon(id);
        }
        prepared
    }
    fn run(
        self: &Arc<Self>,
        id: Thread,
        storage: &ThreadStorage,
        start: u64,
        argument: u64,
    ) -> Outcome {
        let exit = Arc::new(Mutex::new(None));
        let mut interrupted = false;
        let result = (|| {
            let registry = self
                .registry(id, storage, exit.clone())
                .map_err(|_| Error::Fault)?;
            let mut bridge = self.bridge.attach();
            let millis = self.table.remaining_millis()?;
            bridge
                .execute_worker(
                    &self.image,
                    storage,
                    (start, argument),
                    millis,
                    &mut |ordinal, frame| {
                        if self.table.stopped() {
                            interrupted = true;
                            return false;
                        }
                        let Some(key) = self.keys.get(ordinal as usize).and_then(|k| k.as_ref())
                        else {
                            return false;
                        };
                        if !self.admit_call() {
                            return false;
                        }
                        let args = frame.arguments;
                        let result = registry.invoke(key, frame, &mut self.access());
                        self.calls.lock().unwrap_or_else(|p| p.into_inner()).push((
                            id,
                            StartupCall {
                                result,
                                ordinal,
                                key: key.clone(),
                                arguments: args,
                                returned: frame.rax,
                            },
                        ));
                        let returned = matches!(
                            result,
                            Ok(astero_hle::dispatch::prepared::CallResult::Returned)
                        );
                        if self.table.stopped()
                            && matches!(
                                result,
                                Ok(astero_hle::dispatch::prepared::CallResult::Returned
                                    | astero_hle::dispatch::prepared::CallResult::StopRequested)
                            )
                        {
                            interrupted = true;
                        }
                        returned && !interrupted
                    },
                )
                .map_err(|_| Error::Fault)
        })();
        let explicit = *exit.lock().unwrap_or_else(|p| p.into_inner());
        match result {
            Ok(native) => {
                let normal = native.reason == 0 || (native.reason == 2 && explicit.is_some());
                let reason = match native.reason {
                    0 => ThreadEnd::Returned,
                    2 if explicit.is_some() => ThreadEnd::PthreadExit,
                    2 if interrupted => ThreadEnd::Interrupted,
                    2 => ThreadEnd::ProviderStop,
                    3 => ThreadEnd::NativeFault,
                    4 => ThreadEnd::Interrupted,
                    _ => ThreadEnd::BridgeFailure,
                };
                let value = explicit.unwrap_or(native.value);
                self.exits
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push((id, native));
                if !normal {
                    self.table.request_stop();
                    self.synchronization.shutdown();
                }
                Outcome { value, reason }
            }
            Err(error) => {
                self.table.request_stop();
                self.synchronization.shutdown();
                Outcome {
                    value: 0,
                    reason: if error == Error::Interrupted {
                        ThreadEnd::Interrupted
                    } else {
                        ThreadEnd::BridgeFailure
                    },
                }
            }
        }
    }
}
struct Spawn {
    runtime: Weak<Runtime>,
}
impl astero_libs::pthread::thread::exports::Spawner for Spawn {
    fn create(
        &self,
        attr: Attributes,
        start: u64,
        argument: u64,
        name: Vec<u8>,
        output: u64,
        memory: &mut dyn GuestMemory,
    ) -> ThreadResult {
        self.runtime
            .upgrade()
            .ok_or(Error::Interrupted)?
            .create(attr, start, argument, name, output, memory)
    }
}
pub struct Access<'a>(&'a Runtime);
impl Access<'_> {
    fn with_regions<T>(&self, f: impl FnOnce(&[&NativeImage]) -> T) -> T {
        let r = self.0;
        let storage = r.storage.lock().unwrap_or_else(|p| p.into_inner());
        let mut regions = vec![r.image.as_ref(), &r.foundation.image];
        regions.extend(r.main.native_regions());
        for (_, s) in storage.iter() {
            regions.extend(s.native_regions());
        }
        f(&regions)
    }
}
impl GuestMemory for Access<'_> {
    fn charge(&self, n: u64) -> std::result::Result<(), AccessError> {
        self.0.foundation.access_budget.charge(n)
    }
    fn validate(&self, a: u64, n: u64, write: bool) -> std::result::Result<(), AccessError> {
        self.with_regions(|r| astero_memory::access::native::validate(r, a, n, write))
            .map_err(super::foundation::access_error)
    }
    fn read_window(&self, a: u64, n: u64) -> std::result::Result<Vec<u8>, AccessError> {
        self.with_regions(|r| astero_memory::access::native::read_window(r, a, n))
            .map_err(super::foundation::access_error)
    }
    fn read(&self, a: u64, n: u64) -> std::result::Result<Vec<u8>, AccessError> {
        if n > astero_hle::calls::memory::COPY_CHUNK_BYTES {
            return Err(AccessError::Limit);
        }
        self.with_regions(|r| astero_memory::access::native::read(r, a, n))
            .map_err(super::foundation::access_error)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> std::result::Result<(), AccessError> {
        if b.len() as u64 > astero_hle::calls::memory::COPY_CHUNK_BYTES {
            return Err(AccessError::Limit);
        }
        self.with_regions(|r| astero_memory::access::native::write(r, a, b))
            .map_err(super::foundation::access_error)
    }
    fn allocate_aligned(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> std::result::Result<u64, AccessError> {
        self.0
            .foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .allocate_aligned(size, alignment)
            .map_err(|_| AccessError::Allocation)
    }
    fn usable_size(&self, a: u64) -> std::result::Result<u64, AccessError> {
        self.0
            .foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .usable_size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocate(&mut self, n: u64) -> std::result::Result<u64, AccessError> {
        self.0
            .foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .allocate(n)
            .map_err(|_| AccessError::Allocation)
    }
    fn free(&mut self, a: u64) -> std::result::Result<(), AccessError> {
        self.0
            .foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .free(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, a: u64) -> std::result::Result<u64, AccessError> {
        self.0
            .foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use astero_loader::{
        artifact::SourceArtifact,
        elf::dynamic::{
            bounded::DynamicLimits,
            candidates::workload::{self, LinkageLimits},
            hash::{HashLimits, bounded::HashMetadataLimits},
            identity::{self, IdentityLimits},
            symbol_table::bounded::SymbolObservationLimits,
            synthetic::{put32, put64},
        },
        load_plan::link::{PlanningLimits, plan},
        metadata::VirtualAddress,
    };
    static SERIAL: Mutex<()> = Mutex::new(());
    #[test]
    fn detached_native_return_releases_storage_without_guest_join() {
        let _serial = SERIAL.lock().unwrap();
        let (g, _b, r, _e) = fixture(&[0x48, 0x89, 0xf8, 0xc3], 0);
        r.create(
            Attributes {
                detached: true,
                ..Default::default()
            },
            0x220001100,
            51,
            vec![],
            g.thread.layout().stack.start.0,
            &mut r.access(),
        )
        .unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while r.table.snapshot()[1].state
            != astero_kernel::threading::thread::lifecycle::State::Exited
        {
            assert!(std::time::Instant::now() < until);
            std::thread::yield_now();
        }
        assert_eq!(
            r.table.snapshot()[1].outcome.as_ref().unwrap().reason,
            ThreadEnd::Returned
        );
        r.stop_and_join();
        assert_eq!(
            r.table.snapshot()[1].state,
            astero_kernel::threading::thread::lifecycle::State::Reclaimed
        );
        assert!(
            r.observers
                .lock()
                .unwrap()
                .iter()
                .all(|o| o.active_reservations() == 0)
        );
    }
    #[test]
    fn migrated_provider_budget_is_atomic_across_producers() {
        let _serial = SERIAL.lock().unwrap();
        let (_g, _b, r, _e) = fixture(&[0xc3], 0);
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let r = r.clone();
                std::thread::spawn(move || (0..4096).filter(|_| r.admit_call()).count())
            })
            .collect();
        assert_eq!(
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .sum::<usize>(),
            4096
        );
        assert!(!r.admit_call());
        r.stop_and_join();
    }
    fn fixture(
        code: &[u8],
        nid: u64,
    ) -> (
        PreparedGuest,
        astero_kernel::execution::host::Bridge,
        Arc<Runtime>,
        astero_timing::scheduler::TimingEngine,
    ) {
        fixture_impl(code, nid, false)
    }
    fn fixture_impl(
        code: &[u8],
        nid: u64,
        append_import: bool,
    ) -> (
        PreparedGuest,
        astero_kernel::execution::host::Bridge,
        Arc<Runtime>,
        astero_timing::scheduler::TimingEngine,
    ) {
        let mut bridge = astero_kernel::execution::host::Bridge::new().unwrap();
        bridge.validate().unwrap();
        let mut bytes = if code.is_empty() {
            astero_kernel::execution::preparation::boundary::import_stub(0, bridge.import_landing())
                .unwrap()
                .to_vec()
        } else {
            code.to_vec()
        };
        if append_import {
            bytes.extend_from_slice(
                &astero_kernel::execution::preparation::boundary::import_stub(
                    0,
                    bridge.import_landing(),
                )
                .unwrap(),
            );
        }
        let mut b = identity::synthetic::identity_image();
        put64(&mut b, 24, 0x1100);
        put32(&mut b, 68, 5);
        put64(&mut b, 0x668, 0);
        put64(&mut b, 0x698, 0);
        b[0x100..0x100 + bytes.len()].copy_from_slice(&bytes);
        let evidence = identity::observe(
            Arc::new(workload::observe(
                SourceArtifact::new(b, Some("M34 owned synthetic worker".into())).unwrap(),
                LinkageLimits {
                    hash: HashMetadataLimits {
                        dynamic: DynamicLimits {
                            max_program_headers: 2,
                            max_dynamic_entries: 32,
                        },
                        hash: HashLimits { max_words: 64 },
                    },
                    symbols: SymbolObservationLimits {
                        max_descriptors: 2,
                        max_symbols: 3,
                        max_name_lookups: 16,
                        max_name_scan_bytes: 64,
                        max_total_name_scan_bytes: 256,
                    },
                    max_relocations: 4,
                },
            )),
            IdentityLimits {
                max_identity_records: 16,
            },
        );
        let plan = plan(
            Arc::new(evidence),
            vec![],
            VirtualAddress(0x220000000),
            PlanningLimits {
                max_providers: 1,
                max_plan_records: 256,
            },
        )
        .unwrap();
        let (staged, _) = crate::input::staging::stage(
            Arc::new(plan),
            crate::input::staging::StagingLimits {
                max_mapped_bytes: 65536,
            },
        );
        let (native, _) = crate::input::native::realize(
            &staged.unwrap(),
            crate::input::native::NativeLimits {
                max_reserved_bytes: 65536,
                max_committed_bytes: 65536,
            },
        );
        let (guest, _) = prepare(
            native.unwrap(),
            RuntimeLimits {
                stack_base: 0x230000000,
                stack_bytes: 0x4000,
                tls_base: 0x231000000,
                max_runtime_bytes: 0x20000,
            },
            PreparedRegistry::new(vec![], 0).unwrap(),
            None,
        )
        .unwrap();
        bridge.prepare_ranges(&[(0x220001000, 4096)]).unwrap();
        let engine =
            astero_timing::scheduler::TimingEngine::real(astero_timing::scheduler::Config {
                max_pending: 128,
                max_snapshot_entries: 128,
            })
            .unwrap();
        let sync = Arc::new(Synchronization::new(engine.scheduler(), 16, 16).unwrap());
        let (_, startup) = astero_libs::libc::startup::owned_registrations(
            16,
            vec![(0x220001000, 4096)],
            Some(bridge.return_landing()),
        );
        let foundation = Arc::new(
            super::super::foundation::Foundation::build(0x232000000, 4096, 0x500000).unwrap(),
        );
        let runtime = Runtime::new(
            &guest,
            foundation,
            sync.clone(),
            engine.scheduler(),
            startup,
            &bridge,
            vec![Some(ProviderKey {
                nid,
                library: b"libkernel".to_vec(),
                module: b"libkernel".to_vec(),
            })],
        )
        .unwrap();
        let deadline = astero_timing::time::Deadline::after(
            engine.scheduler().now().unwrap(),
            astero_timing::time::Span::from_millis(500).unwrap(),
        )
        .unwrap();
        sync.arm(deadline).unwrap();
        runtime.table.arm(deadline);
        (guest, bridge, runtime, engine)
    }
    #[test]
    fn composed_workers_return_and_exit_through_real_provider_registry() {
        let _serial = SERIAL.lock().unwrap();
        for (code, nid) in [
            (vec![0x48, 0x89, 0xf8, 0xc3], 0),
            (vec![], 0xde483bad3d0d408b),
        ] {
            let (g, _b, r, _e) = fixture(&code, nid);
            let address = g.thread.layout().stack.start.0;
            r.create(
                Attributes::default(),
                0x220001100,
                77,
                vec![],
                address,
                &mut r.access(),
            )
            .unwrap();
            let id = Thread(u64::from_le_bytes(
                r.access().read(address, 8).unwrap().try_into().unwrap(),
            ));
            assert_eq!(r.table.join(Thread(1), id), Ok(77));
            r.stop_and_join();
            assert!(
                r.observers
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|o| o.active_reservations() == 0)
            );
            assert!(
                r.exits
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|(_, e)| e.host_fs_restored && e.host_gs_preserved)
            );
        }
    }
    #[test]
    fn composed_workers_use_distinct_errno_and_same_guest_identity_model() {
        let _serial = SERIAL.lock().unwrap();
        let (g, _b, r, _e) = fixture(&[], astero_libs::libc::process::ERROR_NID);
        let address = g.thread.layout().stack.start.0;
        let mut pointers = vec![];
        for n in 0..2 {
            r.create(
                Attributes::default(),
                0x220001100,
                0,
                vec![],
                address,
                &mut r.access(),
            )
            .unwrap();
            pointers.push(r.table.join(Thread(1), Thread(n + 2)).unwrap());
        }
        assert_ne!(pointers[0], pointers[1]);
        assert_ne!(pointers[0], g.thread.layout().thread_pointer + 8);
        r.stop_and_join();
        assert!(
            r.observers
                .lock()
                .unwrap()
                .iter()
                .all(|o| o.active_reservations() == 0)
        );
    }
    #[test]
    fn composed_worker_unknown_provider_stops_and_tears_down() {
        let _serial = SERIAL.lock().unwrap();
        let (g, _b, r, _e) = fixture(&[], 0xdeadbeef);
        r.create(
            Attributes::default(),
            0x220001100,
            0,
            vec![],
            g.thread.layout().stack.start.0,
            &mut r.access(),
        )
        .unwrap();
        let result = r.table.join(Thread(1), Thread(2));
        assert!(matches!(
            result,
            Err(Error::Fault) | Err(Error::Interrupted)
        ));
        r.stop_and_join();
        assert_eq!(r.exits.lock().unwrap()[0].1.reason, 2);
        assert!(
            r.observers
                .lock()
                .unwrap()
                .iter()
                .all(|o| o.active_reservations() == 0)
        );
    }
    #[test]
    fn composed_shutdown_interrupts_worker_in_m33_mutex_wait() {
        let _serial = SERIAL.lock().unwrap();
        let (g, _b, r, _e) = fixture(&[], 0xf542b5bcb6507ede);
        let address = g.thread.layout().stack.start.0;
        let object = r
            .synchronization
            .create(
                address,
                astero_kernel::synchronization::owned::Kind::Mutex,
                1,
            )
            .unwrap();
        r.access().write(address, &object.to_le_bytes()).unwrap();
        r.synchronization
            .mutex_lock(object, address, Thread(1), false, None)
            .unwrap();
        r.create(
            Attributes::default(),
            0x220001100,
            address,
            vec![],
            address + 16,
            &mut r.access(),
        )
        .unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while r.synchronization.snapshot().waiters == 0 {
            assert!(std::time::Instant::now() < end);
            std::thread::yield_now();
        }
        r.stop_and_join();
        assert_eq!(r.synchronization.snapshot().waiters, 0);
        assert_eq!(
            r.table.snapshot()[1].outcome.as_ref().unwrap().reason,
            ThreadEnd::Interrupted
        );
        assert_eq!(
            r.calls.lock().unwrap().last().unwrap().1.result,
            Ok(astero_hle::dispatch::prepared::CallResult::StopRequested)
        );
        assert!(
            r.observers
                .lock()
                .unwrap()
                .iter()
                .all(|o| o.active_reservations() == 0)
        );
    }
    #[test]
    fn worker_creation_refuses_nonexecutables_and_failed_output_without_worker() {
        let _serial = SERIAL.lock().unwrap();
        let (g, _b, r, _e) = fixture(&[0xc3], 0);
        for (start, out) in [
            (g.thread.layout().rsp, g.thread.layout().stack.start.0),
            (0x220001100, 1),
        ] {
            assert_eq!(
                r.create(
                    Attributes::default(),
                    start,
                    0,
                    vec![],
                    out,
                    &mut r.access()
                ),
                Err(Error::Invalid)
            );
        }
        assert_eq!(r.table.snapshot().len(), 1);
        r.stop_and_join();
        assert!(r.observers.lock().unwrap().is_empty());
    }
    #[test]
    fn supervised_native_workers_perform_large_checked_memset_on_shared_heap() {
        let _serial = SERIAL.lock().unwrap();
        let code = [0xbe, 42, 0, 0, 0, 0xba, 0x40, 0xa1, 0x14, 0];
        let (g, _bridge, r, _engine) = fixture_impl(&code, 0xf334c5bc120020df, true);
        let slot = g.thread.layout().stack.start.0;
        let mut blocks = vec![];
        for i in 0..2 {
            let p = r.access().allocate(1_352_000).unwrap();
            blocks.push(p);
            r.create(
                Attributes::default(),
                0x220001100,
                p,
                vec![],
                slot + i * 8,
                &mut r.access(),
            )
            .unwrap();
        }
        for (i, p) in blocks.iter().enumerate() {
            assert_eq!(r.table.join(Thread(1), Thread(i as u64 + 2)), Ok(*p));
            let mut at = 0;
            while at < 1_352_000 {
                let n = (1_352_000 - at).min(65536);
                assert!(
                    r.access()
                        .read(*p + at, n)
                        .unwrap()
                        .iter()
                        .all(|b| *b == 42)
                );
                at += n;
            }
            r.access().free(*p).unwrap();
        }
        r.stop_and_join();
        assert_eq!(r.foundation.access_budget.snapshot().large_operations, 2);
        assert_eq!(r.foundation.heap.lock().unwrap().snapshot().live, 0);
        assert!(
            r.observers
                .lock()
                .unwrap()
                .iter()
                .all(|o| o.active_reservations() == 0)
        );
    }
}
