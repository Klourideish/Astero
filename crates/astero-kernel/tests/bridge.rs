#![cfg(all(windows, target_arch = "x86_64"))]
use astero_kernel::execution::host::{Bridge, BridgeError, SyntheticProbe};
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn worker_image(
    bytes: &[u8],
) -> std::sync::Arc<astero_memory::mapping::windows_native::NativeImage> {
    use astero_memory::mapping::{GuestAddress, windows_native::*};
    let mut code = vec![0xcc; 4096];
    code[..bytes.len()].copy_from_slice(bytes);
    std::sync::Arc::new(
        realize(
            &[NativeRegion {
                range: GuestRange {
                    start: GuestAddress(0x740000000),
                    size: 4096,
                },
                bytes: &code,
                protection: Protection {
                    read: true,
                    write: false,
                    execute: true,
                },
            }],
            NativeLimits {
                max_reserved_bytes: 4096,
                max_committed_bytes: 4096,
            },
        )
        .0
        .unwrap(),
    )
}
fn worker_storage(
    n: u64,
    landing: u64,
) -> astero_kernel::execution::preparation::storage::ThreadStorage {
    use astero_kernel::execution::preparation::{layout::*, storage::ThreadStorage};
    let base = 0x750000000 + n * 0x100000;
    let l = plan(
        RuntimeLimits {
            stack_base: base,
            stack_bytes: 0x4000,
            tls_base: base + 0x10000,
            max_runtime_bytes: 0x20000,
        },
        astero_memory::mapping::windows_native::host_geometry().unwrap(),
        8,
        32,
        16,
    )
    .unwrap();
    let (s, _) = ThreadStorage::build(l, &[0xa5; 8]).unwrap();
    assert_eq!(s.read_tls(s.layout().tls.start.0, 8).unwrap(), [0xa5; 8]);
    assert_eq!(s.read_tls(s.layout().tls.start.0 + 8, 24).unwrap(), [0; 24]);
    s.install_return(landing).unwrap();
    s
}
#[test]
fn attached_workers_share_adapter_but_not_stack_tls_or_frames() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    // Astero-owned code stores and reads its argument through FS+8, then RET.
    let image = worker_image(&[
        0x64, 0x48, 0x89, 0x3c, 0x25, 8, 0, 0, 0, 0x64, 0x48, 0x8b, 4, 0x25, 8, 0, 0, 0, 0xc3,
    ]);
    b.prepare_ranges(&[(0x740000000, 4096)]).unwrap();
    let lease = b.worker_lease().unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut handles = vec![];
    let mut observers = vec![];
    for n in 0..2 {
        let s = worker_storage(n, b.return_landing());
        observers.extend(s.observers());
        let l = lease.clone();
        let i = image.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let mut b = l.attach();
            barrier.wait();
            let r = b
                .execute_worker(&i, &s, (0x740000000, 100 + n), 100, &mut |_, _| false)
                .unwrap();
            assert_eq!(r.value, 100 + n);
            assert_eq!(
                s.read_tls(s.layout().thread_pointer + 8, 8).unwrap(),
                (100 + n).to_le_bytes()
            );
            assert!(r.host_fs_restored && r.host_gs_preserved);
            r
        }));
    }
    drop(b);
    assert!(matches!(Bridge::new(), Err(BridgeError::Busy)));
    barrier.wait();
    for h in handles {
        assert_eq!(h.join().unwrap().reason, 0);
    }
    assert!(observers.iter().all(|o| o.active_reservations() == 0));
    drop(lease);
    assert!(Bridge::new().is_ok());
}
#[test]
fn attached_infinite_worker_remains_supervised_and_joinable() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    let image = worker_image(&[0xeb, 0xfe]);
    b.prepare_ranges(&[(0x740000000, 4096)]).unwrap();
    let l = b.worker_lease().unwrap();
    let s = worker_storage(0, b.return_landing());
    let observers = s.observers();
    let h = std::thread::spawn(move || {
        l.attach()
            .execute_worker(&image, &s, (0x740000000, 0), 25, &mut |_, _| false)
            .unwrap()
    });
    let r = h.join().unwrap();
    assert_eq!(r.reason, 4);
    assert_eq!(r.rip, 0x740000000);
    assert!(r.host_fs_restored && r.host_gs_preserved);
    let sup = r.supervision.unwrap();
    assert_eq!(sup.suspends, sup.resumes);
    assert!(sup.redirected);
    assert!(observers.iter().all(|o| o.active_reservations() == 0));
}
#[test]
fn attached_worker_hle_exit_and_fault_use_existing_recovery() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    for code in [
        astero_kernel::execution::preparation::boundary::import_stub(17, b.import_landing())
            .unwrap()
            .to_vec(),
        vec![0x0f, 0x0b],
    ] {
        let image = worker_image(&code);
        b.prepare_ranges(&[(0x740000000, 4096)]).unwrap();
        let l = b.worker_lease().unwrap();
        let s = worker_storage(0, b.return_landing());
        let h = std::thread::spawn(move || {
            l.attach()
                .execute_worker(&image, &s, (0x740000000, 123), 100, &mut |n, c| {
                    assert_eq!(n, 17);
                    assert_eq!(c.arguments[0], 123);
                    c.rax = 321;
                    false
                })
                .unwrap()
        });
        let r = h.join().unwrap();
        assert!(matches!(r.reason, 2 | 3));
        assert!(r.host_fs_restored && r.host_gs_preserved);
    }
}
#[test]
fn windows_register_stack_fs_and_fault_paths() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    assert!(b.validated());
    for (p, code) in [
        (SyntheticProbe::IllegalInstruction, 0xc000001d),
        (SyntheticProbe::AccessViolation, 0xc0000005),
    ] {
        let r = b.synthetic(p, &mut |_, _| false).unwrap();
        assert_eq!(r.exception, code);
        assert_ne!(r.rip, 0);
        assert_eq!(r.registers[4], r.rsp);
        assert!(r.host_fs_restored && r.host_gs_preserved);
    }
}
#[test]
fn import_ordinal_return_lanes_and_unresolved_stop() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    let r = b
        .synthetic(SyntheticProbe::ImportOrdinal(37), &mut |i, c| {
            assert_eq!(i, 37);
            assert_eq!(c.arguments, [7, 9, 0, 0, 0, 0]);
            c.rax = 123;
            true
        })
        .unwrap();
    assert_eq!(r.value, 123);
    assert_eq!(r.reason, 0);
    assert_eq!(
        b.synthetic(SyntheticProbe::Import, &mut |_, _| false)
            .unwrap()
            .reason,
        2
    );
}
#[test]
fn panicking_provider_returns_control_and_adapter_remains_usable() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    assert_eq!(
        b.synthetic(SyntheticProbe::Import, &mut |_, _| panic!(
            "synthetic panic"
        ))
        .unwrap()
        .reason,
        2
    );
    assert_eq!(
        b.synthetic(SyntheticProbe::Return, &mut |_, _| false)
            .unwrap()
            .value,
        16
    );
}
#[test]
fn exclusive_adapter_and_deterministic_handler_teardown() {
    let _guard = SERIAL.lock().unwrap();
    let b = Bridge::new().unwrap();
    assert!(matches!(Bridge::new(), Err(BridgeError::Busy)));
    assert!(
        std::thread::spawn(|| matches!(Bridge::new(), Err(BridgeError::Busy)))
            .join()
            .unwrap()
    );
    drop(b);
    let mut next = Bridge::new().unwrap();
    next.validate().unwrap();
}
#[test]
fn range_metadata_is_checked_without_granting_transfer() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    assert!(b.prepare_ranges(&[(u64::MAX, 2)]).is_err());
    b.prepare_ranges(&[(0x1000, 4096)]).unwrap();
    assert_eq!(b.prepared_ranges(), 1);
    assert!(!b.validated());
}

