use super::*;
use crate::mapping::GuestAddress;
fn up(v: u64, a: u64) -> Result<u64, NativeError> {
    v.checked_add(a - 1)
        .map(|v| v & !(a - 1))
        .ok_or(NativeError::Overflow)
}
/// Full bounded layout before any Windows mutation. Holes remain reserved, uncommitted.
pub fn plan_layout(
    regions: &[NativeRegion<'_>],
    geometry: Geometry,
    limits: NativeLimits,
) -> Result<NativeLayout, NativeError> {
    let Geometry {
        page_size: page,
        allocation_granularity: gran,
    } = geometry;
    if !page.is_power_of_two() || !gran.is_power_of_two() || gran < page {
        return Err(NativeError::Geometry);
    }
    if regions.is_empty() {
        return Err(NativeError::Empty);
    }
    let (mut first, mut last) = (u64::MAX, 0);
    for (i, r) in regions.iter().enumerate() {
        let start = r.range.start.0;
        let end = start
            .checked_add(r.range.size)
            .ok_or(NativeError::Overflow)?;
        if r.range.size == 0 || r.bytes.len() as u64 != r.range.size {
            return Err(NativeError::SourceSize { index: i });
        }
        if regions[..i]
            .iter()
            .any(|p| start < p.range.start.0 + p.range.size && p.range.start.0 < end)
        {
            return Err(NativeError::Overlap { index: i });
        }
        first = first.min(start);
        last = last.max(end);
    }
    let base = first & !(gran - 1);
    let end = up(last, page)?;
    let size = end.checked_sub(base).ok_or(NativeError::Overflow)?;
    if base == 0 {
        return Err(NativeError::PlacementMismatch);
    }
    if size > limits.max_reserved_bytes {
        return Err(NativeError::Budget {
            kind: "reservation",
            required: size,
            maximum: limits.max_reserved_bytes,
        });
    }
    // One slot per envelope page bounds work and avoids quadratic per-byte traversal.
    let count = usize::try_from(size / page).map_err(|_| NativeError::Allocation)?;
    let stride = usize::try_from(page).map_err(|_| NativeError::Geometry)?;
    let mut slots: Vec<Option<NativePage>> = Vec::new();
    slots
        .try_reserve_exact(count)
        .map_err(|_| NativeError::Allocation)?;
    slots.resize(count, None);
    for r in regions {
        let a = r.range.start.0 & !(page - 1);
        let end = up(r.range.start.0 + r.range.size, page)?;
        for addr in (a..end).step_by(stride) {
            let slot = &mut slots[((addr - base) / page) as usize];
            let mut merged = match slot {
                Some(old) => Protection {
                    read: old.protection.read || r.protection.read,
                    write: old.protection.write || r.protection.write,
                    execute: old.protection.execute || r.protection.execute,
                },
                None => r.protection,
            };
            if merged.write {
                merged.read = true;
            }
            if merged.write && merged.execute {
                return Err(NativeError::WritableExecutable { address: addr });
            }
            let widened = slot
                .as_ref()
                .is_some_and(|p| p.widened || p.protection != merged || r.protection != merged)
                || r.range.start.0 > addr
                || r.range.start.0 + r.range.size < addr + page;
            *slot = Some(NativePage {
                range: GuestRange {
                    start: GuestAddress(addr),
                    size: page,
                },
                protection: merged,
                widened,
            });
        }
    }
    let committed = slots.iter().filter(|p| p.is_some()).count() as u64 * page;
    if committed > limits.max_committed_bytes {
        return Err(NativeError::Budget {
            kind: "commit",
            required: committed,
            maximum: limits.max_committed_bytes,
        });
    }
    let mut pages = Vec::new();
    pages
        .try_reserve_exact((committed / page) as usize)
        .map_err(|_| NativeError::Allocation)?;
    pages.extend(slots.into_iter().flatten());
    Ok(NativeLayout {
        envelope: GuestRange {
            start: GuestAddress(base),
            size,
        },
        pages,
        geometry,
    })
}
