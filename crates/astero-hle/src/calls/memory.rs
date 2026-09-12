//! Borrowed checked-copy interface. A provider never receives a Rust guest reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessError {
    Range,
    Limit,
    Allocation,
    InvalidAllocation,
    Overlap,
    HostCopy {
        operation: &'static str,
        address: u64,
        size: u64,
        code: u32,
    },
}
/// Runtime policy, independent of whether an address is mapped. Copies use bounded scratch.
pub const COPY_CHUNK_BYTES: u64 = 64 * 1024;
pub const MAX_OPERATION_BYTES: u64 = 64 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeapStats {
    pub arena_bytes: u64,
    pub live_bytes: u64,
    pub peak_bytes: u64,
    pub live: usize,
    pub peak_live: usize,
}
pub trait GuestMemory {
    fn heap_stats(&self) -> Result<HeapStats, AccessError> {
        Err(AccessError::InvalidAllocation)
    }

    /// Non-mutating full-range permission preflight. Zero length accesses no bytes.
    fn validate(&self, address: u64, size: u64, write: bool) -> Result<(), AccessError>;
    fn charge(&self, size: u64) -> Result<(), AccessError> {
        if size > MAX_OPERATION_BYTES {
            Err(AccessError::Limit)
        } else {
            Ok(())
        }
    }
    /// A readable prefix, without requiring bytes beyond a string terminator to be mapped.
    fn read_window(&self, address: u64, maximum: u64) -> Result<Vec<u8>, AccessError> {
        self.read(address, maximum.min(1))
    }
    fn allocate_aligned(&mut self, size: u64, alignment: u64) -> Result<u64, AccessError> {
        if alignment <= 16 && alignment.is_power_of_two() {
            self.allocate(size)
        } else {
            Err(AccessError::Allocation)
        }
    }
    fn read(&self, address: u64, size: u64) -> Result<Vec<u8>, AccessError>;
    fn write(&mut self, address: u64, bytes: &[u8]) -> Result<(), AccessError>;
    fn allocate(&mut self, size: u64) -> Result<u64, AccessError>;
    fn free(&mut self, address: u64) -> Result<(), AccessError>;
    fn usable_size(&self, address: u64) -> Result<u64, AccessError> {
        self.allocation_size(address)
    }
    fn allocation_size(&self, address: u64) -> Result<u64, AccessError>;
}
pub struct Unavailable;
impl GuestMemory for Unavailable {
    fn validate(&self, _: u64, _: u64, _: bool) -> Result<(), AccessError> {
        Err(AccessError::Range)
    }
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
