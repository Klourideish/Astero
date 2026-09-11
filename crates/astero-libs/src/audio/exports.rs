//! Guest AudioOut contracts. Exact library/module keys; no host FILE/device handles.
use astero_audio::output::service::{AudioService, Config, Error, Kind};
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use std::sync::Arc;
pub const LEGACY: &[(&str, u64)] = &[
    ("Init", 0x25f10f5d5c6116a0),
    ("Open", 0x7a436fb13db6aec6),
    ("Close", 0xb35fffb84f66045c),
    ("Output", 0x40e42d6de0eab13e),
    ("Outputs", 0xc373dd6924d2c061),
    ("SetVolume", 0x6feb8057cf489711),
    ("GetPortState", 0x1ab43db3822b35a4),
];
pub const AUDIO2: &[(&str, u64)] = &[
    ("Initialize", 0x836b558852288471),
    ("ContextResetParam", 0xb7962b8b3b9fa507),
    ("ContextQueryMemory", 0xa439a67bb0609ba1),
    ("ContextCreate", 0xd31ea8d555406126),
    ("ContextDestroy", 0xa27e991fb01ba35d),
    ("ContextAdvance", 0x3c4db31cca8b487b),
    ("ContextPush", 0x68823d8799e58bd5),
    ("UserCreate", 0xc72c1871107b9db4),
    ("UserDestroy", 0x21a65727d33bf6ea),
    ("PortCreate", 0x24adb06a664fcf03),
    ("PortSetAttributes", 0xf174c0ad23f25879),
    ("PortDestroy", 0x71df91b70f83d71f),
    ("PortGetState", 0x81ab4450a1be11ae),
    ("ContextGetQueueLevel", 0x47b774175836aac5),
    ("GetSpeakerArrayMemorySize", 0x1b560e2832585f66),
    ("GetSystemState", 0x6e404df8230bc117),
    ("MasteringInit", 0x5c7977f193649dbb),
    ("ContextSetAttributes", 0xe1dab6adb956960d),
];
pub fn key(nid: u64, two: bool) -> ProviderKey {
    ProviderKey {
        nid,
        library: if two {
            b"libSceAudioOut2".to_vec()
        } else {
            b"libSceAudioOut".to_vec()
        },
        module: b"libSceAudioOut".to_vec(),
    }
}
#[derive(Debug)]
enum Failure {
    Service(Error),
    Memory(AccessError),
    Unsupported,
}
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self::Service(e)
    }
}
impl From<AccessError> for Failure {
    fn from(e: AccessError) -> Self {
        Self::Memory(e)
    }
}
type Result<T> = std::result::Result<T, Failure>;
fn read(m: &dyn GuestMemory, a: u64, n: u64) -> Result<Vec<u8>> {
    if a == 0 {
        return Err(AccessError::Range.into());
    }
    m.charge(n)?;
    m.validate(a, n, false)?;
    Ok(m.read(a, n)?)
}
fn preflight(m: &dyn GuestMemory, a: u64, n: u64) -> Result<()> {
    if a == 0 {
        return Err(AccessError::Range.into());
    }
    m.charge(n)?;
    m.validate(a, n, true)?;
    Ok(())
}
fn write(m: &mut dyn GuestMemory, a: u64, b: &[u8]) -> Result<()> {
    preflight(m, a, b.len() as u64)?;
    m.write(a, b)?;
    Ok(())
}
fn u32at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().expect("checked structure"))
}
fn publish(
    s: &AudioService,
    m: &mut dyn GuestMemory,
    out: u64,
    kind: Kind,
    parent: Option<u64>,
    c: Config,
) -> Result<u64> {
    preflight(m, out, 8)?;
    let id = s.create(kind, parent, c)?;
    if let Err(e) = m.write(out, &id.to_le_bytes()) {
        let _ = s.close(id, kind);
        return Err(e.into());
    }
    Ok(0)
}
fn legacy(name: &str, a: [u64; 6], m: &mut dyn GuestMemory, s: &AudioService) -> Result<u64> {
    match name {
        "Init" => {
            s.initialize()?;
            Ok(0)
        }
        "Open" => {
            if a[2] != 0
                || a[3] > u32::MAX as u64
                || a[4] > u32::MAX as u64
                || a[5] > u32::MAX as u64
            {
                return Err(Error::Invalid.into());
            }
            Ok(s.create(
                Kind::Legacy,
                None,
                Config::pcm(a[3] as u32, a[4] as u32, a[5] as u32, a[1] as u32)?,
            )?)
        }
        "Close" => {
            s.close(a[0], Kind::Legacy)?;
            Ok(0)
        }
        "Output" => {
            let p = s.record(a[0], Kind::Legacy)?;
            if a[1] == 0 {
                return Ok(0);
            }
            let pcm = read(m, a[1], p.config.bytes() as u64)?;
            s.submit(vec![(a[0], pcm)], Kind::Legacy, true)?;
            Ok(p.config.frames as u64)
        }
        "Outputs" => {
            if !(1..=25).contains(&a[1]) {
                return Err(Error::Invalid.into());
            }
            let b = read(m, a[0], a[1] * 16)?;
            let mut items = vec![];
            let mut grain = 0;
            for e in b.chunks_exact(16) {
                let id = u32at(e, 0) as u64;
                let addr = u64::from_le_bytes(e[8..16].try_into().unwrap());
                let p = s.record(id, Kind::Legacy)?;
                if grain == 0 {
                    grain = p.config.frames as u64
                }
                if addr != 0 {
                    items.push((id, read(m, addr, p.config.bytes() as u64)?));
                }
            }
            if !items.is_empty() {
                s.submit(items, Kind::Legacy, true)?
            }
            Ok(grain)
        }
        "SetVolume" => {
            let b = read(m, a[2], 32)?;
            let mut v = [0; 8];
            for (i, x) in v.iter_mut().enumerate() {
                *x = u32at(&b, i * 4) as i32;
            }
            s.volume(a[0], a[1] as u32, v)?;
            Ok(0)
        }
        "GetPortState" => {
            let p = s.record(a[0], Kind::Legacy)?;
            let mut b = [0u8; 32];
            let (output, channels): (u16, u8) = match p.config.port_type {
                2 | 3 => (0x40, 1),
                4 | 10 => (4, 1),
                127 => (0x80, 0),
                _ => (1, p.config.channels.min(2)),
            };
            b[..2].copy_from_slice(&output.to_le_bytes());
            b[2] = channels;
            b[4..6].copy_from_slice(&(p.volume[0] as i16).to_le_bytes());
            write(m, a[1], &b)?;
            Ok(0)
        }
        _ => Err(Failure::Unsupported),
    }
}
fn audio2(name: &str, a: [u64; 6], m: &mut dyn GuestMemory, s: &AudioService) -> Result<u64> {
    match name {
        "Initialize" => {
            s.initialize()?;
            Ok(0)
        }
        "MasteringInit" => {
            if a[0] != 0 {
                return Err(Failure::Unsupported);
            }
            s.mastering()?;
            Ok(0)
        }
        "ContextResetParam" => {
            let mut b = [0u8; 64];
            for (i, v) in [(0, 256u32), (4, 256), (12, 4), (16, 512), (20, 1)] {
                b[i..i + 4].copy_from_slice(&v.to_le_bytes());
            }
            write(m, a[0], &b)?;
            Ok(0)
        }
        "ContextQueryMemory" => {
            if a[0] != 0 {
                read(m, a[0], 64)?;
            }
            write(m, a[1], &0x100000u64.to_le_bytes())?;
            Ok(0)
        }
        "GetSpeakerArrayMemorySize" => {
            if !(1..=32).contains(&a[0]) {
                return Err(Error::Invalid.into());
            }
            Ok(0x400
                + a[0] * if a[2] != 0 { 0x100 } else { 0x40 }
                + if a[1] != 0 { 0x200 } else { 0 })
        }
        "GetSystemState" => {
            write(m, a[0], &[0; 64])?;
            Ok(0)
        }
        "ContextCreate" => {
            let c = if a[0] == 0 {
                Config::context(256, 4)?
            } else {
                let b = read(m, a[0], 64)?;
                let depth = u32at(&b, 12);
                Config::context(u32at(&b, 16), if depth == 0 { 4 } else { depth as usize })?
            };
            // Work area is guest-owned; validate advertised prototype-sized storage, retain no pointer.
            if a[1] == 0 || a[2] < 0x100000 {
                return Err(Error::Invalid.into());
            }
            m.validate(a[1], 0x100000, true)?;
            publish(s, m, a[3], Kind::Context, None, c)
        }
        "ContextDestroy" => {
            s.close(a[0], Kind::Context)?;
            Ok(0)
        }
        "ContextAdvance" => {
            s.record(a[0], Kind::Context)?;
            Ok(0)
        }
        "ContextPush" => {
            s.submit(vec![(a[0], vec![])], Kind::Context, a[1] != 0)?;
            Ok(0)
        }
        "ContextGetQueueLevel" => {
            if a[1] != 0 {
                preflight(m, a[1], 4)?
            }
            if a[2] != 0 {
                preflight(m, a[2], 4)?
            }
            let p = s.record(a[0], Kind::Context)?;
            if a[1] != 0 {
                write(m, a[1], &(p.pending as u32).to_le_bytes())?
            }
            if a[2] != 0 {
                write(
                    m,
                    a[2],
                    &((p.config.depth - p.pending) as u32).to_le_bytes(),
                )?
            }
            Ok(0)
        }
        "UserCreate" => publish(s, m, a[1], Kind::User, None, Config::context(256, 1)?),
        "UserDestroy" => {
            s.close(a[0], Kind::User)?;
            Ok(0)
        }
        "PortCreate" => {
            let c = s.record(a[0], Kind::Context)?.config;
            let b = read(m, a[1], 16)?;
            let rate = u32at(&b, 8);
            if rate != 0 && rate != 48000 {
                return Err(Error::Invalid.into());
            }
            publish(s, m, a[2], Kind::Port, Some(a[0]), c)
        }
        "PortDestroy" => {
            s.close(a[0], Kind::Port)?;
            Ok(0)
        }
        "PortGetState" => {
            let p = s.record(a[0], Kind::Port)?;
            let mut b = [0u8; 64];
            b[..2].copy_from_slice(&1u16.to_le_bytes());
            b[2] = 2;
            b[4..6].copy_from_slice(&127i16.to_le_bytes());
            b[8..12].copy_from_slice(&(p.submitted as u32).to_le_bytes());
            write(m, a[1], &b)?;
            Ok(0)
        }
        "ContextSetAttributes" | "PortSetAttributes" => {
            s.record(
                a[0],
                if name.starts_with("Port") {
                    Kind::Port
                } else {
                    Kind::Context
                },
            )?;
            if a[2] != 0 {
                return Err(Failure::Unsupported);
            }
            Ok(0)
        }
        _ => Err(Failure::Unsupported),
    }
}
pub fn registrations(service: Arc<AudioService>) -> Vec<Registration> {
    [(false, LEGACY), (true, AUDIO2)]
        .into_iter()
        .flat_map(|(two, entries)| {
            let service = service.clone();
            entries.iter().map(move |(name, nid)| {
                let s = service.clone();
                Registration {
                    key: key(*nid, two),
                    kind: ProviderKind::HleImplementation,
                    handler: Some(Box::new(move |f, m| {
                        let r = if two {
                            audio2(name, f.arguments, m, &s)
                        } else {
                            legacy(name, f.arguments, m, &s)
                        };
                        match r {
                            Ok(v) => {
                                f.rax = v;
                                CallResult::Returned
                            }
                            Err(Failure::Unsupported) => CallResult::Unsupported,
                            Err(Failure::Memory(e)) => CallResult::AccessFailure(e),
                            Err(Failure::Service(Error::Interrupted)) => CallResult::StopRequested,
                            Err(Failure::Service(e)) => {
                                let code = if two {
                                    if e == Error::Busy {
                                        0x80268008u32
                                    } else {
                                        0x80268001
                                    }
                                } else {
                                    match e {
                                        Error::Handle => 0x80260003,
                                        Error::Capacity => 0x80260005,
                                        _ => 0x80260001,
                                    }
                                };
                                f.rax = (code as i32 as i64) as u64;
                                CallResult::Returned
                            }
                        }
                    })),
                }
            })
        })
        .collect()
}
