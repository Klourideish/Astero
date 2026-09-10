//! Borrowed checked-copy interface. A provider never receives a Rust guest reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessError {
    Range,
    Limit,
    Allocation,
    InvalidAllocation,
}
pub trait GuestMemory {
    fn read(&self, address: u64, size: u64) -> Result<Vec<u8>, AccessError>;
    fn write(&mut self, address: u64, bytes: &[u8]) -> Result<(), AccessError>;
    fn allocate(&mut self, size: u64) -> Result<u64, AccessError>;
    fn free(&mut self, address: u64) -> Result<(), AccessError>;
    fn allocation_size(&self, address: u64) -> Result<u64, AccessError>;
}
pub struct Unavailable;
impl GuestMemory for Unavailable {
    fn read(&self, _: u64, _: u64) -> Result<Vec<u8>, AccessError> {
        Err(AccessError::Range)
    }
    fn write(&mut self, _: u64, _: &[u8]) -> Result<(), AccessError> {
        Err(AccessError::Range)
    }
    fn allocate(&mut self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::Allocation)
    }
    fn free(&mut self, _: u64) -> Result<(), AccessError> {
        Err(AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}
