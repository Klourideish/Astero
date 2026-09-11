//! Runtime-owned AJM lifecycle bookkeeping; codec execution is deliberately separate.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidContext,
    InvalidParameter,
    Capacity,
    Stopped,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub contexts: usize,
    pub instances: usize,
    pub memories: usize,
    pub stopped: bool,
}
#[derive(Default)]
struct Context {
    modules: BTreeSet<u32>,
    instances: BTreeMap<u32, (u32, u64)>,
    memory: BTreeMap<u64, u64>,
}
struct State {
    contexts: BTreeMap<u32, Context>,
    next: u32,
    stopped: bool,
}
pub struct Ajm {
    state: Mutex<State>,
    capacity: usize,
}
impl Ajm {
    pub fn new(capacity: usize) -> Self {
        Self {
            state: Mutex::new(State {
                contexts: BTreeMap::new(),
                next: 1,
                stopped: false,
            }),
            capacity,
        }
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn id(s: &mut State) -> Result<u32, Error> {
        if s.stopped {
            return Err(Error::Stopped);
        }
        let id = s.next;
        s.next = id.checked_add(1).ok_or(Error::Capacity)?;
        Ok(id)
    }
    fn count(s: &State) -> usize {
        s.contexts.len()
            + s.contexts
                .values()
                .map(|c| c.modules.len() + c.instances.len() + c.memory.len())
                .sum::<usize>()
    }
    pub fn initialize(&self) -> Result<u32, Error> {
        let mut s = self.lock();
        if Self::count(&s) >= self.capacity {
            return Err(Error::Capacity);
        }
        let id = Self::id(&mut s)?;
        s.contexts.insert(id, Context::default());
        Ok(id)
    }
    pub fn finalize(&self, id: u32) -> Result<(), Error> {
        self.lock()
            .contexts
            .remove(&id)
            .map(|_| ())
            .ok_or(Error::InvalidContext)
    }
    pub fn module(&self, id: u32, codec: u32, register: bool) -> Result<(), Error> {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Stopped);
        }
        let full = Self::count(&s) >= self.capacity;
        let c = s.contexts.get_mut(&id).ok_or(Error::InvalidContext)?;
        if register {
            if full && !c.modules.contains(&codec) {
                return Err(Error::Capacity);
            }
            c.modules.insert(codec);
        } else {
            if c.instances.values().any(|v| v.0 == codec) {
                return Err(Error::InvalidParameter);
            }
            c.modules.remove(&codec);
        }
        Ok(())
    }
    pub fn memory(&self, id: u32, address: u64, bytes: u64, register: bool) -> Result<(), Error> {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Stopped);
        }
        let full = Self::count(&s) >= self.capacity;
        let c = s.contexts.get_mut(&id).ok_or(Error::InvalidContext)?;
        if address == 0 {
            return Err(Error::InvalidParameter);
        }
        if register {
            if bytes == 0 || address.checked_add(bytes).is_none() {
                return Err(Error::InvalidParameter);
            }
            if full && !c.memory.contains_key(&address) {
                return Err(Error::Capacity);
            }
            c.memory.insert(address, bytes);
        } else {
            c.memory.remove(&address);
        }
        Ok(())
    }
    pub fn create(&self, id: u32, codec: u32, flags: u64) -> Result<u32, Error> {
        let mut s = self.lock();
        if !s
            .contexts
            .get(&id)
            .ok_or(Error::InvalidContext)?
            .modules
            .contains(&codec)
        {
            return Err(Error::InvalidParameter);
        }
        if Self::count(&s) >= self.capacity {
            return Err(Error::Capacity);
        }
        let instance = Self::id(&mut s)?;
        s.contexts
            .get_mut(&id)
            .ok_or(Error::InvalidContext)?
            .instances
            .insert(instance, (codec, flags));
        Ok(instance)
    }
    pub fn destroy(&self, id: u32, instance: u32) -> Result<(), Error> {
        self.lock()
            .contexts
            .get_mut(&id)
            .ok_or(Error::InvalidContext)?
            .instances
            .remove(&instance)
            .map(|_| ())
            .ok_or(Error::InvalidParameter)
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.lock();
        Snapshot {
            contexts: s.contexts.len(),
            instances: s.contexts.values().map(|c| c.instances.len()).sum(),
            memories: s.contexts.values().map(|c| c.memory.len()).sum(),
            stopped: s.stopped,
        }
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        s.stopped = true;
        s.contexts.clear();
    }
}
