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
    pub peak_bytes: u64,
    pub peak_live: usize,
    pub arena_bytes: u64,
}
pub struct GuestHeap {
    free: Vec<(u64, u64)>,
    live: Vec<(u64, u64, u64)>,
    maximum: usize,
    allocations: u64,
    frees: u64,
    peak_bytes: u64,
    peak_live: usize,
    arena_bytes: u64,
}
impl GuestHeap {
    pub fn new(base: u64, size: u64, maximum: usize) -> Result<Self, HeapError> {
        if base == 0 || !base.is_multiple_of(16) || size == 0 || base.checked_add(size).is_none() {
            return Err(HeapError::Geometry);
        }
        let mut free = Vec::new();
        free.try_reserve_exact(maximum.checked_add(2).ok_or(HeapError::Capacity)?)
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
            peak_bytes: 0,
            peak_live: 0,
            arena_bytes: size,
        })
    }
    pub fn allocate(&mut self, size: u64) -> Result<u64, HeapError> {
        self.allocate_aligned(size, 16)
    }
    pub fn allocate_aligned(&mut self, size: u64, alignment: u64) -> Result<u64, HeapError> {
        if alignment == 0 || !alignment.is_power_of_two() {
            return Err(HeapError::Geometry);
        }
        let alignment = alignment.max(16);
        let extent = size.max(1).checked_add(15).ok_or(HeapError::Capacity)? & !15;
        if self.live.len() == self.maximum {
            return Err(HeapError::Capacity);
        }
        let (i, address) = self
            .free
            .iter()
            .enumerate()
            .find_map(|(i, (a, n))| {
                let aligned = a.checked_add(alignment - 1)? & !(alignment - 1);
                (aligned.checked_add(extent)? <= a.checked_add(*n)?).then_some((i, aligned))
            })
            .ok_or(HeapError::Capacity)?;
        let (base, n) = self.free.remove(i);
        if address > base {
            self.free.push((base, address - base));
        }
        if address + extent < base + n {
            self.free
                .push((address + extent, base + n - address - extent));
        }
        self.free.sort_unstable();
        self.live.push((address, extent, size));
        self.peak_live = self.peak_live.max(self.live.len());
        self.peak_bytes = self
            .peak_bytes
            .max(self.live.iter().map(|(_, n, _)| *n).sum());
        self.allocations = self.allocations.saturating_add(1);
        Ok(address)
    }
    pub fn usable_size(&self, address: u64) -> Result<u64, HeapError> {
        self.live
            .iter()
            .find(|(a, _, _)| *a == address)
            .map(|(_, n, _)| *n)
            .ok_or(HeapError::InvalidFree)
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
            peak_bytes: self.peak_bytes,
            peak_live: self.peak_live,
            arena_bytes: self.arena_bytes,
        }
    }
}
