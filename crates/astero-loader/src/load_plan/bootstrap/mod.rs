//! Reuse observed PH/dynamic evidence for initial-thread planning; no invocation or TLS allocation.
use crate::{
    artifact::BoundSourceRange,
    elf::inspect::bounded::InspectionOutcome,
    load_plan::link::{Blocker, GuestLoadPlan},
    metadata::{AddressRange, VirtualAddress},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlsTemplate {
    pub program_index: usize,
    pub source: BoundSourceRange,
    pub memory_size: u64,
    pub alignment: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapEvidence {
    pub raw_entry: u64,
    pub tls: Option<TlsTemplate>,
    pub relro: Vec<AddressRange>,
    pub procparam: Option<AddressRange>,
    pub dynamic_init: Vec<(i64, u64)>,
}
#[derive(Debug)]
pub enum BootstrapError {
    Headers,
    MultipleTls,
    MultipleProcparam,
    TlsShape,
    Overflow,
    Source(crate::artifact::SourceError),
    Allocation,
}
pub fn observe(plan: &GuestLoadPlan) -> Result<BootstrapEvidence, BootstrapError> {
    let InspectionOutcome::Complete(h) = plan.headers().outcome() else {
        return Err(BootstrapError::Headers);
    };
    let mut b = BootstrapEvidence {
        raw_entry: h.header().entry,
        tls: None,
        relro: Vec::new(),
        procparam: None,
        dynamic_init: Vec::new(),
    };
    b.relro
        .try_reserve(h.program_headers().len())
        .map_err(|_| BootstrapError::Allocation)?;
    b.dynamic_init
        .try_reserve(plan.blockers().len())
        .map_err(|_| BootstrapError::Allocation)?;
    for (i, p) in h.program_headers().iter().enumerate() {
        match p.kind {
            7 => {
                if b.tls.is_some() {
                    return Err(BootstrapError::MultipleTls);
                }
                if p.file_size > p.memory_size
                    || p.alignment > 1
                        && (!p.alignment.is_power_of_two()
                            || !p.virtual_address.is_multiple_of(p.alignment))
                {
                    return Err(BootstrapError::TlsShape);
                }
                b.tls = Some(TlsTemplate {
                    program_index: i,
                    source: plan
                        .headers()
                        .source()
                        .checked_range(p.file_offset, p.file_size)
                        .map_err(BootstrapError::Source)?,
                    memory_size: p.memory_size,
                    alignment: p.alignment,
                });
            }
            0x6474e552 | 0x61000001 => {
                let start = plan
                    .image_bias()
                    .0
                    .checked_add(p.virtual_address)
                    .ok_or(BootstrapError::Overflow)?;
                start
                    .checked_add(p.memory_size)
                    .ok_or(BootstrapError::Overflow)?;
                let range = AddressRange {
                    start: VirtualAddress(start),
                    size: p.memory_size,
                };
                if p.kind == 0x6474e552 {
                    b.relro.push(range)
                } else {
                    if b.procparam.is_some() {
                        return Err(BootstrapError::MultipleProcparam);
                    }
                    b.procparam = Some(range)
                }
            }
            _ => {}
        }
    }
    for p in plan.blockers() {
        if let Blocker::Bootstrap { tag, value } = p {
            b.dynamic_init.push((*tag, *value));
        }
    }
    Ok(b)
}
