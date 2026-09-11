//! PS5Rust attribute baseline with explicit unsupported host-policy handling.
use super::lifecycle::{Error, Result};
use std::sync::Mutex;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attributes {
    pub stack_address: u64,
    pub stack_size: u64,
    pub guard_size: u64,
    pub detached: bool,
    pub priority: i32,
    pub policy: i32,
    pub inherit: i32,
}
impl Default for Attributes {
    fn default() -> Self {
        Self {
            stack_address: 0,
            stack_size: 0x200000,
            guard_size: u64::MAX,
            detached: false,
            priority: 700,
            policy: 1,
            inherit: 0,
        }
    }
}
impl Attributes {
    pub fn validate(&self) -> Result {
        if self.stack_size < 0x4000
            || self.stack_size > 0x800000
            || !self.stack_size.is_multiple_of(0x1000)
        {
            return Err(Error::Invalid);
        }
        // M34 owns guarded stacks. Caller-supplied stacks and host scheduling are not silently ignored.
        if self.stack_address != 0
            || ![u64::MAX, 0, 0x1000].contains(&self.guard_size)
            || self.policy != 1
            || self.priority != 700
            || ![0, 1].contains(&self.inherit)
        {
            return Err(Error::Unsupported);
        }
        Ok(())
    }
}
struct AttributeState {
    next: u64,
    entries: Vec<(u64, u64, Attributes)>,
}
pub struct AttributeTable {
    state: Mutex<AttributeState>,
    capacity: usize,
}
impl AttributeTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            state: Mutex::new(AttributeState {
                next: 1,
                entries: Vec::new(),
            }),
            capacity,
        }
    }
    pub fn create(&self, address: u64) -> Result<u64> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if address == 0 || s.entries.iter().any(|(_, a, _)| *a == address) {
            return Err(Error::Invalid);
        }
        if s.entries.len() == self.capacity {
            return Err(Error::Capacity);
        }
        s.entries.try_reserve(1).map_err(|_| Error::Capacity)?;
        let id = 0xa740000000000000u64
            .checked_add(s.next)
            .ok_or(Error::Capacity)?;
        s.next = s.next.checked_add(1).ok_or(Error::Capacity)?;
        s.entries.push((id, address, Attributes::default()));
        Ok(id)
    }
    pub fn get(&self, id: u64, address: u64) -> Result<Attributes> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entries
            .iter()
            .find(|(i, a, _)| *i == id && *a == address)
            .map(|(_, _, v)| v.clone())
            .ok_or(Error::Invalid)
    }
    pub fn set(&self, id: u64, address: u64, value: Attributes) -> Result {
        value.validate()?;
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let v = s
            .entries
            .iter_mut()
            .find(|(i, a, _)| *i == id && *a == address)
            .ok_or(Error::Invalid)?;
        v.2 = value;
        Ok(())
    }
    pub fn destroy(&self, id: u64, address: u64) -> Result {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let i = s
            .entries
            .iter()
            .position(|(i, a, _)| *i == id && *a == address)
            .ok_or(Error::Invalid)?;
        s.entries.remove(i);
        Ok(())
    }
}
