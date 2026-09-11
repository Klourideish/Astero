//! Byte-oriented strings with checked page-window scans, no UTF-8 requirement.
use astero_hle::calls::memory::{AccessError, COPY_CHUNK_BYTES, GuestMemory, MAX_OPERATION_BYTES};
struct Cursor<'a> {
    memory: &'a dyn GuestMemory,
    base: u64,
    offset: u64,
    limit: u64,
    bytes: Vec<u8>,
    at: usize,
}
impl<'a> Cursor<'a> {
    fn new(memory: &'a dyn GuestMemory, base: u64, limit: u64) -> Self {
        Self {
            memory,
            base,
            offset: 0,
            limit: limit.min(MAX_OPERATION_BYTES),
            bytes: Vec::new(),
            at: 0,
        }
    }
    fn next(&mut self) -> Result<Option<u8>, AccessError> {
        if self.offset == self.limit {
            return Ok(None);
        }
        if self.at == self.bytes.len() {
            self.bytes = self.memory.read_window(
                self.base
                    .checked_add(self.offset)
                    .ok_or(AccessError::Range)?,
                (self.limit - self.offset).min(4096),
            )?;
            if self.bytes.is_empty() {
                return Err(AccessError::Range);
            }
            self.memory.charge(self.bytes.len() as u64)?;
            self.at = 0;
        }
        let b = self.bytes[self.at];
        self.at += 1;
        self.offset += 1;
        Ok(Some(b))
    }
}
pub fn length(m: &dyn GuestMemory, a: u64, limit: Option<u64>) -> Result<u64, AccessError> {
    if limit.is_some_and(|n| n > MAX_OPERATION_BYTES) {
        return Err(AccessError::Limit);
    }
    let mut c = Cursor::new(m, a, limit.unwrap_or(MAX_OPERATION_BYTES));
    let mut n = 0;
    while let Some(b) = c.next()? {
        if b == 0 {
            return Ok(n);
        }
        n += 1;
    }
    if limit.is_some() {
        Ok(n)
    } else {
        Err(AccessError::Limit)
    }
}
fn charset(m: &dyn GuestMemory, a: u64) -> Result<[bool; 256], AccessError> {
    let mut set = [false; 256];
    let mut c = Cursor::new(m, a, MAX_OPERATION_BYTES);
    while let Some(b) = c.next()? {
        if b == 0 {
            return Ok(set);
        }
        set[b as usize] = true;
    }
    Err(AccessError::Limit)
}
pub fn call(name: &str, a: [u64; 6], m: &mut dyn GuestMemory) -> Result<u64, AccessError> {
    let [x, y, z, ..] = a;
    match name {
        "strlen" => length(m, x, None),
        "strnlen" => length(m, x, Some(y)),
        "strcmp" | "strncmp" | "strcasecmp" | "strncasecmp" => {
            let bounded = matches!(name, "strncmp" | "strncasecmp");
            let case = matches!(name, "strcasecmp" | "strncasecmp");
            let limit = if bounded { z } else { MAX_OPERATION_BYTES };
            if limit > MAX_OPERATION_BYTES {
                return Err(AccessError::Limit);
            }
            let mut l = Cursor::new(m, x, limit);
            let mut r = Cursor::new(m, y, limit);
            while let Some(mut b) = l.next()? {
                let mut c = r.next()?.ok_or(AccessError::Range)?;
                if case {
                    b = b.to_ascii_lowercase();
                    c = c.to_ascii_lowercase();
                }
                if b != c {
                    return Ok((b as i32 - c as i32) as u32 as u64);
                }
                if b == 0 {
                    return Ok(0);
                }
            }
            if bounded {
                Ok(0)
            } else {
                Err(AccessError::Limit)
            }
        }
        "strchr" | "strrchr" => {
            let mut c = Cursor::new(m, x, MAX_OPERATION_BYTES);
            let mut result = 0;
            let mut i = 0;
            while let Some(b) = c.next()? {
                if b == y as u8 {
                    result = x.checked_add(i).ok_or(AccessError::Range)?;
                    if name == "strchr" {
                        return Ok(result);
                    }
                }
                if b == 0 {
                    return Ok(result);
                }
                i += 1;
            }
            Err(AccessError::Limit)
        }
        "strcpy" | "strncpy" | "strcat" | "strncat" => {
            let bounded = matches!(name, "strncpy" | "strncat");
            let n = length(m, y, if bounded { Some(z) } else { None })?;
            let offset = if matches!(name, "strcat" | "strncat") {
                length(m, x, None)?
            } else {
                0
            };
            let dst = x.checked_add(offset).ok_or(AccessError::Range)?;
            let extent = if name == "strncpy" {
                z
            } else {
                n.checked_add(1).ok_or(AccessError::Range)?
            };
            m.charge(extent)?;
            m.validate(dst, extent, true)?;
            m.validate(y, n, false)?;
            if n > 0
                && dst < y.checked_add(n).ok_or(AccessError::Range)?
                && y < dst.checked_add(extent).ok_or(AccessError::Range)?
            {
                return Err(AccessError::Overlap);
            }
            let mut done = 0;
            while done < extent {
                let size = (extent - done).min(COPY_CHUNK_BYTES);
                let from = (n.saturating_sub(done)).min(size);
                let mut b = vec![0; size as usize];
                if from > 0 {
                    b[..from as usize].copy_from_slice(&m.read(y + done, from)?);
                }
                m.write(dst + done, &b)?;
                done += size;
            }
            Ok(x)
        }
        "strdup" => {
            let n = length(m, x, None)?
                .checked_add(1)
                .ok_or(AccessError::Range)?;
            let p = match m.allocate(n) {
                Ok(p) => p,
                Err(AccessError::Allocation) => return Ok(0),
                Err(e) => return Err(e),
            };
            if let Err(e) = super::bulk::copy(m, p, x, n, false) {
                m.free(p)?;
                return Err(e);
            }
            Ok(p)
        }
        "strspn" | "strcspn" => {
            let set = charset(m, y)?;
            let mut c = Cursor::new(m, x, MAX_OPERATION_BYTES);
            let mut n = 0;
            while let Some(b) = c.next()? {
                if b == 0 || set[b as usize] != (name == "strspn") {
                    return Ok(n);
                }
                n += 1;
            }
            Err(AccessError::Limit)
        }
        "strtok_r" => {
            let set = charset(m, y)?;
            m.validate(z, 8, true)?;
            let start = if x == 0 {
                u64::from_le_bytes(m.read(z, 8)?.try_into().map_err(|_| AccessError::Range)?)
            } else {
                x
            };
            if start == 0 {
                return Ok(0);
            }
            let mut c = Cursor::new(m, start, MAX_OPERATION_BYTES);
            let mut offset = 0;
            let mut token = None;
            while let Some(b) = c.next()? {
                if b == 0 {
                    let result = token.map_or(0, |i| start + i);
                    m.write(z, &0u64.to_le_bytes())?;
                    return Ok(result);
                }
                if set[b as usize] {
                    if let Some(first) = token {
                        let at = start.checked_add(offset).ok_or(AccessError::Range)?;
                        let next = at.checked_add(1).ok_or(AccessError::Range)?;
                        m.validate(at, 1, true)?;
                        m.write(at, &[0])?;
                        m.write(z, &next.to_le_bytes())?;
                        return Ok(start + first);
                    }
                } else if token.is_none() {
                    token = Some(offset);
                }
                offset += 1;
            }
            Err(AccessError::Limit)
        }
        "strstr" => {
            // Linear KMP search. Pattern scratch has a separate bounded capacity.
            let n = length(m, y, Some(1024 * 1024))?;
            if n == 1024 * 1024 {
                return Err(AccessError::Limit);
            }
            if n == 0 {
                return Ok(x);
            }
            let mut needle = Vec::new();
            needle
                .try_reserve_exact(n as usize)
                .map_err(|_| AccessError::Allocation)?;
            let mut at = 0;
            while at < n {
                let count = (n - at).min(COPY_CHUNK_BYTES);
                needle.extend(m.read(y + at, count)?);
                at += count;
            }
            let mut prefix = vec![0usize; needle.len()];
            let mut k = 0;
            for i in 1..needle.len() {
                while k > 0 && needle[k] != needle[i] {
                    k = prefix[k - 1];
                }
                if needle[k] == needle[i] {
                    k += 1;
                }
                prefix[i] = k;
            }
            let mut c = Cursor::new(m, x, MAX_OPERATION_BYTES);
            let mut i = 0;
            k = 0;
            while let Some(b) = c.next()? {
                if b == 0 {
                    return Ok(0);
                }
                while k > 0 && needle[k] != b {
                    k = prefix[k - 1];
                }
                if needle[k] == b {
                    k += 1;
                }
                i += 1;
                if k == needle.len() {
                    return x.checked_add(i - k as u64).ok_or(AccessError::Range);
                }
            }
            Err(AccessError::Limit)
        }
        _ => Err(AccessError::Range),
    }
}
