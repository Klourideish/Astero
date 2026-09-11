//! Checked copies across explicitly retained adjacent native owners. Never raw guest references.
use crate::mapping::{
    GuestAddress,
    windows_native::{NativeError, NativeImage},
};
fn owner<'a>(
    images: &[&'a NativeImage],
    a: u64,
    write: bool,
) -> Result<(&'a NativeImage, u64), NativeError> {
    for image in images {
        let pages = image.pages();
        let i = pages.partition_point(|p| p.range.start.0 <= a);
        if let Some(p) = i.checked_sub(1).and_then(|i| pages.get(i)) {
            let end = p
                .range
                .start
                .0
                .checked_add(p.range.size)
                .ok_or(NativeError::Overflow)?;
            if a < end {
                image.validate(GuestAddress(a), 1, write)?;
                return Ok((image, end - a));
            }
        }
    }
    Err(NativeError::Unreadable)
}
pub fn validate(images: &[&NativeImage], a: u64, n: u64, write: bool) -> Result<(), NativeError> {
    let end = a.checked_add(n).ok_or(NativeError::Overflow)?;
    let mut at = a;
    while at < end {
        let (_, span) = owner(images, at, write)?;
        at += span.min(end - at);
    }
    Ok(())
}
pub fn read_window(images: &[&NativeImage], a: u64, n: u64) -> Result<Vec<u8>, NativeError> {
    if n == 0 {
        return Ok(Vec::new());
    }
    let (image, span) = owner(images, a, false)?;
    image.read(GuestAddress(a), n.min(span))
}
pub fn read(images: &[&NativeImage], a: u64, n: u64) -> Result<Vec<u8>, NativeError> {
    validate(images, a, n, false)?;
    let len = usize::try_from(n).map_err(|_| NativeError::Allocation)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(len)
        .map_err(|_| NativeError::Allocation)?;
    let mut at = a;
    while result.len() < len {
        let b = read_window(images, at, (len - result.len()) as u64)?;
        at += b.len() as u64;
        result.extend_from_slice(&b);
    }
    Ok(result)
}
pub fn write(images: &[&NativeImage], a: u64, bytes: &[u8]) -> Result<(), NativeError> {
    validate(images, a, bytes.len() as u64, true)?;
    let mut done = 0;
    while done < bytes.len() {
        let (image, span) = owner(images, a + done as u64, true)?;
        let n = (span as usize).min(bytes.len() - done);
        image.write(GuestAddress(a + done as u64), &bytes[done..done + n])?;
        done += n;
    }
    Ok(())
}
