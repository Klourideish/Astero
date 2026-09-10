//! Per-runtime recovery coordination contract. No VEH or assembly is installed in M29.
//! Native adapter must capture host stack/nonvolatiles/FS before entering and switch back
//! to the host stack before calling Rust on a guest fault. A model is not an armed adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Requested,
    Returned,
    Fault { code: u32, rip: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryState {
    Prepared,
    Stopped(StopReason),
    Released,
}
pub struct RecoveryBoundary {
    state: BoundaryState,
}
impl Default for RecoveryBoundary {
    fn default() -> Self {
        Self {
            state: BoundaryState::Prepared,
        }
    }
}
impl RecoveryBoundary {
    pub fn state(&self) -> BoundaryState {
        self.state
    }
    pub fn native_installed(&self) -> bool {
        false
    }
    /// Host-only first-stop-wins instrumentation. Never handles a Windows exception.
    pub fn record_stop(&mut self, reason: StopReason) -> bool {
        if self.state != BoundaryState::Prepared {
            return false;
        }
        self.state = BoundaryState::Stopped(reason);
        true
    }
    pub fn release(&mut self) {
        self.state = BoundaryState::Released;
    }
}
impl Drop for RecoveryBoundary {
    fn drop(&mut self) {
        self.release();
    }
}
/// Planned legacy 24-byte import encoding. No executable mapping/call is created.
/// push imm32 sign-extends, so reject indices outside signed positive range.
pub fn import_stub(index: u32, landing: u64) -> Option<[u8; 24]> {
    if index > i32::MAX as u32 || landing == 0 {
        return None;
    }
    let mut b = [0x90; 24];
    b[0] = 0x68;
    b[1..5].copy_from_slice(&index.to_le_bytes());
    b[5] = 0xff;
    b[6] = 0x25;
    b[7..11].fill(0);
    b[11..19].copy_from_slice(&landing.to_le_bytes());
    Some(b)
}
