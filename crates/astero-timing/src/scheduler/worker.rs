use super::{model::State, *};
use crate::{
    clock::ClockMode,
    diagnostics::{Counters, TerminalEvent, Transition},
    time::{Span, Tick},
};
use std::sync::Arc;
pub(super) fn stop(shared: &Shared, reason: StopReason) {
    let mut state = shared.lock();
    if state.stopped.is_some() {
        return;
    }
    state.stopped = Some(reason);
    state.worker_waiting = false;
    let now = shared.clock.now().ok();
    while let Some(e) = state.events.pop() {
        e.signal.complete(Completion::Stopped(reason));
        let c = &mut state.counters;
        Counters::add(&mut c.released, &mut c.saturated);
        state.last_terminal = Some(TerminalEvent {
            event: e.details,
            observed_at: now,
            transition: Transition::Released,
            lateness: Span::ZERO,
        });
    }
    shared.wake.notify_all();
}
fn expire(state: &mut State, now: Tick) {
    let e = state.events.remove(0);
    let late = now
        .elapsed_since(e.details.deadline.tick())
        .expect("due predicate");
    Counters::add(&mut state.counters.fired, &mut state.counters.saturated);
    state.max_lateness = state.max_lateness.max(late);
    state.last_terminal = Some(TerminalEvent {
        event: e.details,
        observed_at: Some(now),
        transition: Transition::Fired,
        lateness: late,
    });
    e.signal.complete(Completion::Fired(Expiry {
        sequence: e.details.sequence,
        dispatch_order: state.counters.fired,
        scheduled_at: e.details.scheduled_at,
        deadline: e.details.deadline,
        fired_at: now,
        lateness: late,
    }));
}
pub(super) fn run(shared: &Arc<Shared>) {
    loop {
        let mut s = shared.lock();
        if s.stopped.is_some() {
            return;
        }
        let now = match shared.clock.now() {
            Ok(t) => t,
            Err(_) => {
                drop(s);
                stop(shared, StopReason::ClockOverflow);
                return;
            }
        };
        if s.events
            .first()
            .is_some_and(|e| e.details.deadline.is_due(now))
        {
            expire(&mut s, now);
            drop(s);
            continue;
        }
        let wait = s.events.first().map(|e| e.details.deadline.remaining(now));
        s.worker_waiting = true;
        let counters = &mut s.counters;
        Counters::add(&mut counters.worker_waits, &mut counters.saturated);
        if shared.clock.mode() == ClockMode::HostMonotonic
            && let Some(wait) = wait
        {
            // Bound platform timeout conversions; hourly recheck only for very distant deadlines.
            let timeout = wait.as_std().min(std::time::Duration::from_secs(3600));
            let (mut s, _) = shared
                .wake
                .wait_timeout(s, timeout)
                .unwrap_or_else(|p| p.into_inner());
            s.worker_waiting = false;
        } else {
            let mut s = shared.wake.wait(s).unwrap_or_else(|p| p.into_inner());
            s.worker_waiting = false;
        }
    }
}
