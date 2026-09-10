//! Deliberately unreadable unresolved-object traps; never synthetic object contents.
use super::*;
use astero_memory::mapping::{GuestAddress, windows_native::*};
#[derive(Clone, Debug)]
pub struct ObjectTrap {
    pub symbol: u32,
    pub range: GuestRange,
    pub anchor: u64,
}
pub(super) struct ObjectTraps {
    pub image: Option<NativeImage>,
    pub records: Vec<ObjectTrap>,
    pub bytes: u64,
}
/// Checked signed addend envelope. Every planned S+A stays within NOACCESS storage.
pub fn trap_geometry(start: u64, min: i64, max: i64, page: u64) -> Option<(u64, u64)> {
    if min > 0 || max < 0 || min > max || !page.is_power_of_two() {
        return None;
    }
    let span = u64::try_from(i128::from(max) - i128::from(min) + 1).ok()?;
    let size = span.checked_add(page - 1)? & !(page - 1);
    start.checked_add(size)?;
    let anchor = u64::try_from(i128::from(start) - i128::from(min)).ok()?;
    Some((anchor, size))
}
pub(super) fn build(
    start: u64,
    page: u64,
    maximum: u64,
    groups: &std::collections::BTreeMap<u32, (i64, i64)>,
) -> Result<ObjectTraps, ClosureError> {
    let mut records = Vec::new();
    records
        .try_reserve(groups.len())
        .map_err(|_| ClosureError::Allocation)?;
    let mut size = 0u64;
    for (&symbol, &(lo, hi)) in groups {
        let at = start.checked_add(size).ok_or(ClosureError::Budget)?;
        let (anchor, n) = trap_geometry(at, lo, hi, page).ok_or(ClosureError::Budget)?;
        size = size.checked_add(n).ok_or(ClosureError::Budget)?;
        if size > maximum {
            return Err(ClosureError::Budget);
        }
        records.push(ObjectTrap {
            symbol,
            range: GuestRange {
                start: GuestAddress(at),
                size: n,
            },
            anchor,
        });
    }
    if size == 0 {
        return Ok(ObjectTraps {
            image: None,
            records,
            bytes: 0,
        });
    }
    let len = usize::try_from(size).map_err(|_| ClosureError::Budget)?;
    let mut zeros = Vec::new();
    zeros
        .try_reserve_exact(len)
        .map_err(|_| ClosureError::Allocation)?;
    zeros.resize(len, 0);
    let (image, _) = realize(
        &[NativeRegion {
            range: GuestRange {
                start: GuestAddress(start),
                size,
            },
            bytes: &zeros,
            protection: Protection::default(),
        }],
        NativeLimits {
            max_reserved_bytes: maximum,
            max_committed_bytes: maximum,
        },
    );
    Ok(ObjectTraps {
        image: Some(image.map_err(|e| ClosureError::Native {
            operation: "object traps",
            error: e,
        })?),
        records,
        bytes: size,
    })
}