#[test]
fn supervisor_interrupts_loop_and_joins_exact_thread() {
    let _guard = SERIAL.lock().unwrap();
    let worker = std::thread::spawn(|| {
        let mut b = Bridge::new().unwrap();
        let r = b
            .supervised_synthetic(SyntheticProbe::InfiniteLoop, 20, &mut |_, _| false)
            .unwrap();
        assert_eq!(r.reason, 4);
        let s = r.supervision.as_ref().unwrap();
        assert!(s.redirected && Bridge::synthetic_loop_contains(s.rip));
        assert_eq!(s.suspends, s.resumes);
        assert!(r.host_fs_restored && r.host_gs_preserved);
        r
    });
    assert_eq!(worker.join().unwrap().reason, 4);
    assert!(Bridge::new().is_ok());
}
#[test]
fn supervised_boundaries_cancel_deadline_and_preserve_host() {
    let _guard = SERIAL.lock().unwrap();
    std::thread::spawn(|| {
        let mut b = Bridge::new().unwrap();
        for (p, reason) in [
            (SyntheticProbe::Return, 0),
            (SyntheticProbe::Import, 2),
            (SyntheticProbe::IllegalInstruction, 3),
            (SyntheticProbe::AccessViolation, 3),
            (SyntheticProbe::GuardRead, 3),
        ] {
            let r = b.supervised_synthetic(p, 100, &mut |_, _| false).unwrap();
            assert_eq!(r.reason, reason);
            assert!(r.host_fs_restored && r.host_gs_preserved);
            let s = r.supervision.unwrap();
            assert_eq!(s.suspends, s.resumes);
        }
        let r = b
            .supervised_synthetic(SyntheticProbe::Import, 100, &mut |_, c| {
                c.rax = 42;
                true
            })
            .unwrap();
        assert_eq!(r.value, 42);
    })
    .join()
    .unwrap();
}
#[test]
fn unbounded_loop_and_invalid_deadline_are_refused() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    assert!(
        b.synthetic(SyntheticProbe::InfiniteLoop, &mut |_, _| false)
            .is_err()
    );
    for ms in [0, 501, u64::MAX] {
        assert!(
            b.supervised_synthetic(SyntheticProbe::InfiniteLoop, ms, &mut |_, _| false)
                .is_err()
        );
    }
}

