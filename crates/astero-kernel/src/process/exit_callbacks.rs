//! Bounded guest exit-callback storage. This service never invokes stored addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackError {
    Null,
    Capacity,
    Allocation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackTarget {
    Guest(u64),
    RuntimeReturn(u64),
}
impl CallbackTarget {
    pub fn address(self) -> u64 {
        match self {
            Self::Guest(a) | Self::RuntimeReturn(a) => a,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackRecord {
    pub target: CallbackTarget,
    pub argument: Option<u64>,
    pub dso: Option<u64>,
}
pub struct ExitCallbacks {
    addresses: Vec<CallbackRecord>,
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
        self.register_target(CallbackTarget::Guest(address))
    }
    pub fn register_target(&mut self, target: CallbackTarget) -> Result<(), CallbackError> {
        self.register_record(CallbackRecord {
            target,
            argument: None,
            dso: None,
        })
    }
    pub fn register_record(&mut self, record: CallbackRecord) -> Result<(), CallbackError> {
        if record.target.address() == 0 {
            return Err(CallbackError::Null);
        }
        if self.addresses.len() >= self.maximum {
            return Err(CallbackError::Capacity);
        }
        self.addresses
            .try_reserve(1)
            .map_err(|_| CallbackError::Allocation)?;
        self.addresses.push(record);
        Ok(())
    }
    /// Reverse registration order, retaining duplicates. Observation, not execution.
    pub fn pending(&self) -> impl Iterator<Item = u64> + '_ {
        self.addresses.iter().rev().map(|t| t.target.address())
    }
    pub fn records(&self) -> impl Iterator<Item = &CallbackRecord> {
        self.addresses.iter().rev()
    }
    pub fn len(&self) -> usize {
        self.addresses.len()
    }
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }
}
