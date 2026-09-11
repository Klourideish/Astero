use astero_audio::output::service::*;
use astero_timing::{
    clock::ManualClock,
    scheduler::{Config as TC, TimingEngine},
    time::{Deadline, Span, Tick},
};
use std::sync::Arc;
fn setup(n: usize) -> (TimingEngine, ManualClock, Arc<AudioService>) {
    let (e, c) = TimingEngine::manual(TC {
        max_pending: n,
        max_snapshot_entries: 128,
    })
    .unwrap();
    let s = Arc::new(AudioService::new(e.scheduler()));
    (e, c, s)
}
fn cfg() -> Config {
    Config::pcm(480, 48000, 1, 0).unwrap()
}
fn until(mut f: impl FnMut() -> bool) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !f() {
        assert!(std::time::Instant::now() < end);
        std::thread::yield_now();
    }
}
#[test]
fn lifecycle() {
    let (_e, _c, s) = setup(32);
    let a = s.create(Kind::Legacy, None, cfg()).unwrap();
    s.close(a, Kind::Legacy).unwrap();
    assert_eq!(s.close(a, Kind::Legacy), Err(Error::Handle));
    let b = s.create(Kind::Legacy, None, cfg()).unwrap();
    assert_ne!(a, b);
    assert!(s.record(a, Kind::Legacy).is_err());
}
#[test]
fn capacity() {
    let (_e, _c, s) = setup(32);
    for _ in 0..16 {
        s.create(Kind::Legacy, None, cfg()).unwrap();
    }
    assert_eq!(s.create(Kind::Legacy, None, cfg()), Err(Error::Capacity));
}
#[test]
fn formats() {
    for f in 0..8 {
        assert!(Config::pcm(65536, 48000, f, 0).unwrap().bytes() <= 2097152);
    }
    for (f, r, n) in [
        (8, 48000, 256),
        (0, 44100, 256),
        (0, 0, 256),
        (0, 48000, 0),
        (0, 48000, 65537),
    ] {
        assert!(Config::pcm(n, r, f, 0).is_err());
    }
}
#[test]
fn forged_config() {
    let (_e, _c, s) = setup(32);
    let mut c = cfg();
    c.rate = 0;
    assert_eq!(s.create(Kind::Legacy, None, c), Err(Error::Invalid));
}
#[test]
fn exact_period() {
    let (_e, c, s) = setup(32);
    let p = s.create(Kind::Legacy, None, cfg()).unwrap();
    s.submit(vec![(p, vec![7; 1920])], Kind::Legacy, false)
        .unwrap();
    c.advance(Span::from_millis(9).unwrap()).unwrap();
    assert_eq!(s.snapshot().completed, 0);
    c.advance(Span::from_millis(1).unwrap()).unwrap();
    until(|| s.snapshot().completed == 1);
    assert_eq!(s.snapshot().bytes, 1920);
}
#[test]
fn batch_invalid_tail() {
    let (_e, _c, s) = setup(32);
    let p = s.create(Kind::Legacy, None, cfg()).unwrap();
    assert_eq!(
        s.submit(
            vec![(p, vec![0; 1920]), (999, vec![0; 1920])],
            Kind::Legacy,
            false
        ),
        Err(Error::Handle)
    );
    assert_eq!(s.snapshot().submitted, 0);
}
#[test]
fn ticket_failure_rolls_back() {
    let (e, _c, s) = setup(1);
    let a = s.create(Kind::Legacy, None, cfg()).unwrap();
    let b = s.create(Kind::Legacy, None, cfg()).unwrap();
    assert_eq!(
        s.submit(
            vec![(a, vec![0; 1920]), (b, vec![0; 1920])],
            Kind::Legacy,
            false
        ),
        Err(Error::Capacity)
    );
    assert_eq!(s.snapshot().submitted, 0);
    assert_eq!(e.scheduler().snapshot(0).unwrap().pending_total, 0);
}
#[test]
fn invalid_size_duplicate() {
    let (_e, _c, s) = setup(32);
    let p = s.create(Kind::Legacy, None, cfg()).unwrap();
    assert!(
        s.submit(vec![(p, vec![0; 1])], Kind::Legacy, false)
            .is_err()
    );
    assert!(
        s.submit(
            vec![(p, vec![0; 1920]), (p, vec![0; 1920])],
            Kind::Legacy,
            false
        )
        .is_err()
    );
    assert_eq!(s.snapshot().submitted, 0);
}
#[test]
fn context_depth() {
    let (_e, c, s) = setup(32);
    let p = s
        .create(Kind::Context, None, Config::context(480, 2).unwrap())
        .unwrap();
    for _ in 0..2 {
        s.submit(vec![(p, vec![])], Kind::Context, false).unwrap();
    }
    assert_eq!(
        s.submit(vec![(p, vec![])], Kind::Context, false),
        Err(Error::Busy)
    );
    c.advance(Span::from_millis(20).unwrap()).unwrap();
    until(|| s.snapshot().completed == 2);
}
fn blocked(s: &Arc<AudioService>) -> (u64, std::thread::JoinHandle<Result>) {
    let p = s
        .create(Kind::Context, None, Config::context(480, 1).unwrap())
        .unwrap();
    s.submit(vec![(p, vec![])], Kind::Context, false).unwrap();
    let other = s.clone();
    let w = std::thread::spawn(move || other.submit(vec![(p, vec![])], Kind::Context, true));
    until(|| s.snapshot().waits == 1);
    (p, w)
}
#[test]
fn blocking_completion() {
    let (_e, c, s) = setup(32);
    let (_, w) = blocked(&s);
    c.advance(Span::from_millis(10).unwrap()).unwrap();
    assert_eq!(w.join().unwrap(), Ok(()));
}
#[test]
fn shutdown_interrupts() {
    let (e, _c, s) = setup(32);
    let (_, w) = blocked(&s);
    s.shutdown();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
    assert!(s.snapshot().objects.is_empty());
    assert_eq!(e.scheduler().snapshot(0).unwrap().pending_total, 0);
}
#[test]
fn deadline_not_completion() {
    let (_e, c, s) = setup(32);
    s.arm(Deadline::at(Tick::from_nanos(1000000)));
    let (_, w) = blocked(&s);
    c.advance(Span::from_millis(1).unwrap()).unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
    assert_eq!(s.snapshot().completed, 0);
}
#[test]
fn close_interrupts() {
    let (_e, _c, s) = setup(32);
    let (p, w) = blocked(&s);
    s.close(p, Kind::Context).unwrap();
    assert_eq!(w.join().unwrap(), Err(Error::Interrupted));
}
#[test]
fn child_kind_lifetime() {
    let (_e, _c, s) = setup(32);
    let c = s.create(Kind::Context, None, cfg()).unwrap();
    let p = s.create(Kind::Port, Some(c), cfg()).unwrap();
    assert_eq!(s.close(c, Kind::Context), Err(Error::Busy));
    assert_eq!(s.record(p, Kind::User).unwrap_err(), Error::Handle);
    s.close(p, Kind::Port).unwrap();
    s.close(c, Kind::Context).unwrap();
}
#[test]
fn volume() {
    let (_e, _c, s) = setup(32);
    let p = s.create(Kind::Legacy, None, cfg()).unwrap();
    s.volume(p, 2, [99; 8]).unwrap();
    let v = s.record(p, Kind::Legacy).unwrap().volume;
    assert_eq!((v[0], v[1]), (127, 99));
    assert_eq!(s.volume(p, 256, [0; 8]), Err(Error::Invalid));
}
#[test]
fn concurrent_teardown() {
    let (e, _c, s) = setup(32);
    let w: Vec<_> = (0..8)
        .map(|_| {
            let s = s.clone();
            std::thread::spawn(move || {
                let p = s.create(Kind::Legacy, None, cfg()).unwrap();
                s.submit(vec![(p, vec![0; 1920])], Kind::Legacy, false)
                    .unwrap();
            })
        })
        .collect();
    for w in w {
        w.join().unwrap()
    }
    s.shutdown();
    s.shutdown();
    assert_eq!(s.snapshot().closed, 8);
    assert_eq!(s.snapshot().retired.len(), 8);
    assert_eq!(e.scheduler().snapshot(0).unwrap().pending_total, 0);
}
