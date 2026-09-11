//! PS5Rust checked CRT copy family. Constraint failures clear destination per inherited rules.
//! Resource exhaustion remains a structured runtime refusal, never a successful short operation.
use astero_hle::calls::memory::{AccessError, GuestMemory, MAX_OPERATION_BYTES};
fn failure(m: &mut dyn GuestMemory, dst: u64, code: u64) -> Result<u64, AccessError> {
    m.write(dst, &[0])?;
    Ok(code)
}
fn map(
    m: &mut dyn GuestMemory,
    dst: u64,
    result: Result<(), AccessError>,
) -> Result<u64, AccessError> {
    match result {
        Ok(()) => Ok(0),
        Err(AccessError::Range | AccessError::Overlap) => failure(m, dst, 22),
        Err(e) => Err(e),
    }
}
pub fn call(name: &str, a: [u64; 6], m: &mut dyn GuestMemory) -> Result<u64, AccessError> {
    let [dst, capacity, src, count, ..] = a;
    if dst == 0 || capacity == 0 || m.validate(dst, 1, true).is_err() {
        return Ok(22);
    }
    if name == "memset_s" {
        if count > capacity {
            if m.validate(dst, capacity, true).is_ok() {
                super::bulk::fill(m, dst, 0, capacity)?;
            } else {
                m.write(dst, &[0])?;
            }
            return Ok(34);
        }
        let r = super::bulk::fill(m, dst, src as u8, count);
        return map(m, dst, r);
    }
    if name == "memcpy_s" {
        if count > capacity {
            return failure(m, dst, 34);
        }
        if count == 0 {
            return Ok(0);
        }
        if src == 0 {
            return failure(m, dst, 22);
        }
        let r = super::bulk::copy(m, dst, src, count, false);
        return map(m, dst, r);
    }
    let concat = matches!(name, "strcat_s" | "strncat_s");
    let bounded = matches!(name, "strncpy_s" | "strncat_s");
    let prefix = if concat {
        match super::strings::length(m, dst, Some(capacity.min(MAX_OPERATION_BYTES))) {
            Ok(n) if n < capacity.min(MAX_OPERATION_BYTES) => n,
            Ok(_) => return failure(m, dst, 34),
            Err(AccessError::Range) => return failure(m, dst, 22),
            Err(e) => return Err(e),
        }
    } else {
        0
    };
    if bounded && count == 0 {
        return Ok(0);
    }
    if src == 0 {
        return failure(m, dst, 22);
    }
    let bound = if bounded {
        count
    } else {
        capacity.saturating_sub(prefix).min(MAX_OPERATION_BYTES)
    };
    let n = match super::strings::length(m, src, Some(bound)) {
        Ok(n) => n,
        Err(AccessError::Range) => return failure(m, dst, 22),
        Err(e) => return Err(e),
    };
    if (!bounded && n == bound)
        || prefix
            .checked_add(n)
            .and_then(|v| v.checked_add(1))
            .is_none_or(|v| v > capacity)
    {
        return failure(m, dst, 34);
    }
    let at = dst.checked_add(prefix).ok_or(AccessError::Range)?;
    let result = (|| {
        m.validate(at, n + 1, true)?;
        m.charge(1)?;
        super::bulk::copy(m, at, src, n, false)?;
        m.write(at + n, &[0])
    })();
    map(m, dst, result)
}
