use astero_kernel::synchronization::owned::*;
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Span, Tick},
};
use std::sync::Arc;
fn setup() -> (
    TimingEngine,
    astero_timing::clock::ManualClock,
    Arc<Synchronization>,
) {
    let (e, c) = TimingEngine::manual(Config {
        max_pending: 32,
        max_snapshot_entries: 32,
    })
    .unwrap();
    let s = Arc::new(Synchronization::new(e.scheduler(), 16, 16).unwrap());
    s.arm(Deadline::at(Tick::from_nanos(1000))).unwrap();
    (e, c, s)
}
fn waiters(s: &Synchronization, n: usize) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while s.snapshot().waiters != n {
        assert!(std::time::Instant::now() < until, "waiter registration");
        std::thread::yield_now();
    }
}
#[test]
fn rw_readers_exclude_writer_and_destroy() {
    let (_e, _c, s) = setup();
    let id = s.create(8, Kind::Rwlock, 1).unwrap();
    for t in [Thread(1), Thread(2)] {
        s.rw_lock(id, 8, t, false, false, None).unwrap();
    }
    assert_eq!(
        s.rw_lock(id, 8, Thread(3), true, true, None),
        Err(Error::Busy)
    );
    assert_eq!(s.destroy(id, 8, Kind::Rwlock), Err(Error::Busy));
    s.rw_unlock(id, 8, Thread(1)).unwrap();
    s.rw_unlock(id, 8, Thread(2)).unwrap();
    s.destroy(id, 8, Kind::Rwlock).unwrap();
}
#[test]
fn rw_writer_exclusion_and_ownership() {
    let (_e, _c, s) = setup();
    let id = s.create(8, Kind::Rwlock, 1).unwrap();
    s.rw_lock(id, 8, Thread(1), true, false, None).unwrap();
    assert_eq!(
        s.rw_lock(id, 8, Thread(2), false, true, None),
        Err(Error::Busy)
    );
    assert_eq!(s.rw_unlock(id, 8, Thread(2)), Err(Error::Permission));
    assert_eq!(
        s.rw_lock(id, 8, Thread(1), false, true, None),
        Err(Error::Deadlock)
    );
}
#[test]
fn rw_wait_wakes_on_unlock() {
    let (e, _c, s) = setup();
    let id = s.create(8, Kind::Rwlock, 1).unwrap();
    s.rw_lock(id, 8, Thread(1), true, false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.rw_lock(id, 8, Thread(2), false, false, None));
    waiters(&s, 1);
    s.rw_unlock(id, 8, Thread(1)).unwrap();
    h.join().unwrap().unwrap();
    assert_eq!(e.scheduler().snapshot(32).unwrap().pending_total, 0);
}
#[test]
fn mutex_recursive_and_errorcheck() {
    let (_e, _c, s) = setup();
    let id = s.create(8, Kind::Mutex, 2).unwrap();
    s.mutex_lock(id, 8, Thread(1), false, None).unwrap();
    s.mutex_lock(id, 8, Thread(1), true, None).unwrap();
    s.mutex_unlock(id, 8, Thread(1)).unwrap();
    assert_eq!(s.mutex_lock(id, 8, Thread(2), true, None), Err(Error::Busy));
    s.mutex_unlock(id, 8, Thread(1)).unwrap();
    s.destroy(id, 8, Kind::Mutex).unwrap();
    let id = s.create(8, Kind::Mutex, 1).unwrap();
    s.mutex_lock(id, 8, Thread(1), false, None).unwrap();
    assert_eq!(
        s.mutex_lock(id, 8, Thread(1), false, None),
        Err(Error::Deadlock)
    );
}
#[test]
fn mutex_wrong_owner_and_busy_destroy() {
    let (_e, _c, s) = setup();
    let id = s.create(8, Kind::Mutex, 1).unwrap();
    s.mutex_lock(id, 8, Thread(1), false, None).unwrap();
    assert_eq!(s.mutex_unlock(id, 8, Thread(2)), Err(Error::Permission));
    assert_eq!(s.destroy(id, 8, Kind::Mutex), Err(Error::Busy));
}
#[test]
fn copied_mutex_tokens_remain_valid_but_stale_or_wrong_kind_refused() {
    let (_e, _c, s) = setup();
    let id = s.create(8, Kind::Mutex, 1).unwrap();
    assert_eq!(s.validate(id, 16, Kind::Mutex), Ok(()));
    assert_eq!(s.validate(id, 8, Kind::Cond), Err(Error::Invalid));
    s.destroy(id, 8, Kind::Mutex).unwrap();
    let newer = s.create(8, Kind::Mutex, 1).unwrap();
    assert_ne!(id, newer);
    assert_eq!(s.validate(id, 8, Kind::Mutex), Err(Error::Invalid));
}
#[test]
fn exact_object_capacity() {
    let (e, _c, _s) = setup();
    let s = Synchronization::new(e.scheduler(), 1, 1).unwrap();
    s.create(8, Kind::Cond, 0).unwrap();
    assert_eq!(s.create(16, Kind::Cond, 0), Err(Error::Capacity));
}
#[test]
fn mutex_timeout_cleans_ticket() {
    let (e, c, s) = setup();
    let id = s.create(8, Kind::Mutex, 1).unwrap();
    s.mutex_lock(id, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || {
        w.mutex_lock(
            id,
            8,
            Thread(2),
            false,
            Some(Deadline::at(Tick::from_nanos(10))),
        )
    });
    waiters(&s, 1);
    c.advance(Span::from_nanos(10)).unwrap();
    assert_eq!(h.join().unwrap(), Err(Error::Timeout));
    assert_eq!(s.snapshot().waiters, 0);
    assert_eq!(e.scheduler().snapshot(32).unwrap().pending_total, 0);
}
#[test]
fn shutdown_interrupts_blocked_mutex() {
    let (e, _c, s) = setup();
    let id = s.create(8, Kind::Mutex, 3).unwrap();
    s.mutex_lock(id, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.mutex_lock(id, 8, Thread(1), false, None));
    waiters(&s, 1);
    s.shutdown();
    assert_eq!(h.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(e.scheduler().snapshot(32).unwrap().pending_total, 0);
}
#[test]
fn execution_deadline_interrupts_indefinite_wait() {
    let (_e, c, s) = setup();
    let id = s.create(8, Kind::Rwlock, 1).unwrap();
    s.rw_lock(id, 8, Thread(1), true, false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.rw_lock(id, 8, Thread(2), false, false, None));
    waiters(&s, 1);
    c.advance(Span::from_nanos(1000)).unwrap();
    assert_eq!(h.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(s.snapshot().waiters, 0);
}
#[test]
fn cond_signal_atomic_release_reacquire() {
    let (_e, _c, s) = setup();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.cond_wait((c, 16), (m, 8), Thread(1), None));
    waiters(&s, 1);
    s.mutex_lock(m, 8, Thread(2), true, None).unwrap();
    s.cond_signal(c, 16, false).unwrap();
    assert_eq!(s.destroy(c, 16, Kind::Cond), Err(Error::Busy));
    s.mutex_unlock(m, 8, Thread(2)).unwrap();
    h.join().unwrap().unwrap();
    assert_eq!(s.snapshot().objects[0].owner, Some(Thread(1)));
}
#[test]
fn cond_timeout_reacquires_mutex() {
    let (e, clock, s) = setup();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || {
        w.cond_wait(
            (c, 16),
            (m, 8),
            Thread(1),
            Some(Deadline::at(Tick::from_nanos(5))),
        )
    });
    waiters(&s, 1);
    clock.advance(Span::from_nanos(5)).unwrap();
    assert_eq!(h.join().unwrap(), Err(Error::Timeout));
    s.mutex_unlock(m, 8, Thread(1)).unwrap();
    assert_eq!(e.scheduler().snapshot(32).unwrap().pending_total, 0);
}
#[test]
fn cond_broadcast_releases_all() {
    let (_e, _c, s) = setup();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    let mut threads = Vec::new();
    for n in 1..=3 {
        s.mutex_lock(m, 8, Thread(n), false, None).unwrap();
        let w = s.clone();
        threads.push(std::thread::spawn(move || {
            w.cond_wait((c, 16), (m, 8), Thread(n), None).unwrap();
            w.mutex_unlock(m, 8, Thread(n)).unwrap();
        }));
        waiters(&s, n as usize);
    }
    s.cond_signal(c, 16, true).unwrap();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(s.snapshot().waiters, 0);
}
#[test]
fn cond_shutdown_never_resumes_guest_with_unowned_mutex() {
    let (_e, _c, s) = setup();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.cond_wait((c, 16), (m, 8), Thread(1), None));
    waiters(&s, 1);
    s.shutdown();
    assert_eq!(h.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(s.snapshot().waiters, 0);
}
#[test]
fn cond_requires_single_owned_depth() {
    let (_e, _c, s) = setup();
    let m = s.create(8, Kind::Mutex, 2).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    assert_eq!(
        s.cond_wait((c, 16), (m, 8), Thread(1), None),
        Err(Error::Permission)
    );
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    assert_eq!(
        s.cond_wait((c, 16), (m, 8), Thread(1), None),
        Err(Error::Deadlock)
    );
}
#[test]
fn clock_conversion_rejects_invalid_and_checks_exact_monotonic() {
    let (_e, _c, s) = setup();
    assert_eq!(s.absolute(0, 10, 4).unwrap().tick(), Tick::from_nanos(10));
    assert_eq!(s.absolute(0, 1_000_000_000, 0), Err(Error::Invalid));
    assert_eq!(s.absolute(i64::MAX, 0, 4), Err(Error::Invalid));
}

