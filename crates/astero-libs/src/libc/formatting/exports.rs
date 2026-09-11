//! Exact printf-family providers and bounded output records. No host FILE or va_list.
use super::{Error, Result, arguments::Arguments, engine};
use astero_hle::{
    calls::memory::{AccessError, COPY_CHUNK_BYTES, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::process::output::Output;
use std::sync::{Arc, Mutex};
pub const EXPORTS: &[(&str, u64)] = &[
    ("vsnprintf", 0x43657E8AABE3802D),
    ("snprintf", 0x78B743C3A974FDB5),
    ("vsprintf", 0x8DBCFD23DBE4AA49),
    ("sprintf", 0xB5C562E528AF17B4),
    ("printf", 0x85CB90803E775313),
    ("vprintf", 0x18CA6FC4F156F76E),
    ("fprintf", 0x7DF7F010B5CD5450),
    ("vfprintf", 0xA43043718EAE2D20),
];
#[derive(Clone, Debug)]
pub struct Observation {
    pub provider: &'static str,
    pub format_address: u64,
    pub destination: u64,
    pub capacity: Option<u64>,
    pub required: usize,
    pub written: usize,
    pub conversions: usize,
    pub truncated: bool,
    pub stream: Option<&'static str>,
    pub error: Option<Error>,
}
pub struct Formatting {
    pub output: Arc<Mutex<Output>>,
    pub stdout: u64,
    pub stderr: u64,
    records: Mutex<Vec<Observation>>,
}
impl Formatting {
    pub fn new(output: Arc<Mutex<Output>>, stdout: u64, stderr: u64) -> Self {
        Self {
            output,
            stdout,
            stderr,
            records: Mutex::new(Vec::new()),
        }
    }
    pub fn snapshot(&self) -> Vec<Observation> {
        self.records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
}
pub fn key(nid: u64) -> ProviderKey {
    ProviderKey {
        nid,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    }
}
fn perform(
    i: usize,
    f: &astero_abi::layouts::entry::CallFrame,
    m: &mut dyn GuestMemory,
    owner: &Formatting,
    obs: &mut Observation,
) -> Result<u64> {
    let a = f.arguments;
    let (format, named, list, dest, cap, stream) = match i {
        0 => (a[2], 3, Some(a[3]), a[0], Some(a[1]), None),
        1 => (a[2], 3, None, a[0], Some(a[1]), None),
        2 => (a[1], 2, Some(a[2]), a[0], None, None),
        3 => (a[1], 2, None, a[0], None, None),
        4 => (a[0], 1, None, 0, None, Some("stdout")),
        5 => (a[0], 1, Some(a[1]), 0, None, Some("stdout")),
        6 | 7 => {
            let stream = if a[0] == owner.stdout && a[0] != 0 {
                "stdout"
            } else if a[0] == owner.stderr && a[0] != 0 {
                "stderr"
            } else {
                return Err(Error::contract("unowned FILE token"));
            };
            (
                a[1],
                2,
                if i == 7 { Some(a[2]) } else { None },
                a[0],
                None,
                Some(stream),
            )
        }
        _ => unreachable!(),
    };
    obs.format_address = format;
    obs.destination = dest;
    obs.capacity = cap;
    obs.stream = stream;
    let fmt = engine::string(m, format, Some(1024 * 1024))?;
    if fmt.len() == 1024 * 1024 {
        return Err(AccessError::Limit.into());
    }
    let mut args = if let Some(a) = list {
        Arguments::list(m, a)?
    } else {
        Arguments::direct(f, named)
    };
    let rendered = engine::format(m, &fmt, &mut args)?;
    obs.required = rendered.bytes.len();
    obs.conversions = rendered.conversions;
    if stream.is_some() {
        owner
            .output
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .append(&rendered.bytes)
            .map_err(|e| match e {
                astero_kernel::process::output::OutputError::Capacity => AccessError::Limit,
                astero_kernel::process::output::OutputError::Allocation => AccessError::Allocation,
            })?;
        obs.written = rendered.bytes.len();
    } else {
        let capacity = cap.unwrap_or(rendered.bytes.len() as u64 + 1);
        let count = rendered
            .bytes
            .len()
            .min(capacity.saturating_sub(1) as usize);
        obs.truncated = count < rendered.bytes.len();
        if capacity > 0 {
            if dest == 0 {
                return Err(AccessError::Range.into());
            }
            m.charge(count as u64 + 1)?;
            m.validate(dest, count as u64 + 1, true)?;
            let mut b = rendered.bytes[..count].to_vec();
            b.push(0);
            for (offset, chunk) in b.chunks(COPY_CHUNK_BYTES as usize).enumerate() {
                m.write(dest + offset as u64 * COPY_CHUNK_BYTES, chunk)?;
            }
            obs.written = count;
        }
    }
    Ok(rendered.bytes.len() as u64)
}
pub fn registrations(owner: Arc<Formatting>) -> Vec<Registration> {
    EXPORTS
        .iter()
        .enumerate()
        .map(|(i, (name, nid))| {
            let owner = owner.clone();
            Registration {
                key: key(*nid),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    let mut records = owner.records.lock().unwrap_or_else(|p| p.into_inner());
                    if records.len() == 4096 {
                        return CallResult::AccessFailure(AccessError::Limit);
                    }
                    if records.try_reserve(1).is_err() {
                        return CallResult::AccessFailure(AccessError::Allocation);
                    }
                    let mut obs = Observation {
                        provider: name,
                        format_address: 0,
                        destination: 0,
                        capacity: None,
                        required: 0,
                        written: 0,
                        conversions: 0,
                        truncated: false,
                        stream: None,
                        error: None,
                    };
                    let slot = records.len();
                    records.push(obs.clone());
                    drop(records);
                    let result = perform(i, f, m, &owner, &mut obs);
                    obs.error = result.as_ref().err().cloned();
                    let mut records = owner.records.lock().unwrap_or_else(|p| p.into_inner());
                    records[slot] = obs;
                    match result {
                        Ok(n) => {
                            f.rax = n;
                            CallResult::Returned
                        }
                        Err(Error::Access(e)) => CallResult::AccessFailure(e),
                        Err(Error::Format { offset, reason }) => {
                            CallResult::FormatFailure { offset, reason }
                        }
                    }
                })),
            }
        })
        .collect()
}
