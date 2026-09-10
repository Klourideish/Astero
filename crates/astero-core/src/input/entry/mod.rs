//! Explicit runtime preparation, independent of sessions and offline input operations.
#[cfg(all(windows, target_arch = "x86_64"))]
mod owner;
pub use astero_hle::dispatch::prepared::{
    PreparedRegistry, ProviderKey, ProviderKind, Registration, RegistryError,
};
pub use astero_kernel::execution::preparation::{
    InitialContext,
    layout::{PreparationError, RuntimeLimits, ThreadLayout},
};
#[cfg(all(windows, target_arch = "x86_64"))]
pub use owner::*;

#[cfg(all(windows, target_arch = "x86_64"))]
mod closure;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use closure::*;

#[cfg(all(windows, target_arch = "x86_64"))]
mod traps;
#[cfg(all(windows, target_arch = "x86_64"))]
pub use traps::{ObjectTrap, trap_geometry};
