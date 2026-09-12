use super::super::owned::*;
use astero_timing::time::Deadline;
impl Synchronization {
    pub fn mutex_require_owner(&self, id: u64, address: u64, thread: Thread) -> Result {
        let s = self.lock();
        let i = Self::index(&s, id, address, Kind::Mutex)?;
        if s.objects[i].owner == Some(thread) {
            Ok(())
        } else {
            Err(Error::Permission)
        }
    }

    pub fn mutex_lock(
        &self,
        id: u64,
        address: u64,
        thread: Thread,
        try_only: bool,
        deadline: Option<Deadline>,
    ) -> Result {
        loop {
            let mut s = self.lock();
            self.expired(&s, None)?;
            let i = Self::index(&s, id, address, Kind::Mutex)?;
            let o = &mut s.objects[i];
            if o.owner.is_none() {
                o.owner = Some(thread);
                o.depth = 1;
                return Ok(());
            }
            if o.owner == Some(thread) && o.value == 2 {
                o.depth = o.depth.checked_add(1).ok_or(Error::Capacity)?;
                return Ok(());
            }
            if try_only {
                return Err(Error::Busy);
            }
            if o.owner == Some(thread) && o.value == 1 {
                return Err(Error::Deadlock);
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
    pub fn mutex_unlock(&self, id: u64, address: u64, thread: Thread) -> Result {
        let mut s = self.lock();
        let i = Self::index(&s, id, address, Kind::Mutex)?;
        let o = &mut s.objects[i];
        if o.owner != Some(thread) {
            return Err(Error::Permission);
        }
        o.depth -= 1;
        if o.depth == 0 {
            o.owner = None;
            self.wake(&mut s, id, false)
        }
        Ok(())
    }
}
