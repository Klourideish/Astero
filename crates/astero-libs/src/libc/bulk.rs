//! Full-range preflight before bounded scratch copies; OS failures after mutation do not roll back.
use astero_hle::calls::memory::{AccessError, COPY_CHUNK_BYTES, GuestMemory};
pub fn fill(m: &mut dyn GuestMemory, a: u64, value: u8, n: u64) -> Result<(), AccessError> {
    m.charge(n)?;
    m.validate(a, n, true)?;
    let scratch = vec![value; n.min(COPY_CHUNK_BYTES) as usize];
    let mut done = 0;
    while done < n {
        let count = (n - done).min(COPY_CHUNK_BYTES);
        m.write(a + done, &scratch[..count as usize])?;
        done += count;
    }
    Ok(())
}
pub fn copy(
    m: &mut dyn GuestMemory,
    dst: u64,
    src: u64,
    n: u64,
    moving: bool,
) -> Result<(), AccessError> {
    m.charge(n)?;
    m.validate(src, n, false)?;
    m.validate(dst, n, true)?;
    if n == 0 {
        return Ok(());
    }
    let overlap = dst < src + n && src < dst + n;
    if overlap && !moving {
        return Err(AccessError::Overlap);
    }
    let backwards = overlap && dst > src;
    let mut done = 0;
    while done < n {
        let count = (n - done).min(COPY_CHUNK_BYTES);
        let at = if backwards { n - done - count } else { done };
        let b = m.read(src + at, count)?;
        m.write(dst + at, &b)?;
        done += count;
    }
    Ok(())
}
pub fn compare(m: &dyn GuestMemory, a: u64, b: u64, n: u64) -> Result<u64, AccessError> {
    m.charge(n)?;
    m.validate(a, n, false)?;
    m.validate(b, n, false)?;
    let mut done = 0;
    while done < n {
        let count = (n - done).min(COPY_CHUNK_BYTES);
        let l = m.read(a + done, count)?;
        let r = m.read(b + done, count)?;
        for (x, y) in l.iter().zip(r.iter()) {
            if x != y {
                return Ok((*x as i32 - *y as i32) as u32 as u64);
            }
        }
        done += count;
    }
    Ok(0)
}
pub fn find(m: &dyn GuestMemory, a: u64, value: u8, n: u64) -> Result<u64, AccessError> {
    m.charge(n)?;
    m.validate(a, n, false)?;
    let mut done = 0;
    while done < n {
        let count = (n - done).min(COPY_CHUNK_BYTES);
        let b = m.read(a + done, count)?;
        if let Some(i) = b.iter().position(|v| *v == value) {
            return Ok(a + done + i as u64);
        }
        done += count;
    }
    Ok(0)
}
