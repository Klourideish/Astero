//! Exact libc/libc Itanium-style guard contracts adapted from PS5Rust.
use astero_hle::{calls::memory::GuestMemory, dispatch::prepared::*};
use astero_kernel::{process::guards::Guards, synchronization::owned::Thread};
use std::sync::Arc;
pub const EXPORTS: &[(&str, u64)] = &[
    ("__cxa_guard_acquire", 0xDC63E98D0740313C),
    ("__cxa_guard_release", 0xF6B01E00D4F6B721),
    ("__cxa_guard_abort", 0xD9E99A6A5B96CD4C),
    ("__cxa_pure_virtual", 0xCEBD3DE04437F56C),
];
pub fn registrations(guards: Arc<Guards>, thread: Thread) -> Vec<Registration> {
    EXPORTS
        .iter()
        .enumerate()
        .map(|(op, (_, nid))| {
            let guards = guards.clone();
            Registration {
                key: ProviderKey {
                    nid: *nid,
                    library: b"libc".to_vec(),
                    module: b"libc".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match call(&guards, thread, op, f.arguments[0], m) {
                        Ok(value) => {
                            f.rax = value;
                            CallResult::Returned
                        }
                        Err(e) => e,
                    }
                })),
            }
        })
        .collect()
}
fn call(
    g: &Guards,
    t: Thread,
    op: usize,
    a: u64,
    m: &mut dyn GuestMemory,
) -> Result<u64, CallResult> {
    if op == 3 {
        return Err(CallResult::StopRequested);
    }
    if a == 0 || !a.is_multiple_of(8) {
        return Err(CallResult::Unsupported);
    }
    m.validate(a, 8, true).map_err(CallResult::AccessFailure)?;
    m.validate(a, 8, false).map_err(CallResult::AccessFailure)?;
    if op == 0 {
        g.acquire(a, t).map_err(|_| CallResult::StopRequested)?;
        let bytes = match m.read(a, 8) {
            Ok(b) => b,
            Err(e) => {
                let _ = g.release(a, t);
                return Err(CallResult::AccessFailure(e));
            }
        };
        if bytes[0] & 1 != 0 {
            g.release(a, t).map_err(|_| CallResult::StopRequested)?;
            Ok(0)
        } else {
            Ok(1)
        }
    } else {
        g.require_owner(a, t).map_err(|_| CallResult::Unsupported)?;
        if op == 1 {
            // Publish the completion byte only, preserving all seven ABI-reserved bytes.
            let mut byte = m.read(a, 1).map_err(CallResult::AccessFailure)?;
            byte[0] |= 1;
            m.write(a, &byte).map_err(CallResult::AccessFailure)?;
        }
        g.release(a, t).map_err(|_| CallResult::StopRequested)?;
        Ok(0)
    }
}
