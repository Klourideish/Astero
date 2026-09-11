//! Exact AJM lifecycle exports; successful bookkeeping does not imply decoding support.
use astero_audio::codecs::ajm::{Ajm, Error};
use astero_hle::{calls::memory::GuestMemory, dispatch::prepared::*};
use std::sync::Arc;
pub const EXPORTS: &[(&str, u64)] = &[
    ("sceAjmInitialize", 0x765FB87874B352EE),
    ("sceAjmFinalize", 0x307BABEAA0AC52EB),
    ("sceAjmModuleRegister", 0x43777216EC069FAE),
    ("sceAjmModuleUnregister", 0x5A2EC3B652D5F8A2),
    ("sceAjmMemoryRegister", 0x6E44471181BA9443),
    ("sceAjmMemoryUnregister", 0xA48A4689A6241E43),
    ("sceAjmInstanceCreate", 0x031A03AC8369E09F),
    ("sceAjmInstanceDestroy", 0x45B2DBB8ABFCCE1A),
    ("sceAjmDecAt9ParseConfigData", 0xD6DDE2C58357CAE7),
];
pub fn registrations(service: Arc<Ajm>) -> Vec<Registration> {
    EXPORTS
        .iter()
        .enumerate()
        .map(|(op, (_, nid))| {
            let service = service.clone();
            Registration {
                key: ProviderKey {
                    nid: *nid,
                    library: b"libSceAjm".to_vec(),
                    module: b"libSceAjm".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match call(&service, op, f.arguments, m) {
                        Ok(v) => {
                            f.rax = v;
                            CallResult::Returned
                        }
                        Err(e) => e,
                    }
                })),
            }
        })
        .collect()
}
fn result(r: Result<(), Error>) -> Result<u64, CallResult> {
    match r {
        Ok(()) => Ok(0),
        Err(Error::InvalidContext) => Ok(0xffff_ffff_8093_0002),
        Err(Error::InvalidParameter) => Ok(0xffff_ffff_8093_0005),
        Err(Error::Capacity) => Err(CallResult::Unsupported),
        Err(Error::Stopped) => Err(CallResult::StopRequested),
    }
}
fn call(s: &Ajm, op: usize, a: [u64; 6], m: &mut dyn GuestMemory) -> Result<u64, CallResult> {
    if op == 8 {
        m.validate(a[1], 20, true)
            .map_err(CallResult::AccessFailure)?;
        let bytes = m.read(a[0], 4).map_err(CallResult::AccessFailure)?;
        let bytes = bytes.try_into().map_err(|_| CallResult::Unsupported)?;
        let fields = match astero_audio::codecs::atrac9::configuration(bytes) {
            Ok(v) => v,
            Err(e) => return result(Err(e)),
        };
        let mut output = [0; 20];
        for (chunk, value) in output.chunks_exact_mut(4).zip(fields) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        m.write(a[1], &output).map_err(CallResult::AccessFailure)?;
        return Ok(0);
    }
    if op == 0 || op == 6 {
        let output = if op == 0 { a[1] } else { a[3] };
        if output == 0 || (op == 0 && a[0] != 0) {
            return result(Err(Error::InvalidParameter));
        }
        m.validate(output, 4, true)
            .map_err(CallResult::AccessFailure)?;
        let r = if op == 0 {
            s.initialize()
        } else {
            s.create(a[0] as u32, a[1] as u32, a[2])
        };
        let id = match r {
            Ok(id) => id,
            Err(e) => return result(Err(e)),
        };
        if let Err(e) = m.write(output, &id.to_le_bytes()) {
            if op == 0 {
                let _ = s.finalize(id);
            } else {
                let _ = s.destroy(a[0] as u32, id);
            }
            return Err(CallResult::AccessFailure(e));
        }
        return Ok(0);
    }
    if op == 4 {
        m.validate(a[1], a[2], false)
            .map_err(CallResult::AccessFailure)?;
    }
    result(match op {
        1 => s.finalize(a[0] as u32),
        2 | 3 => s.module(a[0] as u32, a[1] as u32, op == 2),
        4 | 5 => s.memory(a[0] as u32, a[1], a[2], op == 4),
        7 => s.destroy(a[0] as u32, a[1] as u32),
        _ => return Err(CallResult::Unsupported),
    })
}
