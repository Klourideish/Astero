//! Narrow M30 libc startup surface. _init_env is explicitly an experimental no-op.
use astero_hle::dispatch::prepared::{CallResult, ProviderKey, ProviderKind, Registration};
use astero_kernel::process::exit_callbacks::ExitCallbacks;
use std::{cell::RefCell, rc::Rc};
pub const INIT_ENV_NID: u64 = 0x6f3404c72d7cf592;
pub const ATEXIT_NID: u64 = 0xf06d8b07e037af38;
pub struct StartupState {
    pub init_env_calls: u64,
    pub atexit_calls: u64,
    callbacks: ExitCallbacks,
}
impl StartupState {
    pub fn callbacks(&self) -> impl Iterator<Item = u64> + '_ {
        self.callbacks.pending()
    }
}
/// Concrete artifact-correlated libc/libc context, not SOURCE_OWNER or numeric-ID fallback.
pub fn registrations(
    max_callbacks: usize,
    executable_ranges: Vec<(u64, u64)>,
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
                handler: Some(Box::new(move |c| {
                    let mut s = init.borrow_mut();
                    s.init_env_calls = s.init_env_calls.saturating_add(1);
                    c.rax = 0;
                    CallResult::Returned
                })),
            },
            Registration {
                key: key(ATEXIT_NID),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |c| {
                    let mut s = exit.borrow_mut();
                    s.atexit_calls = s.atexit_calls.saturating_add(1);
                    let address = c.arguments[0];
                    let executable = executable_ranges
                        .iter()
                        .any(|&(a, n)| address >= a && address - a < n);
                    c.rax = if executable && s.callbacks.register(address).is_ok() {
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
