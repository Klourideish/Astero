use astero_timing::{clock::ClockMode, scheduler::*, time::*};
use std::{
    sync::{Arc, Barrier},
    time::{Duration, Instant},
};
fn config() -> Config {
    Config {
        max_pending: 128,
        max_snapshot_entries: 16,
    }
}
fn label() -> Label {
    Label::new("test-consumer").unwrap()
}
fn ms(n: u64) -> Span {
    Span::from_millis(n).unwrap()
}
fn done(t: &Ticket) -> Completion {
    t.wait_timeout(Duration::from_secs(5))
        .expect("worker must finish within generous host guard")
}
fn fired(t: &Ticket) -> Expiry {
    let Completion::Fired(e) = done(t) else {
        panic!("not expiry")
    };
    e
}
#[test]
fn typed_arithmetic_refuses_overflow_and_orders_deadlines() {
    assert_eq!(Span::from_millis(u64::MAX), Err(TimeError::Overflow));
    assert_eq!(Span::from_std(Duration::MAX), Err(TimeError::Overflow));
    assert_eq!(
        Tick::from_nanos(u64::MAX).checked_add(Span::from_nanos(1)),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        Tick::ZERO.elapsed_since(Tick::from_nanos(1)),
        Err(TimeError::BeforeOrigin)
    );
    let d = Deadline::after(Tick::ZERO, ms(5)).unwrap();
    assert!(!d.is_due(Tick::ZERO));
    assert_eq!(d.remaining(Tick::from_nanos(5_000_000)), Span::ZERO);
    assert_eq!(
        Deadline::after(Tick::ZERO, Span::ZERO).unwrap(),
        Deadline::at(Tick::ZERO)
    );
}
#[test]
fn manual_advance_is_forward_only_and_delivers_at_exact_boundary() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let t = s.after(ms(5), label()).unwrap();
    c.advance(ms(4)).unwrap();
    assert_eq!(t.poll(), None);
    c.advance(ms(1)).unwrap();
    let f = fired(&t);
    assert_eq!(f.fired_at, Tick::from_nanos(5_000_000));
    assert_eq!(f.lateness, Span::ZERO);
    assert_eq!(
        c.advance(Span::from_nanos(u64::MAX))
            .err()
            .map(|e| e.to_string()),
        Some("timing: Time(Overflow)".into())
    );
    assert_eq!(c.now().unwrap(), f.fired_at);
}
#[test]
fn equal_deadlines_use_registration_sequence_not_consumer_order() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let mut tickets = Vec::new();
    for _ in 0..12 {
        tickets.push(
            s.schedule(Deadline::at(Tick::from_nanos(10)), label())
                .unwrap(),
        );
    }
    c.advance(Span::from_nanos(10)).unwrap();
    for (i, t) in tickets.iter().enumerate().rev() {
        assert_eq!(fired(t).dispatch_order, i as u64 + 1);
        assert_eq!(fired(t).sequence, t.sequence());
    }
    assert_eq!(s.snapshot(0).unwrap().counters.fired, 12);
}
#[test]
fn past_deadlines_and_zero_delays_are_due_without_spin_or_time_advance() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    c.advance(ms(10)).unwrap();
    let s = e.scheduler();
    let t = s.schedule(Deadline::at(Tick::ZERO), label()).unwrap();
    assert_eq!(fired(&t).lateness, ms(10));
    assert_eq!(
        fired(&s.after(Span::ZERO, label()).unwrap()).lateness,
        Span::ZERO
    );
}
#[test]
fn cancellation_and_ticket_clones_publish_one_terminal_outcome() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let t = s.after(ms(5), label()).unwrap();
    let copy = t.clone();
    assert_eq!(s.cancel(&t).unwrap(), CancelOutcome::Cancelled);
    assert_eq!(s.cancel(&copy).unwrap(), CancelOutcome::AlreadyCancelled);
    c.advance(ms(10)).unwrap();
    assert_eq!(done(&copy), Completion::Cancelled);
    let other = TimingEngine::real(config()).unwrap();
    assert!(matches!(
        other.scheduler().cancel(&t),
        Err(Error::ForeignTicket)
    ));
    assert_eq!(s.snapshot(2).unwrap().counters.cancelled, 1);
}
#[test]
fn cancellation_vs_expiry_race_never_claims_both() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    for _ in 0..32 {
        let t = s.after(ms(1), label()).unwrap();
        let gate = Arc::new(Barrier::new(2));
        let g = gate.clone();
        let cc = c.clone();
        let h = std::thread::spawn(move || {
            g.wait();
            cc.advance(ms(1)).unwrap();
        });
        gate.wait();
        let cancel = s.cancel(&t).unwrap();
        h.join().unwrap();
        match cancel {
            CancelOutcome::Cancelled => assert_eq!(done(&t), Completion::Cancelled),
            CancelOutcome::AlreadyFired => {
                fired(&t);
            }
            _ => panic!("unexpected race state"),
        }
    }
    let snap = s.snapshot(0).unwrap();
    assert_eq!(snap.counters.fired + snap.counters.cancelled, 32);
}
#[test]
fn capacity_labels_and_snapshot_limits_are_explicit() {
    let (e, _) = TimingEngine::manual(Config {
        max_pending: 2,
        max_snapshot_entries: 1,
    })
    .unwrap();
    let s = e.scheduler();
    let a = s.after(ms(10), label()).unwrap();
    let _b = s.after(ms(5), label()).unwrap();
    assert!(matches!(
        s.after(ms(1), label()),
        Err(Error::Capacity { maximum: 2 })
    ));
    let v = s.snapshot(999).unwrap();
    assert_eq!(
        (v.pending_total, v.retained.len(), v.omitted, v.peak_pending),
        (2, 1, 1, 2)
    );
    assert_eq!(
        v.retained[0].deadline,
        Deadline::at(Tick::from_nanos(5_000_000))
    );
    assert_eq!(v.counters.rejected, 1);
    s.cancel(&a).unwrap();
    s.after(ms(1), label()).unwrap();
    assert!(Label::new(&"a".repeat(49)).is_err());
    assert_eq!(Label::new(&"a".repeat(48)).unwrap().as_str().len(), 48);
    let (z, _) = TimingEngine::manual(Config {
        max_pending: 0,
        max_snapshot_entries: 0,
    })
    .unwrap();
    assert!(matches!(
        z.scheduler().after(Span::ZERO, label()),
        Err(Error::Capacity { maximum: 0 })
    ));
    assert!(matches!(
        TimingEngine::real(Config {
            max_pending: usize::MAX,
            max_snapshot_entries: 0
        }),
        Err(Error::Allocation { .. })
    ));
}
#[test]
fn shutdown_releases_waiters_and_all_repeated_calls_join() {
    let (e, _) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let t = s.after(ms(100), label()).unwrap();
    let waiter = t.clone();
    let h = std::thread::spawn(move || waiter.wait());
    e.shutdown().unwrap();
    e.shutdown().unwrap();
    assert_eq!(h.join().unwrap(), Completion::Stopped(StopReason::Shutdown));
    assert!(matches!(s.after(ms(1), label()), Err(Error::Closed)));
    assert_eq!(
        s.cancel(&t).unwrap(),
        CancelOutcome::Stopped(StopReason::Shutdown)
    );
    let v = s.snapshot(10).unwrap();
    assert_eq!(v.counters.released, 1);
    assert!(!v.worker_waiting);
    assert_eq!(v.pending_total, 0);
}
#[test]
fn dropping_owner_joins_despite_retained_handles() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let t = s.after(ms(100), label()).unwrap();
    drop(e);
    assert_eq!(done(&t), Completion::Stopped(StopReason::Shutdown));
    assert!(matches!(s.now(), Err(Error::Closed)));
    assert!(matches!(c.advance(ms(1)), Err(Error::Closed)));
}
#[test]
fn concurrent_producers_and_cancellers_have_unique_sequences() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let mut threads = Vec::new();
    for _ in 0..4 {
        let s = s.clone();
        threads.push(std::thread::spawn(move || {
            (0..20)
                .map(|i| {
                    let t = s.after(ms(10), label()).unwrap();
                    if i % 2 == 0 {
                        s.cancel(&t).unwrap();
                    }
                    t
                })
                .collect::<Vec<_>>()
        }));
    }
    let mut tickets = threads
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    tickets.sort_by_key(Ticket::sequence);
    assert!(
        tickets
            .windows(2)
            .all(|w| w[0].sequence() < w[1].sequence())
    );
    c.advance(ms(10)).unwrap();
    for t in &tickets {
        done(t);
    }
    let v = s.snapshot(0).unwrap();
    assert_eq!(
        (v.counters.scheduled, v.counters.fired, v.counters.cancelled),
        (80, 40, 40)
    );
}
#[test]
fn concurrent_shutdown_and_registration_release_every_accepted_ticket() {
    let (e, _) = TimingEngine::manual(config()).unwrap();
    let e = Arc::new(e);
    let s = e.scheduler();
    let gate = Arc::new(Barrier::new(3));
    let g = gate.clone();
    let p = std::thread::spawn(move || {
        g.wait();
        (0..40)
            .filter_map(|_| match s.after(ms(1), label()) {
                Ok(t) => Some(t),
                Err(Error::Closed) => None,
                Err(e) => panic!("{e}"),
            })
            .collect::<Vec<_>>()
    });
    let e2 = e.clone();
    let g = gate.clone();
    let closer = std::thread::spawn(move || {
        g.wait();
        e2.shutdown().unwrap();
    });
    gate.wait();
    e.shutdown().unwrap();
    closer.join().unwrap();
    for t in p.join().unwrap() {
        assert_eq!(done(&t), Completion::Stopped(StopReason::Shutdown));
    }
}
#[test]
fn unread_completions_do_not_block_scheduler_or_other_consumers() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let slow = s.after(ms(1), label()).unwrap();
    let fast = s.after(ms(2), label()).unwrap();
    c.advance(ms(2)).unwrap();
    assert_eq!(fired(&fast).dispatch_order, 2);
    assert_eq!(fired(&slow).dispatch_order, 1);
    let v = s.snapshot(16).unwrap();
    assert_eq!(v.max_lateness, ms(1));
    assert_eq!(v.last_terminal.unwrap().event.sequence, fast.sequence());
    assert_eq!(v.mode, ClockMode::Manual);
}
#[test]
fn earlier_real_deadline_wakes_parked_worker_and_never_fires_early() {
    let e = TimingEngine::real(config()).unwrap();
    let s = e.scheduler();
    let late = s.after(ms(30_000), label()).unwrap();
    let guard = Instant::now();
    while !s.snapshot(0).unwrap().worker_waiting {
        assert!(guard.elapsed() < Duration::from_secs(5));
        std::thread::yield_now();
    }
    let before = Instant::now();
    let early = s.after(ms(15), label()).unwrap();
    let f = fired(&early);
    assert!(f.fired_at >= f.deadline.tick());
    assert!(before.elapsed() < Duration::from_secs(5));
    assert_eq!(late.poll(), None);
    println!(
        "15ms host request: elapsed={:?}, observed lateness={}ns",
        before.elapsed(),
        f.lateness.as_nanos()
    );
    assert_eq!(s.cancel(&early).unwrap(), CancelOutcome::AlreadyFired);
    s.cancel(&late).unwrap();
    for ns in [5_333_333, 16_666_667] {
        let start = Instant::now();
        let t = s.after(Span::from_nanos(ns), label()).unwrap();
        let f = fired(&t);
        assert!(f.fired_at >= f.deadline.tick());
        println!(
            "host request={}ns elapsed={:?} lateness={}ns",
            ns,
            start.elapsed(),
            f.lateness.as_nanos()
        );
    }
}

#[test]
fn different_deadlines_and_shutdown_expiry_races_preserve_terminal_truth() {
    let (e, c) = TimingEngine::manual(config()).unwrap();
    let s = e.scheduler();
    let late = s.after(ms(20), label()).unwrap();
    let early = s.after(ms(10), label()).unwrap();
    c.advance(ms(20)).unwrap();
    assert_eq!(fired(&early).dispatch_order, 1);
    assert_eq!(fired(&late).dispatch_order, 2);
    for _ in 0..8 {
        let (e, c) = TimingEngine::manual(config()).unwrap();
        let t = e.scheduler().after(ms(1), label()).unwrap();
        let gate = Arc::new(Barrier::new(2));
        let g = gate.clone();
        let h = std::thread::spawn(move || {
            g.wait();
            let _ = c.advance(ms(1));
        });
        gate.wait();
        e.shutdown().unwrap();
        h.join().unwrap();
        assert!(matches!(
            done(&t),
            Completion::Fired(_) | Completion::Stopped(StopReason::Shutdown)
        ));
    }
}
