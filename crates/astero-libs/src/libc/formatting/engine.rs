//! Byte-oriented formatter using migrated numeric rendering and one guest argument cursor.
use super::{
    Error, MAX_CONVERSIONS, MAX_FIELD_WIDTH, MAX_OUTPUT, Result, arguments::Arguments, render::*,
};
use astero_hle::calls::memory::{AccessError, COPY_CHUNK_BYTES, GuestMemory};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatted {
    pub bytes: Vec<u8>,
    pub conversions: usize,
}
pub fn string(m: &dyn GuestMemory, a: u64, precision: Option<usize>) -> Result<Vec<u8>> {
    if a == 0 && precision != Some(0) {
        return Err(Error::contract("null guest string"));
    }
    let n = crate::libc::strings::length(m, a, precision.map(|n| n as u64))?;
    let mut out = Vec::new();
    out.try_reserve(n as usize)
        .map_err(|_| AccessError::Allocation)?;
    let mut done = 0;
    while done < n {
        let len = (n - done).min(COPY_CHUNK_BYTES);
        m.charge(len)?;
        let b = m.read(a.checked_add(done).ok_or(AccessError::Range)?, len)?;
        if b.len() != len as usize {
            return Err(AccessError::Range.into());
        }
        out.extend_from_slice(&b);
        done += len;
    }
    Ok(out)
}
fn append(out: &mut Vec<u8>, b: &[u8]) -> Result<()> {
    if out
        .len()
        .checked_add(b.len())
        .is_none_or(|n| n > MAX_OUTPUT)
    {
        return Err(AccessError::Limit.into());
    }
    out.try_reserve(b.len())
        .map_err(|_| AccessError::Allocation)?;
    out.extend_from_slice(b);
    Ok(())
}
fn byte_pad(mut b: Vec<u8>, width: usize, left: bool) -> Vec<u8> {
    if width <= b.len() {
        return b;
    }
    let n = width - b.len();
    if left {
        b.resize(width, b' ');
        b
    } else {
        let mut out = vec![b' '; n];
        out.extend_from_slice(&b);
        out
    }
}
pub fn format(m: &dyn GuestMemory, fmt: &[u8], args: &mut Arguments<'_>) -> Result<Formatted> {
    let mut out = Vec::new();
    let mut i = 0;
    let mut conversions = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            let start = i;
            while i < fmt.len() && fmt[i] != b'%' {
                i += 1
            }
            append(&mut out, &fmt[start..i])?;
            continue;
        }
        let start = i;
        i += 1;
        if fmt.get(i) == Some(&b'%') {
            append(&mut out, b"%")?;
            i += 1;
            continue;
        }
        conversions += 1;
        if conversions > MAX_CONVERSIONS {
            return Err(AccessError::Limit.into());
        }
        let (mut spec, next) = parse_spec(fmt, i).map_err(|e| match e {
            Error::Format { reason, .. } => Error::Format {
                offset: start,
                reason,
            },
            e => e,
        })?;
        i = next;
        let width = match spec.width {
            Width::None => 0,
            Width::Literal(n) => n,
            Width::Argument => {
                let n = args.integer(m)? as i32;
                if n < 0 {
                    spec.flags.left = true;
                }
                n.unsigned_abs() as usize
            }
        };
        if width > MAX_FIELD_WIDTH {
            return Err(AccessError::Limit.into());
        }
        if spec.precision == Some(usize::MAX) {
            let n = args.integer(m)? as i32;
            spec.precision = if n < 0 { None } else { Some(n as usize) };
        }
        if spec.precision.is_some_and(|n| n > MAX_FIELD_WIDTH) {
            return Err(AccessError::Limit.into());
        }
        if spec.length == Length::BigL {
            return Err(Error::Format {
                offset: start,
                reason: "long double/unsupported L modifier",
            });
        }
        let b = match spec.conversion {
            b'd' | b'i' | b'u' | b'o' | b'x' | b'X' => {
                render_integer(args.integer(m)?, spec, width).into_bytes()
            }
            b'p' => {
                if spec.length != Length::None {
                    return Err(Error::contract("pointer length modifier"));
                }
                pad(format!("0x{:x}", args.integer(m)?), width, spec.flags, true).into_bytes()
            }
            b's' => {
                if spec.length != Length::None {
                    return Err(Error::Format {
                        offset: start,
                        reason: "wide string unsupported",
                    });
                }
                byte_pad(
                    string(m, args.integer(m)?, spec.precision)?,
                    width,
                    spec.flags.left,
                )
            }
            b'c' => {
                if spec.length != Length::None {
                    return Err(Error::Format {
                        offset: start,
                        reason: "wide character unsupported",
                    });
                }
                byte_pad(vec![args.integer(m)? as u8], width, spec.flags.left)
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => {
                if !matches!(spec.length, Length::None | Length::L) {
                    return Err(Error::contract("floating length modifier"));
                }
                let v = args.float(m)?;
                let a = v.abs();
                let upper = spec.conversion.is_ascii_uppercase();
                let p = spec.precision.unwrap_or(6);
                let mut body = if a.is_nan() {
                    "nan".to_string()
                } else if a.is_infinite() {
                    "inf".to_string()
                } else {
                    match spec.conversion {
                        b'f' | b'F' => format!("{a:.p$}"),
                        b'e' | b'E' => normalize_exponent(format!("{a:.p$e}"), upper),
                        _ => render_general(a, p.max(1), spec.flags.alternate, upper),
                    }
                };
                if spec.flags.alternate && a.is_finite() && !body.contains('.') {
                    let pos = body.find(['e', 'E']).unwrap_or(body.len());
                    body.insert(pos, '.');
                }
                if upper {
                    body.make_ascii_uppercase()
                }
                let mut flags = spec.flags;
                if !a.is_finite() {
                    flags.zero = false;
                }
                pad(
                    add_sign(flags, v.is_sign_negative(), body),
                    width,
                    flags,
                    true,
                )
                .into_bytes()
            }
            b'n' => {
                return Err(Error::Format {
                    offset: start,
                    reason: "percent-n writeback unsupported",
                });
            }
            _ => {
                return Err(Error::Format {
                    offset: start,
                    reason: "unsupported conversion (including hex float)",
                });
            }
        };
        append(&mut out, &b)?;
    }
    Ok(Formatted {
        bytes: out,
        conversions,
    })
}
