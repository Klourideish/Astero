//! Narrow M30 libc startup surface. _init_env is explicitly an experimental no-op.
use astero_hle::dispatch::prepared::{CallResult, ProviderKey, ProviderKind, Registration};
use astero_kernel::process::exit_callbacks::{CallbackTarget, ExitCallbacks};
use std::{cell::RefCell, rc::Rc};
pub const INIT_ENV_NID: u64 = 0x6f3404c72d7cf592;
pub const ATEXIT_NID: u64 = 0xf06d8b07e037af38;
pub struct StartupState {
    pub init_env_calls: u64,
    pub atexit_calls: u64,
    callbacks: ExitCallbacks,
}
impl StartupState {
    pub fn records(
        &self,
    ) -> impl Iterator<Item = &astero_kernel::process::exit_callbacks::CallbackRecord> {
        self.callbacks.records()
    }
    pub fn callbacks(&self) -> impl Iterator<Item = u64> + '_ {
        self.callbacks.pending()
    }
}
/// Concrete artifact-correlated libc/libc context, not SOURCE_OWNER or numeric-ID fallback.
pub fn registrations(
    max_callbacks: usize,
    executable_ranges: Vec<(u64, u64)>,
) -> (Vec<Registration>, Rc<RefCell<StartupState>>) {
    owned_registrations(max_callbacks, executable_ranges, None)
}
/// Exact runtime landing is borrowed from the enclosing bridge owner, retained through teardown.
/// It is a distinct callback identity and is never invoked through a guest callback path.
pub fn owned_registrations(
    max_callbacks: usize,
    executable_ranges: Vec<(u64, u64)>,
    return_landing: Option<u64>,
) -> (Vec<Registration>, Rc<RefCell<StartupState>>) {
    let state = Rc::new(RefCell::new(StartupState {
        init_env_calls: 0,
        atexit_calls: 0,
        callbacks: ExitCallbacks::new(max_callbacks),
    }));
    let init = state.clone();
    let exit = state.clone();
    let key = |nid| ProviderKey {
        nid,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    (
        vec![
            Registration {
                key: key(INIT_ENV_NID),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |c, _| {
                    let mut s = init.borrow_mut();
                    s.init_env_calls = s.init_env_calls.saturating_add(1);
                    c.rax = 0;
                    CallResult::Returned
                })),
            },
            Registration {
                key: key(ATEXIT_NID),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |c, _| {
                    let mut s = exit.borrow_mut();
                    s.atexit_calls = s.atexit_calls.saturating_add(1);
                    let address = c.arguments[0];
                    let executable = executable_ranges
                        .iter()
                        .any(|&(a, n)| address >= a && address - a < n);
                    let target = if return_landing == Some(address) {
                        Some(CallbackTarget::RuntimeReturn(address))
                    } else if executable {
                        Some(CallbackTarget::Guest(address))
                    } else {
                        None
                    };
                    c.rax = if target.is_some_and(|t| s.callbacks.register_target(t).is_ok()) {
                        0
                    } else {
                        u32::MAX as u64
                    };
                    CallResult::Returned
                })),
            },
        ],
        state,
    )
}

/// PS5Rust deterministic process guard policy; not a firmware security value.
pub const STACK_GUARD_VALUE: u64 = 0x9e3779b97f4a7c15;
pub fn stack_guard(address: u64) -> astero_hle::providers::data::DataExport {
    astero_hle::providers::data::DataExport {
        key: ProviderKey {
            nid: 0x7fbb8ec58f663355,
            library: b"libkernel".to_vec(),
            module: b"libkernel".to_vec(),
        },
        address,
        size: 8,
        read_only: true,
    }
}

pub const CXA_ATEXIT_NID: u64 = 0xb6cbc49a77a7cf8f;
/// Bounded DSO/argument retention only; no callback invocation or unwind support.
pub fn cxa_registration(
    state: Rc<RefCell<StartupState>>,
    executable: Vec<(u64, u64)>,
) -> Registration {
    Registration {
        key: ProviderKey {
            nid: CXA_ATEXIT_NID,
            library: b"libc".to_vec(),
            module: b"libc".to_vec(),
        },
        kind: ProviderKind::HleImplementation,
        handler: Some(Box::new(move |f, _| {
            let [address, argument, dso, ..] = f.arguments;
            if address == 0 {
                f.rax = 0;
                return CallResult::Returned;
            }
            if !executable
                .iter()
                .any(|&(a, n)| address >= a && address - a < n)
            {
                return CallResult::Unsupported;
            }
            let r = astero_kernel::process::exit_callbacks::CallbackRecord {
                target: CallbackTarget::Guest(address),
                argument: Some(argument),
                dso: Some(dso),
            };
            f.rax = if state.borrow_mut().callbacks.register_record(r).is_ok() {
                0
            } else {
                u32::MAX as u64
            };
            CallResult::Returned
        })),
    }
}
