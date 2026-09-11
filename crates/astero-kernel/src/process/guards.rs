//! Bounded process-owned once-initialization exclusion; guest bytes remain a libs contract.
use crate::synchronization::owned::{Error, Kind, Result, Synchronization, Thread};
use astero_timing::{scheduler::Scheduler, time::Deadline};
use std::sync::Mutex;
pub struct Guards {
    sync: Synchronization,
    ids: Mutex<Vec<(u64, u64)>>,
    capacity: usize,
}
impl Guards {
    pub fn new(scheduler: Scheduler, capacity: usize) -> Result<Self> {
        let mut ids = Vec::new();
        ids.try_reserve_exact(capacity)
            .map_err(|_| Error::Capacity)?;
        Ok(Self {
            sync: Synchronization::new(scheduler, capacity, 64)?,
            ids: Mutex::new(ids),
            capacity,
        })
    }
    pub fn arm(&self, deadline: Deadline) -> Result {
        self.sync.arm(deadline)
    }
    pub fn shutdown(&self) {
        self.sync.shutdown();
    }
    pub fn snapshot(&self) -> crate::synchronization::owned::Snapshot {
        self.sync.snapshot()
    }
    fn id(&self, address: u64, create: bool) -> Result<u64> {
        let mut ids = self.ids.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((_, id)) = ids.iter().find(|(a, _)| *a == address) {
            return Ok(*id);
        }
        if !create || address == 0 || !address.is_multiple_of(8) {
            return Err(Error::Invalid);
        }
        if ids.len() == self.capacity {
            return Err(Error::Capacity);
        }
        let id = self.sync.create(address, Kind::Mutex, 1)?; // errorcheck: recursive initialization refuses
        ids.push((address, id));
        Ok(id)
    }
    pub fn acquire(&self, address: u64, thread: Thread) -> Result {
        self.sync
            .mutex_lock(self.id(address, true)?, address, thread, false, None)
    }
    pub fn require_owner(&self, address: u64, thread: Thread) -> Result {
        self.sync
            .mutex_require_owner(self.id(address, false)?, address, thread)
    }
    pub fn release(&self, address: u64, thread: Thread) -> Result {
        self.sync
            .mutex_unlock(self.id(address, false)?, address, thread)
    }
}
