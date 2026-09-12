use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::filesystem::service::{Error, Filesystem, Mode};
use std::sync::Arc;
enum Failure {
    Access(AccessError),
    Fs(Error),
}
impl From<AccessError> for Failure {
    fn from(e: AccessError) -> Self {
        Self::Access(e)
    }
}
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self::Fs(e)
    }
}
fn string(m: &dyn GuestMemory, p: u64) -> Result<String, Failure> {
    if p == 0 {
        return Err(AccessError::Range.into());
    }
    let mut b = Vec::new();
    for i in 0..4096 {
        let c = m.read(p.checked_add(i).ok_or(AccessError::Range)?, 1)?[0];
        if c == 0 {
            return String::from_utf8(b).map_err(|_| Error::Invalid.into());
        }
        b.push(c);
    }
    Err(Error::Invalid.into())
}
fn io(
    s: &Filesystem,
    m: &mut dyn GuestMemory,
    fd: u32,
    p: u64,
    n: u64,
    write: bool,
    at: Option<u64>,
) -> Result<u64, Failure> {
    m.charge(n)?;
    m.validate(p, n, !write)?;
    if let Some(x) = at {
        x.checked_add(n).ok_or(Error::Invalid)?;
    }
    let mut done = 0;
    while done < n {
        let count = (n - done).min(65536) as usize;
        let pos = at.map(|x| x + done);
        let k = if write {
            let b = m.read(p + done, count as u64)?;
            s.write(fd, &b, pos)?
        } else {
            let mut b = vec![0; count];
            let k = s.read(fd, &mut b, pos)?;
            m.write(p + done, &b[..k])?;
            k
        };
        done += k as u64;
        if k == 0 || k < count {
            break;
        }
    }
    Ok(done)
}
fn run(op: &str, a: [u64; 6], s: &Filesystem, m: &mut dyn GuestMemory) -> Result<u64, Failure> {
    let fd = || u32::try_from(a[0]).map_err(|_| Error::BadDescriptor);
    match op {
        "open" => Ok(s.open(&string(m, a[0])?, Mode::flags(a[1])?)? as u64),
        "fopen" => {
            let path = string(m, a[0])?;
            let mode = Mode::parse(&string(m, a[1])?)?;
            let fd = s.open(&path, mode)?;
            let token = match m.allocate(16) {
                Ok(t) => t,
                Err(e) => {
                    s.close(fd)?;
                    return Err(e.into());
                }
            };
            if let Err(e) = m.write(token, &[0; 16]) {
                s.close(fd)?;
                m.free(token)?;
                return Err(e.into());
            }
            s.bind_stream(token, fd)?;
            Ok(token)
        }
        "close" => {
            s.close(fd()?)?;
            Ok(0)
        }
        "fclose" => {
            s.close(s.stream(a[0])?)?;
            Ok(0)
        }
        "read" | "write" | "pread" | "pwrite" => io(
            s,
            m,
            fd()?,
            a[1],
            a[2],
            matches!(op, "write" | "pwrite"),
            if matches!(op, "pread" | "pwrite") {
                Some(a[3])
            } else {
                None
            },
        ),
        "fread" | "fwrite" => {
            let n = a[1].checked_mul(a[2]).ok_or(Error::Invalid)?;
            if n == 0 {
                return Ok(0);
            }
            Ok(io(s, m, s.stream(a[3])?, a[0], n, op == "fwrite", None)? / a[1])
        }
        "seek" => Ok(s.seek(fd()?, a[1] as i64, a[2])?),
        "fseek" => {
            s.seek(s.stream(a[0])?, a[1] as i64, a[2])?;
            Ok(0)
        }
        "ftell" => Ok(s.seek(s.stream(a[0])?, 0, 1)?),
        "rewind" => {
            let fd = s.stream(a[0])?;
            s.seek(fd, 0, 0)?;
            s.clear(fd)?;
            Ok(0)
        }
        "feof" | "ferror" => {
            let (e, r) = s.indicators(s.stream(a[0])?)?;
            Ok(if op == "feof" { e } else { r } as u64)
        }
        "fflush" => {
            if a[0] != 0 {
                s.flush(s.stream(a[0])?)?;
            }
            Ok(0)
        }
        "fileno" => Ok(s.stream(a[0])? as u64),
        "stat" | "fstat" => {
            m.validate(a[1], 120, true)?;
            let meta = if op == "stat" {
                s.metadata(&string(m, a[0])?)?
            } else {
                s.fmetadata(fd()?)?
            };
            m.write(
                a[1],
                &astero_abi::layouts::stat::encode(meta.is_dir(), meta.len()),
            )?;
            Ok(0)
        }
        "truncate" => {
            s.truncate(fd()?, a[1])?;
            Ok(0)
        }
        "access" => {
            s.metadata(&string(m, a[0])?)?;
            Ok(0)
        }
        "fgetc" => {
            let mut b = [0];
            let n = s.read(s.stream(a[0])?, &mut b, None)?;
            Ok(if n == 0 { u64::MAX } else { b[0] as u64 })
        }
        "fputc" => {
            if s.write(s.stream(a[1])?, &[a[0] as u8], None)? != 1 {
                return Err(Error::Io.into());
            }
            Ok(a[0] & 255)
        }
        "fputs" => {
            let b = string(m, a[0])?;
            if io(s, m, s.stream(a[1])?, a[0], b.len() as u64, true, None)? != b.len() as u64 {
                return Err(Error::Io.into());
            }
            Ok(0)
        }
        "fgets" => {
            if a[1] == 0 || a[1] > 64 * 1024 * 1024 {
                return Err(Error::Invalid.into());
            }
            m.validate(a[0], a[1], true)?;
            let fd = s.stream(a[2])?;
            let mut n = 0;
            while n + 1 < a[1] {
                let mut b = [0];
                if s.read(fd, &mut b, None)? == 0 {
                    break;
                }
                m.write(a[0] + n, &b)?;
                n += 1;
                if b[0] == 10 {
                    break;
                }
            }
            if n == 0 && a[1] > 1 {
                return Ok(0);
            }
            m.write(a[0] + n, &[0])?;
            Ok(a[0])
        }
        _ => Err(Error::Invalid.into()),
    }
}
pub fn registrations(s: Arc<Filesystem>, errno: u64) -> Vec<Registration> {
    super::EXPORTS
        .iter()
        .map(|(_, nid, op, style)| {
            let s = s.clone();
            let lib = if *style == "libc" {
                b"libc".to_vec()
            } else {
                b"libkernel".to_vec()
            };
            Registration {
                key: ProviderKey {
                    nid: *nid,
                    library: lib.clone(),
                    module: lib,
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| match run(op, f.arguments, &s, m) {
                    Ok(v) => {
                        f.rax = v;
                        CallResult::Returned
                    }
                    Err(Failure::Access(e)) => CallResult::AccessFailure(e),
                    Err(Failure::Fs(e)) => {
                        if *style == "sce" {
                            f.rax = (0x80020000 | e.errno()) as i32 as i64 as u64;
                        } else {
                            if let Err(e) = m.write(errno, &e.errno().to_le_bytes()) {
                                return CallResult::AccessFailure(e);
                            }
                            f.rax = if matches!(*op, "fopen" | "fread" | "fwrite" | "fgets") {
                                0
                            } else {
                                u64::MAX
                            };
                        }
                        CallResult::Returned
                    }
                })),
            }
        })
        .collect()
}
