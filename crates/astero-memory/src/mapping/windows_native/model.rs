use crate::mapping::GuestAddress;
use std::sync::{
    Arc,
    atomic::{AtomicU32, AtomicU64, Ordering},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeLimits {
    pub max_reserved_bytes: u64,
    pub max_committed_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub page_size: u64,
    pub allocation_granularity: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Protection {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuestRange {
    pub start: GuestAddress,
    pub size: u64,
}
pub struct NativeRegion<'a> {
    pub range: GuestRange,
    pub bytes: &'a [u8],
    pub protection: Protection,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePage {
    pub range: GuestRange,
    pub protection: Protection,
    pub widened: bool,
}
#[derive(Debug)]
pub struct NativeLayout {
    pub(super) envelope: GuestRange,
    pub(super) pages: Vec<NativePage>,
    pub(super) geometry: Geometry,
}
impl NativeLayout {
    pub fn envelope(&self) -> GuestRange {
        self.envelope
    }
    pub fn pages(&self) -> &[NativePage] {
        &self.pages
    }
    pub fn geometry(&self) -> Geometry {
        self.geometry
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeError {
    ReservationRefused {
        code: u32,
        evidence: Box<ReservationEvidence>,
    },
    SharedOwnership,
    UnsupportedHost,
    Geometry,
    Empty,
    Overflow,
    Overlap {
        index: usize,
    },
    SourceSize {
        index: usize,
    },
    Budget {
        kind: &'static str,
        required: u64,
        maximum: u64,
    },
    Allocation,
    WritableExecutable {
        address: u64,
    },
    Os {
        operation: &'static str,
        address: u64,
        size: u64,
        code: u32,
    },
    PlacementMismatch,
    Readback {
        address: u64,
    },
    ProtectionMismatch {
        address: u64,
    },
    Released,
    Unreadable,
}
/// Bounded post-failure observation, not a lease or attribution of host ownership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationEvidence {
    pub requested: GuestRange,
    pub geometry: Geometry,
    pub regions: Vec<ReservationRegion>,
    pub complete: bool,
    pub query_error: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationRegion {
    pub base: u64,
    pub size: u64,
    pub allocation_base: u64,
    pub state: u32,
    pub protection: u32,
    pub kind: u32,
}
impl std::fmt::Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native VM: {self:?}")
    }
}
impl std::error::Error for NativeError {}
#[derive(Clone, Debug, Default)]
pub struct NativeObserver {
    pub(super) active: Arc<AtomicU64>,
    pub(super) failure: Arc<AtomicU32>,
}
impl NativeObserver {
    pub fn active_reservations(&self) -> u64 {
        self.active.load(Ordering::SeqCst)
    }
    pub fn release_error(&self) -> u32 {
        self.failure.load(Ordering::SeqCst)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSnapshot {
    pub guest_envelope: GuestRange,
    pub host_envelope: GuestRange,
    pub geometry: Geometry,
    pub committed_bytes: u64,
    pub copied_bytes: u64,
    pub widened_pages: usize,
    pub active: bool,
    pub readback_before_protection: bool,
    pub protections_verified: bool,
    pub instruction_cache_flushed: bool,
}
