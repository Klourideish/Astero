use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
/// Logical guest address, not a host pointer. Native reservation is a separate backend contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuestAddress(pub u64);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    Overflow,
    Capacity { requested: u64, maximum: u64 },
    Overlap,
    Allocation,
    Unmapped { address: u64, size: u64 },
    Finalized,
}
impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "guest memory: {self:?}")
    }
}
impl std::error::Error for MemoryError {}
/// Per-image observer survives teardown; never a global registry.
#[derive(Clone, Debug, Default)]
pub struct MappingObserver(Arc<AtomicU64>);
impl MappingObserver {
    pub fn active_mappings(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
#[derive(Debug)]
struct Region {
    start: u64,
    bytes: Vec<u8>,
}
/// Non-executable byte backend. Writes are available only during isolated staging.
#[derive(Debug)]
pub struct OwnedAddressSpace {
    regions: Vec<Region>,
    maximum: u64,
    used: u64,
    frozen: bool,
    observer: MappingObserver,
}
impl OwnedAddressSpace {
    pub fn new(maximum: u64) -> Self {
        Self {
            regions: Vec::new(),
            maximum,
            used: 0,
            frozen: false,
            observer: MappingObserver::default(),
        }
    }
    pub fn observer(&self) -> MappingObserver {
        self.observer.clone()
    }
    pub fn map_zeroed(&mut self, address: GuestAddress, size: u64) -> Result<(), MemoryError> {
        if self.frozen {
            return Err(MemoryError::Finalized);
        }
        let end = address.0.checked_add(size).ok_or(MemoryError::Overflow)?;
        let requested = self.used.checked_add(size).ok_or(MemoryError::Overflow)?;
        if requested > self.maximum {
            return Err(MemoryError::Capacity {
                requested,
                maximum: self.maximum,
            });
        }
        if size == 0
            || self
                .regions
                .iter()
                .any(|r| address.0 < r.start + r.bytes.len() as u64 && r.start < end)
        {
            return Err(MemoryError::Overlap);
        }
        let len = usize::try_from(size).map_err(|_| MemoryError::Allocation)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(len)
            .map_err(|_| MemoryError::Allocation)?;
        bytes.resize(len, 0);
        self.regions
            .try_reserve(1)
            .map_err(|_| MemoryError::Allocation)?;
        self.regions.push(Region {
            start: address.0,
            bytes,
        });
        self.used = requested;
        self.observer.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    pub fn write(&mut self, address: GuestAddress, bytes: &[u8]) -> Result<(), MemoryError> {
        if self.frozen {
            return Err(MemoryError::Finalized);
        }
        let end = address
            .0
            .checked_add(bytes.len() as u64)
            .ok_or(MemoryError::Overflow)?;
        let r = self
            .regions
            .iter_mut()
            .find(|r| address.0 >= r.start && end <= r.start + r.bytes.len() as u64)
            .ok_or(MemoryError::Unmapped {
                address: address.0,
                size: bytes.len() as u64,
            })?;
        let offset = (address.0 - r.start) as usize;
        r.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
    /// Diagnostic read of owned bytes, not a guest CPU fetch or a permission bypass for execution.
    pub fn read(&self, address: GuestAddress, size: u64) -> Result<&[u8], MemoryError> {
        let end = address.0.checked_add(size).ok_or(MemoryError::Overflow)?;
        let r = self
            .regions
            .iter()
            .find(|r| address.0 >= r.start && end <= r.start + r.bytes.len() as u64)
            .ok_or(MemoryError::Unmapped {
                address: address.0,
                size,
            })?;
        Ok(&r.bytes[(address.0 - r.start) as usize..(end - r.start) as usize])
    }
    pub fn finalize(&mut self) {
        self.frozen = true;
    }
    pub fn clear(&mut self) {
        self.regions = Vec::new();
        self.used = 0;
        self.frozen = true;
        self.observer.0.store(0, Ordering::SeqCst);
    }
}
impl Drop for OwnedAddressSpace {
    fn drop(&mut self) {
        self.clear();
    }
}
