use super::super::owned::*;
use astero_timing::time::Deadline;
impl Synchronization {
    pub fn cond_wait(
        &self,
        cond: (u64, u64),
        mutex: (u64, u64),
        thread: Thread,
        deadline: Option<Deadline>,
    ) -> Result {
        let mut s = self.lock();
        Self::index(&s, cond.0, cond.1, Kind::Cond)?;
        let mi = Self::index(&s, mutex.0, mutex.1, Kind::Mutex)?;
        if s.objects[mi].owner != Some(thread) {
            return Err(Error::Permission);
        }
        if s.objects[mi].depth != 1 {
            return Err(Error::Deadlock);
        }
        // Ticket registration and mutex release share the signal coordination lock.
        let t = self.ticket(&mut s, thread, cond.0, Some(mutex.0), deadline)?;
        s.objects[mi].owner = None;
        s.objects[mi].depth = 0;
        self.wake(&mut s, mutex.0, false);
        drop(s);
        if matches!(t.wait(), astero_timing::scheduler::Completion::Stopped(_)) {
            self.shutdown();
        }
        let mut s = self.lock();
        let signaled = s
            .waiters
            .iter()
            .any(|w| w.ticket.sequence() == t.sequence() && w.signaled);
        let outcome = if signaled {
            self.expired(&s, None)
        } else {
            self.expired(&s, deadline)
        };
        if outcome == Err(Error::Timeout) {
            s.timeouts += 1;
        }
        // Keep the waiter reservation until reacquisition prevents destroy during handoff.
        drop(s);
        let reacquire = self.mutex_lock(mutex.0, mutex.1, thread, false, None);
        let mut s = self.lock();
        self.remove_waiter(&mut s, &t);
        drop(s);
        // Runtime cancellation cannot promise reacquisition: it exits HLE, never resumes guest.
        reacquire?;
        outcome
    }
    pub fn cond_signal(&self, id: u64, address: u64, broadcast: bool) -> Result {
        let mut s = self.lock();
        let i = Self::index(&s, id, address, Kind::Cond)?;
        s.objects[i].generation = s.objects[i]
            .generation
            .checked_add(1)
            .ok_or(Error::Capacity)?;
        self.wake(&mut s, id, !broadcast);
        Ok(())
    }
}
