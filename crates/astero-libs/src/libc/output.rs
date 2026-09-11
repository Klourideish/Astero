//! puts adapted as bounded process diagnostic output; no host FILE pointers.
use astero_hle::{calls::memory::AccessError, dispatch::prepared::*};
use astero_kernel::process::output::{Output, OutputError};
use std::sync::{Arc, Mutex};
pub const PUTS_NID: u64 = 0x610d276afa7e6087;
pub fn registration(output: Arc<Mutex<Output>>) -> Registration {
    Registration {
        key: ProviderKey {
            nid: PUTS_NID,
            library: b"libc".to_vec(),
            module: b"libc".to_vec(),
        },
        kind: ProviderKind::HleImplementation,
        handler: Some(Box::new(move |f, m| {
            let result = (|| {
                let n = super::strings::length(m, f.arguments[0], Some(8192))?;
                if n == 8192 {
                    return Err(AccessError::Limit);
                }
                let b = m.read(f.arguments[0], n)?;
                output
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .append_line(&b)
                    .map_err(|e| match e {
                        OutputError::Capacity => AccessError::Limit,
                        OutputError::Allocation => AccessError::Allocation,
                    })?;
                Ok(())
            })();
            match result {
                Ok(()) => {
                    f.rax = 0;
                    CallResult::Returned
                }
                Err(e) => CallResult::AccessFailure(e),
            }
        })),
    }
}
