use astero_kernel::threading::thread::{attributes::*, lifecycle::*};
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Span},
};
use std::sync::{Arc, mpsc};
fn setup(cap: usize) -> (TimingEngine, Arc<ThreadTable>) {
    let e = TimingEngine::real(Config {
        max_pending: 16,
        max_snapshot_entries: 16,
    })
    .unwrap();
    let t = Arc::new(ThreadTable::new(e.scheduler(), cap));
    t.arm(
        Deadline::after(
            e.scheduler().now().unwrap(),
            Span::from_millis(500).unwrap(),
        )
        .unwrap(),
    );
    (e, t)
}
fn reserve(t: &ThreadTable, detached: bool) -> Thread {
    t.reserve(
        Attributes {
            detached,
            ..Default::default()
        },
        0x1000,
        7,
        b"worker".to_vec(),
    )
    .unwrap()
}
#[test]
fn attributes_have_exact_owned_slot_and_stale_rejection() {
    let t = AttributeTable::new(1);
    let a = t.create(8).unwrap();
    assert_eq!(t.get(a, 8).unwrap().stack_size, 0x200000);
    assert_eq!(t.get(a, 16), Err(Error::Invalid));
    assert_eq!(t.create(16), Err(Error::Capacity));
    t.destroy(a, 8).unwrap();
    let b = t.create(8).unwrap();
    assert_ne!(a, b);
    assert_eq!(t.get(a, 8), Err(Error::Invalid));
}
#[test]
fn attributes_refuse_unsupported_placement_and_host_scheduling() {
    let t = AttributeTable::new(2);
    let id = t.create(8).unwrap();
    for a in [
        Attributes {
            stack_address: 0x10000,
            ..Default::default()
        },
        Attributes {
            priority: 400,
            ..Default::default()
        },
        Attributes {
            policy: 99,
            ..Default::default()
        },
    ] {
        assert_eq!(t.set(id, 8, a), Err(Error::Unsupported));
    }
    assert_eq!(t.get(id, 8).unwrap(), Attributes::default());
}
#[test]
fn stack_bounds_and_attribute_updates() {
    let t = AttributeTable::new(1);
    let id = t.create(8).unwrap();
    for n in [0, 1, 0x1001, 0x900000] {
        assert_eq!(
            t.set(
                id,
                8,
                Attributes {
                    stack_size: n,
                    ..Default::default()
                }
            ),
            Err(Error::Invalid)
        );
    }
    t.set(
        id,
        8,
        Attributes {
            stack_size: 0x4000,
            detached: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(t.get(id, 8).unwrap().detached);
}
#[test]
fn create_return_join_preserves_value_and_identity() {
    let (_e, t) = setup(2);
    let id = reserve(&t, false);
    assert_eq!(id, Thread(2));
    t.launch(
        id,
        || Outcome {
            value: 0x1234,
            reason: ThreadEnd::Returned,
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(t.join(Thread(1), id), Ok(0x1234));
    assert_eq!(t.join(Thread(1), id), Err(Error::Invalid));
    assert_eq!(t.snapshot()[1].state, State::Joined);
    t.reap_all();
}
#[test]
fn failed_publication_never_runs_job_and_joins_host() {
    let (_e, t) = setup(1);
    let id = reserve(&t, false);
    let entered = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let copy = entered.clone();
    assert_eq!(
        t.launch(
            id,
            move || {
                copy.store(true, std::sync::atomic::Ordering::SeqCst);
                Outcome {
                    value: 0,
                    reason: ThreadEnd::Returned,
                }
            },
            || Err(Error::Invalid)
        ),
        Err(Error::Invalid)
    );
    assert!(!entered.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(t.snapshot()[1].state, State::Failed);
    t.reap_all();
}
#[test]
fn detach_running_and_exited_retains_host_cleanup() {
    for detached in [false, true] {
        let (_e, t) = setup(1);
        let id = reserve(&t, detached);
        let (tx, rx) = mpsc::channel();
        t.launch(
            id,
            move || {
                rx.recv().unwrap();
                Outcome {
                    value: 5,
                    reason: ThreadEnd::Returned,
                }
            },
            || Ok(()),
        )
        .unwrap();
        if !detached {
            t.detach(id).unwrap();
        }
        assert_eq!(t.detach(id), Err(Error::Invalid));
        assert_eq!(t.join(Thread(1), id), Err(Error::Invalid));
        tx.send(()).unwrap();
        t.reap_all();
        assert_eq!(t.snapshot()[1].state, State::Reclaimed);
    }
}
#[test]
fn self_join_invalid_handles_and_capacity_are_distinct() {
    let (_e, t) = setup(1);
    let id = reserve(&t, false);
    assert_eq!(t.join(id, id), Err(Error::Deadlock));
    assert_eq!(t.join(Thread(1), Thread(99)), Err(Error::NoSuchThread));
    assert_eq!(
        t.reserve(Attributes::default(), 1, 0, vec![]),
        Err(Error::Capacity)
    );
    t.abandon(id);
    t.reap_all();
}
#[test]
fn faulted_thread_is_not_successful_join() {
    let (_e, t) = setup(1);
    let id = reserve(&t, false);
    t.launch(
        id,
        || Outcome {
            value: 99,
            reason: ThreadEnd::NativeFault,
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(t.join(Thread(1), id), Err(Error::Fault));
    t.reap_all();
}
#[test]
fn shutdown_interrupts_join_without_losing_host_ownership() {
    let (_e, t) = setup(1);
    let id = reserve(&t, false);
    let (tx, rx) = mpsc::channel();
    t.launch(
        id,
        move || {
            rx.recv().unwrap();
            Outcome {
                value: 0,
                reason: ThreadEnd::Returned,
            }
        },
        || Ok(()),
    )
    .unwrap();
    let copy = t.clone();
    let join = std::thread::spawn(move || copy.join(Thread(1), id));
    t.request_stop();
    tx.send(()).unwrap();
    let r = join.join().unwrap();
    assert!(matches!(r, Ok(0) | Err(Error::Interrupted)));
    t.reap_all();
    assert!(t.stopped());
}
#[test]
fn names_are_bounded_bytes_and_not_identity() {
    let (_e, t) = setup(1);
    let id = reserve(&t, false);
    t.rename(id, vec![0xff, 0xfe]).unwrap();
    assert_eq!(t.snapshot()[1].name, [0xff, 0xfe]);
    assert_eq!(t.rename(id, vec![0; 32]), Err(Error::Invalid));
    t.abandon(id);
    t.reap_all();
}
#[test]
fn thread_identity_integrates_with_mutex_ownership() {
    use astero_kernel::synchronization::owned::{Kind, Synchronization};
    let (e, t) = setup(1);
    let s = Arc::new(Synchronization::new(e.scheduler(), 4, 4).unwrap());
    s.arm(
        Deadline::after(
            e.scheduler().now().unwrap(),
            Span::from_millis(500).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let object = s.create(8, Kind::Mutex, 1).unwrap();
    let id = reserve(&t, false);
    let copy = s.clone();
    t.launch(
        id,
        move || {
            copy.mutex_lock(object, 8, id, false, None).unwrap();
            copy.mutex_unlock(object, 8, id).unwrap();
            Outcome {
                value: id.0,
                reason: ThreadEnd::Returned,
            }
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(t.join(Thread(1), id), Ok(2));
    s.destroy(object, 8, Kind::Mutex).unwrap();
    s.shutdown();
    t.reap_all();
}
#[test]
fn lifecycle_worker_cond_wait_signal_relocks_under_same_identity() {
    use astero_kernel::synchronization::owned::{Kind, Synchronization};
    let (e, t) = setup(1);
    let s = Arc::new(Synchronization::new(e.scheduler(), 4, 4).unwrap());
    s.arm(
        Deadline::after(
            e.scheduler().now().unwrap(),
            Span::from_millis(500).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mutex = s.create(8, Kind::Mutex, 1).unwrap();
    let cond = s.create(16, Kind::Cond, 0).unwrap();
    let id = reserve(&t, false);
    let copy = s.clone();
    t.launch(
        id,
        move || {
            copy.mutex_lock(mutex, 8, id, false, None).unwrap();
            copy.cond_wait((cond, 16), (mutex, 8), id, None).unwrap();
            copy.mutex_unlock(mutex, 8, id).unwrap();
            Outcome {
                value: 1,
                reason: ThreadEnd::Returned,
            }
        },
        || Ok(()),
    )
    .unwrap();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while s.snapshot().waiters == 0 {
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
    s.mutex_lock(mutex, 8, Thread(1), false, None).unwrap();
    s.cond_signal(cond, 16, false).unwrap();
    s.mutex_unlock(mutex, 8, Thread(1)).unwrap();
    assert_eq!(t.join(Thread(1), id), Ok(1));
    assert_eq!(s.snapshot().waiters, 0);
    s.shutdown();
    t.reap_all();
}
