use super::super::owned::*;
use astero_timing::time::Deadline;
impl Synchronization {
    pub fn rw_lock(
        &self,
        id: u64,
        address: u64,
        thread: Thread,
        write: bool,
        try_only: bool,
        deadline: Option<Deadline>,
    ) -> Result {
        loop {
            let mut s = self.lock();
            self.expired(&s, None)?;
            let i = Self::index(&s, id, address, Kind::Rwlock)?;
            let o = &mut s.objects[i];
            if o.owner == Some(thread) || (write && o.readers.iter().any(|r| r.0 == thread)) {
                return Err(Error::Deadlock);
            }
            if o.owner.is_none() && (!write || o.readers.is_empty()) {
                if write {
                    o.owner = Some(thread)
                } else if let Some(r) = o.readers.iter_mut().find(|r| r.0 == thread) {
                    r.1 = r.1.checked_add(1).ok_or(Error::Capacity)?
                } else {
                    if o.readers.len() == self.max_waiters {
                        return Err(Error::Capacity);
                    }
                    o.readers.push((thread, 1));
                }
                return Ok(());
            }
            if try_only {
                return Err(Error::Busy);
            }
            self.expired(&s, deadline)?;
            let t = self.ticket(&mut s, thread, id, None, deadline)?;
            drop(s);
            if matches!(t.wait(), astero_timing::scheduler::Completion::Stopped(_)) {
                self.shutdown();
            }
            let mut s = self.lock();
            self.remove_waiter(&mut s, &t);
            if let Err(e) = self.expired(&s, deadline) {
                if e == Error::Timeout {
                    s.timeouts += 1
                }
                return Err(e);
            }
        }
    }
    pub fn rw_unlock(&self, id: u64, address: u64, thread: Thread) -> Result {
        let mut s = self.lock();
        let i = Self::index(&s, id, address, Kind::Rwlock)?;
        let o = &mut s.objects[i];
        if o.owner == Some(thread) {
            o.owner = None
        } else if let Some(i) = o.readers.iter().position(|r| r.0 == thread) {
            o.readers[i].1 -= 1;
            if o.readers[i].1 == 0 {
                o.readers.remove(i);
            }
        } else {
            return Err(Error::Permission);
        }
        self.wake(&mut s, id, false);
        Ok(())
    }
}
