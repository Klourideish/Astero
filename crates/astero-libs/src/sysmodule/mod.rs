//! Exact sysmodule request adapters. Unknown/unbacked requests fail, never fabricate a PRX.
use astero_hle::{
    dispatch::prepared::*,
    providers::modules::{Error, Modules},
};
use std::sync::Arc;
pub const EXPORTS: &[(&str, u64, u8)] = &[
    ("sceSysmoduleLoadModule", 0x83C70CDFD11467AA, 0),
    ("sceSysmoduleUnloadModule", 0x791D9B6450005344, 1),
    ("sceSysmoduleIsLoaded", 0x7CC3F934750E68C9, 2),
    ("sceSysmoduleLoadModuleInternal", 0xDFD895E44D47A029, 0),
    (
        "sceSysmoduleLoadModuleInternalWithArg",
        0x847AC6A06A0D7FEB,
        3,
    ),
    ("sceSysmoduleUnloadModuleInternal", 0xBD7661AED2719067, 1),
    ("sceSysmoduleIsLoadedInternal", 0xCA714A4396DF1A4B, 2),
];
pub fn registrations(owner: Arc<Modules>) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|&(_, nid, op)| {
            let owner = owner.clone();
            Registration {
                key: ProviderKey {
                    nid,
                    library: b"libSceSysmodule".to_vec(),
                    module: b"libSceSysmodule".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    let a = f.arguments;
                    if a[0] > u16::MAX as u64 {
                        f.rax = 0x80020003;
                        return CallResult::Returned;
                    }
                    let id = a[0] as u16;
                    if op == 3
                        && (a[1] != 0
                            || a[2] != 0
                            || a[3] != 0
                            || a[4] == 0
                            || m.validate(a[4], 4, true).is_err())
                    {
                        f.rax = 0x80020003;
                        return CallResult::Returned;
                    }
                    let result = match op {
                        0 | 3 => owner.load(id),
                        1 => owner.unload(id),
                        _ => {
                            if owner.is_loaded(id) {
                                Ok(())
                            } else {
                                Err(Error::Unknown)
                            }
                        }
                    };
                    if result.is_ok()
                        && op == 3
                        && let Err(e) = m.write(a[4], &0i32.to_le_bytes())
                    {
                        let _ = owner.unload(id);
                        return CallResult::AccessFailure(e);
                    }
                    f.rax = match result {
                        Ok(()) => 0,
                        Err(Error::Busy) => 0x80020010,
                        Err(_) => 0x80020002,
                    };
                    CallResult::Returned
                })),
            }
        })
        .collect()
}
