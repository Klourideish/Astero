//! Bounded stack/variant-II TLS layout, with explicit experimental TCB reserve.
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{Geometry, GuestRange},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLimits {
    pub stack_base: u64,
    pub stack_bytes: u64,
    pub tls_base: u64,
    pub max_runtime_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreparationError {
    Arithmetic,
    Alignment,
    StackTooSmall,
    TlsShape,
    Budget { required: u64, maximum: u64 },
    Allocation,
    Overlap,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadLayout {
    pub guard: GuestRange,
    pub stack: GuestRange,
    pub tls: GuestRange,
    pub thread_pointer: u64,
    pub params: u64,
    pub argv0: u64,
    pub rsp: u64,
    pub tls_template_size: u64,
    pub tls_memory_size: u64,
    pub tls_alignment: u64,
    pub reserved_bytes: u64,
}
fn round(n: u64, a: u64) -> Result<u64, PreparationError> {
    if !a.is_power_of_two() {
        return Err(PreparationError::Alignment);
    }
    n.checked_add(a - 1)
        .map(|x| x & !(a - 1))
        .ok_or(PreparationError::Arithmetic)
}
pub fn plan(
    l: RuntimeLimits,
    g: Geometry,
    file_size: u64,
    memory_size: u64,
    alignment: u64,
) -> Result<ThreadLayout, PreparationError> {
    if !g.page_size.is_power_of_two()
        || !g.allocation_granularity.is_power_of_two()
        || g.allocation_granularity < g.page_size
        || l.stack_base == 0
        || l.tls_base == 0
        || !l.stack_base.is_multiple_of(g.allocation_granularity)
        || !l.tls_base.is_multiple_of(g.allocation_granularity)
        || !l.stack_bytes.is_multiple_of(g.page_size)
    {
        return Err(PreparationError::Alignment);
    }
    if l.stack_bytes < g.page_size {
        return Err(PreparationError::StackTooSmall);
    }
    if file_size > memory_size || (alignment > 1 && !alignment.is_power_of_two()) {
        return Err(PreparationError::TlsShape);
    }
    if !l.tls_base.is_multiple_of(alignment.max(16)) {
        return Err(PreparationError::Alignment);
    }
    let data = round(memory_size, alignment.max(16))?;
    let tls_size = round(
        data.checked_add(0x200)
            .ok_or(PreparationError::Arithmetic)?,
        g.page_size,
    )?;
    let stack_start = l
        .stack_base
        .checked_add(g.page_size)
        .ok_or(PreparationError::Arithmetic)?;
    let top = stack_start
        .checked_add(l.stack_bytes)
        .ok_or(PreparationError::Arithmetic)?;
    let tls_end = l
        .tls_base
        .checked_add(tls_size)
        .ok_or(PreparationError::Arithmetic)?;
    if l.tls_base < top && l.stack_base < tls_end {
        return Err(PreparationError::Overlap);
    }
    let required = g
        .page_size
        .checked_add(l.stack_bytes)
        .and_then(|v| v.checked_add(tls_size))
        .ok_or(PreparationError::Arithmetic)?;
    if required > l.max_runtime_bytes {
        return Err(PreparationError::Budget {
            required,
            maximum: l.max_runtime_bytes,
        });
    }
    let params = top.checked_sub(64).ok_or(PreparationError::Arithmetic)?;
    Ok(ThreadLayout {
        guard: GuestRange {
            start: GuestAddress(l.stack_base),
            size: g.page_size,
        },
        stack: GuestRange {
            start: GuestAddress(stack_start),
            size: l.stack_bytes,
        },
        tls: GuestRange {
            start: GuestAddress(l.tls_base),
            size: tls_size,
        },
        thread_pointer: l
            .tls_base
            .checked_add(data)
            .ok_or(PreparationError::Arithmetic)?,
        params,
        argv0: params + 32,
        rsp: params - 8,
        tls_template_size: file_size,
        tls_memory_size: memory_size,
        tls_alignment: alignment,
        reserved_bytes: required,
    })
}
