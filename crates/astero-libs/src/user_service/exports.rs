//! PS5Rust single-local-user policy adapted through checked guest copies.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::process::users::{Error, Users};
use std::sync::{Arc, Mutex};
pub const USER_ID: i32 = 0x10000000;
pub const NAME: &[u8] = b"Player\0";
/// Complete coherent prototype cluster. No NID-only fallback.
pub const EXPORTS: &[(&str, u64)] = &[
    ("sceUserServiceInitialize", 0x8F760CBB531534DA),
    ("sceUserServiceGetInitialUser", 0x09D5A9D281D61ABD),
    ("sceUserServiceGetAgeLevel", 0xC28369BBEE3944B9),
    ("sceUserServiceGetGamePresets", 0xFEC0F4DA6143061E),
    (
        "sceUserServiceGetAccessibilityChatTranscription",
        0xAE71211EA1BFE31A,
    ),
    (
        "sceUserServiceGetAccessibilityPressAndHoldDelay",
        0x64A26DC5D82FCF08,
    ),
    (
        "sceUserServiceGetAccessibilityVibration",
        0xA96607385C2A0B16,
    ),
    (
        "sceUserServiceGetAccessibilityTriggerEffect",
        0xFF763918EFBF8BBF,
    ),
    (
        "sceUserServiceGetAccessibilityZoomEnabled",
        0x843FC7F3510DF558,
    ),
    (
        "sceUserServiceGetAccessibilityZoomFollowFocus",
        0x3BA216D7F0F09BFC,
    ),
    ("sceUserServiceGetLoginUserIdList", 0x7CF87298A36F2BF0),
    ("sceUserServiceGetUserName", 0xD71C5C3221AED9FA),
    ("sceUserServiceGetEvent", 0xC87D7B43A356B558),
    ("sceUserServiceLogout", 0xDD3F72E710DC7CE9),
    ("sceUserServiceTerminate", 0x6F01634BE6D7F660),
];
pub fn key(nid: u64) -> ProviderKey {
    ProviderKey {
        nid,
        library: b"libSceUserService".to_vec(),
        module: b"libSceUserService".to_vec(),
    }
}
#[derive(Debug)]
enum Failure {
    State(Error),
    Access(AccessError),
    Argument,
}
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self::State(e)
    }
}
impl From<AccessError> for Failure {
    fn from(e: AccessError) -> Self {
        Self::Access(e)
    }
}
fn output(m: &mut dyn GuestMemory, a: u64, b: &[u8]) -> Result<(), Failure> {
    if a == 0 {
        return Err(Failure::Argument);
    }
    m.charge(b.len() as u64)?;
    m.validate(a, b.len() as u64, true)?;
    m.write(a, b)?;
    Ok(())
}
fn invoke(i: usize, a: [u64; 6], m: &mut dyn GuestMemory, u: &mut Users) -> Result<(), Failure> {
    if i == 0 {
        if a[0] != 0 {
            m.charge(4)?;
            m.validate(a[0], 4, false)?;
            let bytes = m.read(a[0], 4)?;
            let priority = i32::from_le_bytes(bytes.try_into().map_err(|_| Failure::Argument)?);
            if !(0..=1023).contains(&priority) {
                return Err(Failure::Argument);
            }
        }
        u.initialize();
        return Ok(());
    }
    if i == 14 {
        u.terminate();
        return Ok(());
    }
    u.require_initialized()?;
    match i {
        1 => output(m, a[0], &USER_ID.to_le_bytes()),
        10 => {
            let s = u.snapshot();
            let mut bytes = [0xff; 16];
            if s.logged_in {
                bytes[..4].copy_from_slice(&s.user.to_le_bytes());
            }
            output(m, a[0], &bytes)
        }
        12 => {
            let e = u.event()?;
            let mut b = [0; 8];
            b[..4].copy_from_slice(&(if e.login { 0i32 } else { 1i32 }).to_le_bytes());
            b[4..].copy_from_slice(&e.user.to_le_bytes());
            output(m, a[0], &b)?;
            u.consume_event()?;
            Ok(())
        }
        13 => {
            u.logout(a[0] as i32)?;
            Ok(())
        }
        _ => {
            u.validate_user(a[0] as i32)?;
            match i {
                11 => {
                    if a[2] < NAME.len() as u64 {
                        return Err(Failure::Access(AccessError::Limit));
                    }
                    output(m, a[1], NAME)
                }
                3 => {
                    if a[1] == 0 {
                        return Err(Failure::Argument);
                    }
                    m.charge(8)?;
                    m.validate(a[1], 8, false)?;
                    let bytes = m.read(a[1], 8)?;
                    let n = u64::from_le_bytes(bytes.try_into().map_err(|_| Failure::Argument)?);
                    if n < 40 {
                        return Err(Failure::Access(AccessError::Limit));
                    }
                    let mut b = [0; 40];
                    b[..8].copy_from_slice(&40u64.to_le_bytes());
                    output(m, a[1], &b)
                }
                _ => {
                    let value: i32 = match i {
                        2 => 18,
                        6 | 7 => 1,
                        _ => 0,
                    };
                    output(m, a[1], &value.to_le_bytes())
                }
            }
        }
    }
}
pub fn registrations(users: Arc<Mutex<Users>>) -> Vec<Registration> {
    EXPORTS
        .iter()
        .enumerate()
        .map(|(i, (_, nid))| {
            let users = users.clone();
            Registration {
                key: key(*nid),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    let result = invoke(
                        i,
                        f.arguments,
                        m,
                        &mut users.lock().unwrap_or_else(|p| p.into_inner()),
                    );
                    match result {
                        Ok(()) => {
                            f.rax = 0;
                            CallResult::Returned
                        }
                        Err(Failure::Access(e)) => CallResult::AccessFailure(e),
                        Err(e) => {
                            f.rax = match e {
                                Failure::State(Error::NotInitialized) => 0x80960002,
                                Failure::State(Error::NoEvent) => 0x80960007,
                                Failure::State(Error::Capacity) => {
                                    return CallResult::AccessFailure(AccessError::Limit);
                                }
                                _ => 0x80960005,
                            };
                            CallResult::Returned
                        }
                    }
                })),
            }
        })
        .collect()
}
