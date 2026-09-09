use super::{model::Event, ticket::Signal, *};
use crate::{
    diagnostics::{Counters, PendingEvent, Snapshot, TerminalEvent, Transition},
    time::{Deadline, Span, Tick},
};
use std::sync::{Arc, Weak};
#[derive(Clone)]
pub struct Scheduler {
    pub(super) shared: Weak<Shared>,
}
impl Scheduler {
    pub fn now(&self) -> Result<Tick, Error> {
        self.shared
            .upgrade()
            .ok_or(Error::Closed)?
            .clock
            .now()
            .map_err(Error::Time)
    }
    pub fn after(&self, delay: Span, label: Label) -> Result<Ticket, Error> {
        self.schedule(Deadline::after(self.now()?, delay)?, label)
    }
    pub fn schedule(&self, deadline: Deadline, label: Label) -> Result<Ticket, Error> {
        let shared = self.shared.upgrade().ok_or(Error::Closed)?;
        let mut s = shared.lock();
        if s.stopped.is_some() {
            return Err(Error::Closed);
        }
        if s.events.len() == shared.config.max_pending {
            let c = &mut s.counters;
            Counters::add(&mut c.rejected, &mut c.saturated);
            return Err(Error::Capacity {
                maximum: shared.config.max_pending,
            });
        }
        let sequence = s.next_sequence;
        let next = sequence.checked_add(1).ok_or(Error::SequenceExhausted)?;
        let scheduled_at = shared.clock.now()?;
        let signal = Arc::new(Signal::new());
        let position = s
            .events
            .partition_point(|e| (e.details.deadline, e.details.sequence) < (deadline, sequence));
        s.events.insert(
            position,
            Event {
                details: PendingEvent {
                    sequence,
                    label,
                    scheduled_at,
                    deadline,
                },
                signal: signal.clone(),
            },
        );
        s.next_sequence = next;
        let c = &mut s.counters;
        Counters::add(&mut c.scheduled, &mut c.saturated);
        s.peak_pending = s.peak_pending.max(s.events.len());
        shared.wake.notify_all();
        Ok(Ticket {
            shared: Arc::downgrade(&shared),
            signal,
            sequence,
        })
    }
    pub fn cancel(&self, ticket: &Ticket) -> Result<CancelOutcome, Error> {
        if !Weak::ptr_eq(&self.shared, &ticket.shared) {
            return Err(Error::ForeignTicket);
        }
        let shared = self.shared.upgrade().ok_or(Error::Closed)?;
        let mut s = shared.lock();
        if let Some(outcome) = ticket.poll() {
            return Ok(match outcome {
                Completion::Fired(_) => CancelOutcome::AlreadyFired,
                Completion::Cancelled => CancelOutcome::AlreadyCancelled,
                Completion::Stopped(r) => CancelOutcome::Stopped(r),
            });
        }
        let i = s
            .events
            .iter()
            .position(|e| e.details.sequence == ticket.sequence)
            .expect("pending ticket belongs to queue");
        let now = shared.clock.now()?;
        let e = s.events.remove(i);
        e.signal.complete(Completion::Cancelled);
        let c = &mut s.counters;
        Counters::add(&mut c.cancelled, &mut c.saturated);
        s.last_terminal = Some(TerminalEvent {
            event: e.details,
            observed_at: Some(now),
            transition: Transition::Cancelled,
            lateness: Span::ZERO,
        });
        shared.wake.notify_all();
        Ok(CancelOutcome::Cancelled)
    }
    pub fn snapshot(&self, maximum: usize) -> Result<Snapshot, Error> {
        let shared = self.shared.upgrade().ok_or(Error::Closed)?;
        let s = shared.lock();
        let count = s
            .events
            .len()
            .min(maximum)
            .min(shared.config.max_snapshot_entries);
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation { entries: count })?;
        retained.extend(s.events.iter().take(count).map(|e| e.details));
        Ok(Snapshot {
            mode: shared.clock.mode(),
            now: shared.clock.now()?,
            stopped: s.stopped,
            pending_total: s.events.len(),
            next_deadline: s.events.first().map(|e| e.details.deadline),
            retained,
            omitted: s.events.len() - count,
            counters: s.counters,
            worker_waiting: s.worker_waiting,
            peak_pending: s.peak_pending,
            max_lateness: s.max_lateness,
            last_terminal: s.last_terminal,
        })
    }
}
