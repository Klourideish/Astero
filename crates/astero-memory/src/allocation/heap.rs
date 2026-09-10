//! Bounded address allocator for an owned guest arena. No host pointers or mappings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapError {
    Geometry,
    Capacity,
    InvalidFree,
    Allocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeapSnapshot {
    pub live: usize,
    pub live_bytes: u64,
    pub allocations: u64,
    pub frees: u64,
}
pub struct GuestHeap {
    free: Vec<(u64, u64)>,
    live: Vec<(u64, u64, u64)>,
    maximum: usize,
    allocations: u64,
    frees: u64,
}
impl GuestHeap {
    pub fn new(base: u64, size: u64, maximum: usize) -> Result<Self, HeapError> {
        if base == 0 || !base.is_multiple_of(16) || size == 0 || base.checked_add(size).is_none() {
            return Err(HeapError::Geometry);
        }
        let mut free = Vec::new();
        free.try_reserve_exact(maximum.checked_add(1).ok_or(HeapError::Capacity)?)
            .map_err(|_| HeapError::Allocation)?;
        free.push((base, size));
        let mut live = Vec::new();
        live.try_reserve_exact(maximum)
            .map_err(|_| HeapError::Allocation)?;
        Ok(Self {
            free,
            live,
            maximum,
            allocations: 0,
            frees: 0,
        })
    }
    pub fn allocate(&mut self, size: u64) -> Result<u64, HeapError> {
        let extent = size.max(1).checked_add(15).ok_or(HeapError::Capacity)? & !15;
        if self.live.len() == self.maximum {
            return Err(HeapError::Capacity);
        }
        let i = self
            .free
            .iter()
            .position(|(_, n)| *n >= extent)
            .ok_or(HeapError::Capacity)?;
        let (address, n) = self.free[i];
        if n == extent {
            self.free.remove(i);
        } else {
            self.free[i] = (address + extent, n - extent);
        }
        self.live.push((address, extent, size));
        self.allocations = self.allocations.saturating_add(1);
        Ok(address)
    }
    pub fn size(&self, address: u64) -> Result<u64, HeapError> {
        self.live
            .iter()
            .find(|(a, _, _)| *a == address)
            .map(|(_, _, n)| *n)
            .ok_or(HeapError::InvalidFree)
    }
    pub fn free(&mut self, address: u64) -> Result<(), HeapError> {
        if address == 0 {
            return Ok(());
        }
        let i = self
            .live
            .iter()
            .position(|(a, _, _)| *a == address)
            .ok_or(HeapError::InvalidFree)?;
        let (a, n, _) = self.live.remove(i);
        self.free.push((a, n));
        self.free.sort_unstable();
        let mut i = 0;
        while i + 1 < self.free.len() {
            if self.free[i].0 + self.free[i].1 == self.free[i + 1].0 {
                let next = self.free.remove(i + 1);
                self.free[i].1 += next.1;
            } else {
                i += 1;
            }
        }
        self.frees = self.frees.saturating_add(1);
        Ok(())
    }
    pub fn snapshot(&self) -> HeapSnapshot {
        HeapSnapshot {
            live: self.live.len(),
            live_bytes: self.live.iter().map(|(_, n, _)| n).sum(),
            allocations: self.allocations,
            frees: self.frees,
        }
    }
}