#[test]
fn waiter_capacity_refuses_without_releasing_or_leaking() {
    let (e, _c, _s) = setup();
    let s = Arc::new(Synchronization::new(e.scheduler(), 4, 1).unwrap());
    s.arm(Deadline::at(Tick::from_nanos(1000))).unwrap();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.mutex_lock(m, 8, Thread(2), false, None));
    waiters(&s, 1);
    assert_eq!(
        s.mutex_lock(m, 8, Thread(3), false, None),
        Err(Error::Capacity)
    );
    s.shutdown();
    assert_eq!(h.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(s.snapshot().waiters, 0);
}
#[test]
fn runtime_owner_drop_releases_object_storage() {
    let (_e, _c, s) = setup();
    s.create(8, Kind::Mutex, 1).unwrap();
    let weak = Arc::downgrade(&s);
    drop(s);
    assert!(weak.upgrade().is_none());
}

#[test]
fn timing_shutdown_is_not_condition_signal_success() {
    let (engine, _clock, s) = setup();
    let m = s.create(8, Kind::Mutex, 1).unwrap();
    let c = s.create(16, Kind::Cond, 0).unwrap();
    s.mutex_lock(m, 8, Thread(1), false, None).unwrap();
    let w = s.clone();
    let h = std::thread::spawn(move || w.cond_wait((c, 16), (m, 8), Thread(1), None));
    waiters(&s, 1);
    engine.shutdown().unwrap();
    assert_eq!(h.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(s.snapshot().waiters, 0);
}
