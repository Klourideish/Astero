use super::*;
use astero_core::input::{
    acquisition, linkage_evidence,
    load_plan::{self, GuestLoadPlan, ProviderInput},
    ps5_identity,
};
use std::{path::Path, sync::Arc};
#[derive(Debug)]
pub enum Failure {
    Acquire(acquisition::AcquisitionError),
    Plan(load_plan::PlanningError),
}
fn evidence(
    path: &Path,
    r: &Request,
) -> Result<Arc<ps5_identity::Ps5IdentityEvidenceReport>, Failure> {
    let lower = &r.identity.linkage;
    let s =
        acquisition::acquire(path, lower.symbols.acquisition.limits).map_err(Failure::Acquire)?;
    let l = linkage_evidence::observe(
        s,
        linkage_evidence::LinkageLimits {
            hash: lower.symbols.hash,
            symbols: lower.symbols.symbols,
            max_relocations: lower.max_relocations,
        },
    );
    Ok(Arc::new(ps5_identity::observe(
        Arc::new(l),
        ps5_identity::IdentityLimits {
            max_identity_records: r.identity.max_identity_records,
        },
    )))
}
pub fn execute(r: &Request) -> Result<GuestLoadPlan, Failure> {
    if r.providers.len() as u64 > r.limits.max_providers {
        return Err(Failure::Plan(load_plan::PlanningError::ProviderBudget {
            count: r.providers.len() as u64,
            maximum: r.limits.max_providers,
        }));
    }
    let input = evidence(&r.identity.linkage.symbols.acquisition.path, r)?;
    let mut providers = Vec::new();
    providers
        .try_reserve_exact(r.providers.len())
        .map_err(|_| {
            Failure::Plan(load_plan::PlanningError::Allocation {
                records: r.providers.len() as u64,
            })
        })?;
    for (path, alias) in &r.providers {
        providers.push(ProviderInput {
            evidence: evidence(path, r)?,
            dependency_alias: alias.clone(),
            image_bias: None,
        });
    }
    load_plan::plan(input, providers, r.bias, r.limits).map_err(Failure::Plan)
}
