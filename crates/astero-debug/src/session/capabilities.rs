//! Capability discovery describes actual support, not optimistic placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    SessionInspection,
    Threads,
    MemoryMaps,
    ModuleNidAttribution,
    GuestFaultCapture,
    Tracing,
    ActualPc,
    GuestSnapshots,
    ExecutionControl,
    Breakpoints,
    Watchpoints,
    GpuVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    Implemented,
    Unsupported,
}

pub const INVENTORY: &[(Capability, Support)] = &[
    (Capability::SessionInspection, Support::Implemented),
    (Capability::Threads, Support::Unsupported),
    (Capability::MemoryMaps, Support::Unsupported),
    (Capability::ModuleNidAttribution, Support::Unsupported),
    (Capability::GuestFaultCapture, Support::Unsupported),
    (Capability::Tracing, Support::Unsupported),
    (Capability::ActualPc, Support::Unsupported),
    (Capability::GuestSnapshots, Support::Unsupported),
    (Capability::ExecutionControl, Support::Unsupported),
    (Capability::Breakpoints, Support::Unsupported),
    (Capability::Watchpoints, Support::Unsupported),
    (Capability::GpuVisibility, Support::Unsupported),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsupported {
    pub capability: Capability,
}

/// A support query only; success does not execute an operation.
pub fn require_support(capability: Capability) -> Result<(), Unsupported> {
    match capability {
        Capability::SessionInspection => Ok(()),
        _ => Err(Unsupported { capability }),
    }
}
