use super::*;
use crate::{
    elf::{
        dynamic::identity::{IdentityOutcome, Ps5IdentityEvidenceReport},
        inspect::bounded::{self, InspectionLimits},
    },
    metadata::VirtualAddress,
};
use std::sync::Arc;
/// Explicit evidence inputs; this function never reads files, starts workers or calls admission.
pub fn plan(
    input: Arc<Ps5IdentityEvidenceReport>,
    providers: Vec<ProviderInput>,
    bias: VirtualAddress,
    limits: PlanningLimits,
) -> Result<GuestLoadPlan, PlanningError> {
    if providers.len() as u64 > limits.max_providers {
        return Err(PlanningError::ProviderBudget {
            count: providers.len() as u64,
            maximum: limits.max_providers,
        });
    }
    let headers = bounded::inspect(
        input.linkage().source().clone(),
        InspectionLimits {
            max_program_headers: input.linkage().limits().hash.dynamic.max_program_headers,
        },
    );
    let mut p = GuestLoadPlan {
        input,
        providers,
        headers,
        limits,
        bias,
        entry: None,
        segments: Vec::new(),
        dependencies: Vec::new(),
        candidates: Vec::new(),
        references: Vec::new(),
        relocations: Vec::new(),
        blockers: Vec::new(),
        experimental: false,
    };
    let mut b = Budget::new(limits.max_plan_records);
    segments::collect(&mut p, &mut b)?;
    if !matches!(p.input.outcome(), IdentityOutcome::Complete(_)) {
        b.push(&mut p.blockers, Blocker::EvidenceUnavailable)?;
        return Ok(p);
    }
    if let IdentityOutcome::Complete(e) = p.input.outcome() {
        p.experimental |= !e.descriptors().is_empty();
    }
    providers::collect(&mut p, &mut b)?;
    relocations::collect(&mut p, &mut b)?;
    if let Some(raw) = p.input.linkage().symbols().input().proof().raw() {
        for e in raw.entries() {
            let tag = match e.tag {
                crate::elf::dynamic::tags::DynamicTag::Init => 12,
                crate::elf::dynamic::tags::DynamicTag::Fini => 13,
                crate::elf::dynamic::tags::DynamicTag::InitArray => 25,
                crate::elf::dynamic::tags::DynamicTag::FiniArray => 26,
                crate::elf::dynamic::tags::DynamicTag::Unknown(32) => 32,
                _ => 0,
            };
            if tag != 0 && e.value != 0 {
                b.push(
                    &mut p.blockers,
                    Blocker::Bootstrap {
                        tag,
                        value: e.value,
                    },
                )?;
            }
        }
    }
    Ok(p)
}
