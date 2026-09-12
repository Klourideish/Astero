use astero_kernel::synchronization::{owned::Thread, semaphore::*};
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Span, Tick},
};
use std::sync::Arc;
fn setup() -> (
    TimingEngine,
    astero_timing::clock::ManualClock,
    Arc<Semaphores>,
) {
    let (e, c) = TimingEngine::manual(Config {
        max_pending: 16,
        max_snapshot_entries: 16,
    })
    .unwrap();
    let s = Arc::new(Semaphores::new(e.scheduler(), 4, 8).unwrap());
    (e, c, s)
}
fn pending(s: &Semaphores, n: usize) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while s.snapshot().waiting_threads.len() != n {
        assert!(std::time::Instant::now() < end);
        std::thread::yield_now();
    }
}
#[test]
fn counts_poll_and_overflow() {
    let (_e, _c, s) = setup();
    let h = s.create(b"s", 2, 3).unwrap();
    s.poll(h, 2).unwrap();
    assert_eq!(s.poll(h, 1), Err(Error::Busy));
    s.signal(h, 3).unwrap();
    assert_eq!(s.signal(h, 1), Err(Error::Overflow));
    assert_eq!(s.snapshot().objects[0].count, 3);
}
#[test]
fn handle_lifetime_and_capacity() {
    let (_e, _c, s) = setup();
    let h = s.create(b"same", 0, 1).unwrap();
    s.delete(h).unwrap();
    assert_eq!(s.delete(h), Err(Error::NotFound));
    for _ in 0..4 {
        assert_ne!(s.create(b"same", 0, 1).unwrap(), h);
    }
    assert_eq!(s.create(b"s", 0, 1), Err(Error::Capacity));
    assert_eq!(s.poll(h, 1), Err(Error::NotFound));
}
#[test]
fn invalid_counts_and_name() {
    let (_e, _c, s) = setup();
    assert_eq!(s.create(&[b'x'; 32], 0, 1), Err(Error::Invalid));
    assert_eq!(s.create(b"s", 2, 1), Err(Error::Invalid));
    let h = s.create(b"s", 0, 1).unwrap();
    assert_eq!(s.signal(h, 0), Err(Error::Invalid));
    assert_eq!(s.poll(h, -1), Err(Error::Invalid));
}
#[test]
fn fifo_counted_waits() {
    let (e, _c, s) = setup();
    let h = s.create(b"s", 0, 3).unwrap();
    let a = s.clone();
    let w1 = std::thread::spawn(move || a.wait(h, 2, Thread(1), None));
    pending(&s, 1);
    let a = s.clone();
    let w2 = std::thread::spawn(move || a.wait(h, 1, Thread(2), None));
    pending(&s, 2);
    s.signal(h, 1).unwrap();
    assert_eq!(s.poll(h, 1), Err(Error::Busy));
    s.signal(h, 2).unwrap();
    assert_eq!(w1.join().unwrap(), Ok(()));
    assert_eq!(w2.join().unwrap(), Ok(()));
    assert_eq!(s.snapshot().objects[0].count, 0);
    assert_eq!(e.scheduler().snapshot(16).unwrap().pending_total, 0);
}
#[test]
fn timeout_uses_manual_clock() {
    let (e, c, s) = setup();
    let h = s.create(b"s", 0, 1).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || {
        a.wait(h, 1, Thread(3), Some(Deadline::at(Tick::from_nanos(5))))
    });
    pending(&s, 1);
    c.advance(Span::from_nanos(5)).unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Timeout));
    assert_eq!(s.snapshot().timeouts, 1);
    assert_eq!(e.scheduler().snapshot(16).unwrap().pending_total, 0);
}
#[test]
fn cancellation_is_not_success() {
    let (_e, _c, s) = setup();
    let h = s.create(b"s", 0, 2).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || a.wait(h, 1, Thread(3), None));
    pending(&s, 1);
    assert_eq!(s.cancel(h, 2), Ok(1));
    assert_eq!(w.join().unwrap(), Err(Error::Cancelled));
    assert_eq!(s.snapshot().objects[0].count, 2);
}
#[test]
fn deletion_releases_waiters() {
    let (e, _c, s) = setup();
    let h = s.create(b"s", 0, 1).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || a.wait(h, 1, Thread(3), None));
    pending(&s, 1);
    s.delete(h).unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Deleted));
    assert_eq!(e.scheduler().snapshot(16).unwrap().pending_total, 0);
}
#[test]
fn shutdown_joins_blocked_caller() {
    let (e, _c, s) = setup();
    let h = s.create(b"s", 0, 1).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || a.wait(h, 1, Thread(3), None));
    pending(&s, 1);
    s.shutdown();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
    assert!(s.snapshot().objects.is_empty());
    assert!(s.snapshot().waiting_threads.is_empty());
    assert_eq!(e.scheduler().snapshot(16).unwrap().pending_total, 0);
}
#[test]
fn supervisor_deadline_interrupts_infinite_wait() {
    let (_e, c, s) = setup();
    s.arm(Deadline::at(Tick::from_nanos(7))).unwrap();
    let h = s.create(b"s", 0, 1).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || a.wait(h, 1, Thread(3), None));
    pending(&s, 1);
    c.advance(Span::from_nanos(7)).unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
}
#[test]
fn scheduler_shutdown_not_success() {
    let (e, _c, s) = setup();
    let h = s.create(b"s", 0, 1).unwrap();
    let a = s.clone();
    let w = std::thread::spawn(move || a.wait(h, 1, Thread(3), None));
    pending(&s, 1);
    e.shutdown().unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
}
