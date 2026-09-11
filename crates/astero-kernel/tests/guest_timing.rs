use astero_abi::layouts::time::Timespec;
use astero_kernel::{
    synchronization::owned::{Synchronization, Thread},
    timing::{clock::*, sleep::*},
};
use astero_timing::{
    clock::ManualClock,
    scheduler::{Config, TimingEngine},
    time::{Deadline, Span, Tick},
};
use std::sync::Arc;
fn setup() -> (TimingEngine, ManualClock, Arc<GuestTiming>) {
    let (e, c) = TimingEngine::manual(Config {
        max_pending: 80,
        max_snapshot_entries: 80,
    })
    .unwrap();
    let t = Arc::new(GuestTiming::new(
        e.scheduler(),
        Realtime::Fixed(9_000_000_000),
    ));
    (e, c, t)
}
fn pending(t: &GuestTiming, n: usize) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while t.snapshot().pending != n {
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
}
#[test]
fn units_and_maximum() {
    assert_eq!(units(1000, 1000).unwrap().as_nanos(), 1_000_000);
    assert_eq!(units(1, 1_000_000).unwrap(), Span::from_millis(1).unwrap());
    assert_eq!(units(u64::MAX, 1).unwrap().as_nanos(), u64::MAX);
    assert_eq!(units(u64::MAX, 1000), Err(Error::Invalid));
    assert_eq!(units(0, 1000).unwrap(), Span::ZERO);
}
#[test]
fn timespec_layout_validation_and_normalization() {
    for n in [0, 999999999, 1000000000, u64::MAX] {
        let t = Timespec::from_nanos(n);
        assert_eq!(Timespec::decode(&t.encode()).unwrap(), t);
        assert_eq!(t.total_nanos().unwrap(), n);
    }
    for t in [
        Timespec {
            seconds: -1,
            nanos: 0,
        },
        Timespec {
            seconds: 0,
            nanos: -1,
        },
        Timespec {
            seconds: 0,
            nanos: 1000000000,
        },
        Timespec {
            seconds: i64::MAX,
            nanos: 0,
        },
    ] {
        assert!(timespec(t).is_err())
    }
    assert!(Timespec::decode(&[0; 15]).is_err());
}
#[test]
fn monotonic_and_realtime_are_distinct() {
    let (_e, c, t) = setup();
    c.advance(Span::from_nanos(1234567)).unwrap();
    assert_eq!(t.query(4).unwrap(), 1234000);
    assert_eq!(t.query(0).unwrap(), 9_000_000_000);
}
#[test]
fn supported_aliases_and_modeled_resolution() {
    let (_e, c, t) = setup();
    c.advance(Span::from_nanos(1234567)).unwrap();
    for id in [4, 5, 7, 11] {
        assert_eq!(t.query(id).unwrap(), 1234000);
    }
    for id in [8, 12] {
        assert_eq!(t.query(id).unwrap(), 1000000);
    }
    assert_eq!(quantum(13).unwrap(), 1_000_000_000);
}
#[test]
fn cpu_clocks_and_unknown_ids_refuse() {
    let (_e, _c, t) = setup();
    for id in [1, 2, 14, 15] {
        assert_eq!(t.query(id), Err(Error::Unsupported));
    }
    assert_eq!(t.query(99), Err(Error::Invalid));
}
#[test]
fn virtual_ticks_have_explicit_frequency_and_overflow() {
    assert_eq!(
        virtual_tsc(Tick::from_nanos(1_000_000_000)).unwrap(),
        TSC_FREQUENCY
    );
    assert_eq!(virtual_tsc(Tick::from_nanos(u64::MAX)), Err(Error::Invalid));
}
#[test]
fn absolute_conversion_past_and_future() {
    let (_e, c, t) = setup();
    c.advance(Span::from_nanos(100)).unwrap();
    assert_eq!(
        absolute(&t.scheduler, t.wall, Timespec::from_nanos(50), 4)
            .unwrap()
            .tick()
            .as_nanos(),
        100
    );
    assert_eq!(
        absolute(&t.scheduler, t.wall, Timespec::from_nanos(9_000_000_500), 0)
            .unwrap()
            .tick()
            .as_nanos(),
        600
    );
}
#[test]
fn synchronization_reuses_monotonic_conversion() {
    let (e, c, _t) = setup();
    c.advance(Span::from_nanos(100)).unwrap();
    let s = Synchronization::new(e.scheduler(), 4, 4).unwrap();
    assert_eq!(s.absolute(0, 250, 4).unwrap().tick().as_nanos(), 250);
    assert!(s.absolute(0, 1000000000, 4).is_err());
}
#[test]
fn manual_microsecond_and_millisecond_never_complete_early() {
    for ns in [1000, 1_000_000] {
        let (_e, c, t) = setup();
        let q = t.clone();
        let h = std::thread::spawn(move || q.sleep(Thread(1), Span::from_nanos(ns)).unwrap());
        pending(&t, 1);
        c.advance(Span::from_nanos(ns - 1)).unwrap();
        assert!(!h.is_finished());
        c.advance(Span::from_nanos(1)).unwrap();
        let r = h.join().unwrap();
        assert_eq!(r.wake, Wake::Completed);
        assert!(r.elapsed_ns >= ns);
        assert_eq!(t.snapshot().pending, 0);
    }
}
#[test]
fn zero_sleep_is_immediately_due() {
    let (_e, _c, t) = setup();
    assert_eq!(
        t.sleep(Thread(1), Span::ZERO).unwrap().wake,
        Wake::Completed
    );
}
#[test]
fn cancel_one_thread_preserves_other_sleeper() {
    let (_e, c, t) = setup();
    let mut hs = vec![];
    for id in [1, 2] {
        let q = t.clone();
        hs.push(std::thread::spawn(move || {
            q.sleep(Thread(id), Span::from_nanos(100)).unwrap()
        }));
    }
    pending(&t, 2);
    t.cancel_thread(Thread(1));
    assert_eq!(hs.remove(0).join().unwrap().wake, Wake::Interrupted);
    c.advance(Span::from_nanos(100)).unwrap();
    assert_eq!(hs.remove(0).join().unwrap().wake, Wake::Completed);
}
#[test]
fn shutdown_releases_all_waiters() {
    let (_e, _c, t) = setup();
    let hs: Vec<_> = (0..4)
        .map(|id| {
            let q = t.clone();
            std::thread::spawn(move || q.sleep(Thread(id), Span::from_nanos(100)).unwrap())
        })
        .collect();
    pending(&t, 4);
    t.shutdown();
    for h in hs {
        assert_eq!(h.join().unwrap().wake, Wake::Interrupted);
    }
    assert_eq!(t.snapshot().pending, 0);
    assert_eq!(t.query(4), Err(Error::Interrupted));
}
#[test]
fn execution_deadline_is_not_sleep_completion() {
    let (_e, c, t) = setup();
    t.arm(Deadline::at(Tick::from_nanos(50)));
    let q = t.clone();
    let h = std::thread::spawn(move || q.sleep(Thread(1), Span::from_nanos(100)).unwrap());
    pending(&t, 1);
    c.advance(Span::from_nanos(50)).unwrap();
    assert_eq!(h.join().unwrap().wake, Wake::Interrupted);
}
#[test]
fn timing_owner_shutdown_is_interruption() {
    let (e, _c, t) = setup();
    let q = t.clone();
    let h = std::thread::spawn(move || q.sleep(Thread(1), Span::from_nanos(100)).unwrap());
    pending(&t, 1);
    e.shutdown().unwrap();
    assert_eq!(h.join().unwrap().wake, Wake::Interrupted);
}
#[test]
fn scheduling_refusal_does_not_retain_waiter() {
    let (e, _c) = TimingEngine::manual(Config {
        max_pending: 0,
        max_snapshot_entries: 0,
    })
    .unwrap();
    let t = GuestTiming::new(e.scheduler(), Realtime::Fixed(0));
    assert!(matches!(
        t.sleep(Thread(1), Span::from_nanos(1)),
        Err(Error::Capacity)
    ));
    assert_eq!(t.snapshot().pending, 0);
}
