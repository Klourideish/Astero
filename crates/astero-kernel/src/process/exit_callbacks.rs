//! Bounded guest exit-callback storage. This service never invokes stored addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackError {
    Null,
    Capacity,
    Allocation,
}
pub struct ExitCallbacks {
    addresses: Vec<u64>,
    maximum: usize,
}
impl ExitCallbacks {
    pub fn new(maximum: usize) -> Self {
        Self {
            addresses: Vec::new(),
            maximum,
        }
    }
    pub fn register(&mut self, address: u64) -> Result<(), CallbackError> {
        if address == 0 {
            return Err(CallbackError::Null);
        }
        if self.addresses.len() >= self.maximum {
            return Err(CallbackError::Capacity);
        }
        self.addresses
            .try_reserve(1)
            .map_err(|_| CallbackError::Allocation)?;
        self.addresses.push(address);
        Ok(())
    }
    /// Reverse registration order, retaining duplicates. Observation, not execution.
    pub fn pending(&self) -> impl Iterator<Item = u64> + '_ {
        self.addresses.iter().rev().copied()
    }
    pub fn len(&self) -> usize {
        self.addresses.len()
    }
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }
}