#[test]
fn blocked_native_hle_wait_uses_timing_and_returns_to_joined_host() {
    let _guard = SERIAL.lock().unwrap();
    std::thread::spawn(|| {
        use astero_kernel::synchronization::owned::*;
        use astero_timing::{
            scheduler::{Config, TimingEngine},
            time::{Deadline, Span},
        };
        let engine = TimingEngine::real(Config {
            max_pending: 4,
            max_snapshot_entries: 4,
        })
        .unwrap();
        let sync = Synchronization::new(engine.scheduler(), 4, 4).unwrap();
        let id = sync.create(8, Kind::Mutex, 1).unwrap();
        sync.mutex_lock(id, 8, Thread(1), false, None).unwrap();
        let mut bridge = Bridge::new().unwrap();
        bridge.validate().unwrap();
        sync.arm(
            Deadline::after(
                engine.scheduler().now().unwrap(),
                Span::from_millis(10).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let exit = bridge
            .supervised_synthetic(SyntheticProbe::Import, 100, &mut |_, _| {
                assert_eq!(
                    sync.mutex_lock(id, 8, Thread(2), false, None),
                    Err(Error::Interrupted)
                );
                false
            })
            .unwrap();
        assert_eq!(exit.reason, 2);
        assert!(exit.host_fs_restored && exit.host_gs_preserved);
        let sup = exit.supervision.unwrap();
        assert_eq!(sup.suspends, sup.resumes);
        assert_eq!(sync.snapshot().waiters, 0);
        assert_eq!(engine.scheduler().snapshot(4).unwrap().pending_total, 0);
    })
    .join()
    .unwrap();
}

#[test]
fn sampled_infinite_worker_captures_actual_guest_pc_and_balances_resume() {
    use astero_kernel::execution::host::sampling::{Domain, Range, Sampler};
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    let image = worker_image(&[0xeb, 0xfe]);
    b.prepare_ranges(&[(0x740000000, 4096)]).unwrap();
    let sampler = std::sync::Arc::new(
        Sampler::new(
            5,
            64,
            vec![Range {
                start: 0x740000000,
                size: 4096,
                domain: Domain::GuestImage,
                source: 0,
            }],
        )
        .unwrap(),
    );
    b.observe_pc(sampler.clone()).unwrap();
    let l = b.worker_lease().unwrap();
    let s = worker_storage(0, b.return_landing());
    let observers = s.observers();
    let r = std::thread::spawn(move || {
        l.attach()
            .execute_worker(&image, &s, (0x740000000, 0), 60, &mut |_, _| false)
            .unwrap()
    })
    .join()
    .unwrap();
    assert_eq!(r.reason, 4);
    assert!(r.host_fs_restored && r.host_gs_preserved);
    let sample = sampler.snapshot();
    assert!(sample.total > 0);
    assert_eq!(sample.suspends, sample.resumes);
    assert!(
        sample
            .samples
            .iter()
            .any(|s| s.rip == 0x740000000 && s.domain == Domain::GuestImage)
    );
    assert!(observers.iter().all(|o| o.active_reservations() == 0));
}
#[test]
fn sampling_preserves_return_import_and_fault_stop_paths() {
    use astero_kernel::execution::host::sampling::Sampler;
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    b.observe_pc(std::sync::Arc::new(Sampler::new(5, 64, vec![]).unwrap()))
        .unwrap();
    for p in [
        SyntheticProbe::Return,
        SyntheticProbe::Import,
        SyntheticProbe::AccessViolation,
        SyntheticProbe::IllegalInstruction,
    ] {
        let r = b.supervised_synthetic(p, 100, &mut |_, _| false).unwrap();
        assert!(r.host_fs_restored && r.host_gs_preserved);
        let s = r.supervision.unwrap();
        assert_eq!(s.suspends, s.resumes);
        assert!(!s.redirected);
    }
}
