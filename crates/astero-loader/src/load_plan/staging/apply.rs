use super::*;
use crate::load_plan::link::{Action, Blocker, GuestLoadPlan, RelocationProblem};
use std::sync::Arc;
/// Conservative stage admission. Unknown program semantics remain stage-blocking except
/// known TLS/RELRO/SCE parameter records whose contents remain execution work.
pub fn classify_blocker(b: &Blocker) -> BlockerClass {
    match b {
        Blocker::Dependency { .. }
        | Blocker::Reference { .. }
        | Blocker::ProviderRequiresOwnLoadPlan { .. }
        | Blocker::ProviderEvidence { .. } => BlockerClass::ResolutionBlocking,
        Blocker::EntryUnavailable | Blocker::EntryOutsideExecutable | Blocker::Bootstrap { .. } => {
            BlockerClass::ExecutionBlocking
        }
        Blocker::ProgramSemantics {
            kind: 7 | 0x6474e552 | 0x61000001 | 0x61000002 | 0x6fffff00 | 0x6fffff01,
            ..
        } => BlockerClass::ExecutionBlocking,
        Blocker::Relocation {
            reason: RelocationProblem::SymbolUnavailable | RelocationProblem::Unsupported,
            ..
        } => BlockerClass::ResolutionBlocking,
        _ => BlockerClass::StageBlocking,
    }
}
/// Consumes a fresh backend. Any error drops isolated storage; no partial image escapes.
/// Proven independent values are written verbatim; providers are never assumed resident.
pub fn stage<B: StagingBackend>(
    plan: Arc<GuestLoadPlan>,
    mut backend: B,
    limits: StagingLimits,
) -> Result<StagedGuestImage<B>, StagingError<B::Error>> {
    let result = apply(&plan, &mut backend, limits);
    match result {
        Ok((relocations, snapshot)) => Ok(StagedGuestImage {
            plan,
            backend,
            relocations,
            snapshot,
        }),
        Err(e) => {
            backend.clear();
            Err(e)
        }
    }
}
fn apply<B: StagingBackend>(
    plan: &GuestLoadPlan,
    backend: &mut B,
    limits: StagingLimits,
) -> Result<(Vec<RelocationState>, StagingSnapshot), StagingError<B::Error>> {
    for (index, b) in plan.blockers().iter().enumerate() {
        if classify_blocker(b) == BlockerClass::StageBlocking {
            return Err(StagingError::Blocked {
                index,
                blocker: b.clone(),
            });
        }
    }
    let (mut total, mut copied) = (0u64, 0u64);
    for (i, s) in plan.segments().iter().enumerate() {
        let m = &s.mapping;
        let end = m
            .range
            .start
            .0
            .checked_add(m.range.size)
            .ok_or(StagingError::Arithmetic)?;
        let n = m.copy.as_ref().map_or(0, |c| c.source.extent().size);
        if n > m.range.size
            || !m.alignment.max(1).is_power_of_two()
            || !plan.image_bias().0.is_multiple_of(m.alignment.max(1))
            || m.copy.as_ref().is_some_and(|c| {
                c.destination != m.range.start
                    || c.source.extent().offset.0 % m.alignment.max(1)
                        != m.range.start.0 % m.alignment.max(1)
            })
            || plan.segments()[..i].iter().any(|prior| {
                m.range.start.0 < prior.mapping.range.start.0 + prior.mapping.range.size
                    && prior.mapping.range.start.0 < end
            })
        {
            return Err(StagingError::InvalidMapping { segment: i });
        }
        let expected_zero = m.range.size - n;
        if m.zero_fill.map(|z| (z.start.0, z.size))
            != (expected_zero > 0).then_some((m.range.start.0 + n, expected_zero))
        {
            return Err(StagingError::InvalidMapping { segment: i });
        }
        if let Some(c) = &m.copy {
            plan.headers()
                .source()
                .read(&c.source)
                .map_err(|error| StagingError::Source { segment: i, error })?;
        }
        total = total
            .checked_add(m.range.size)
            .ok_or(StagingError::Arithmetic)?;
        copied = copied.checked_add(n).ok_or(StagingError::Arithmetic)?;
    }
    if total > limits.max_mapped_bytes {
        return Err(StagingError::Budget {
            required: total,
            maximum: limits.max_mapped_bytes,
        });
    }
    let mut states = Vec::new();
    states
        .try_reserve_exact(plan.relocations().len())
        .map_err(|_| StagingError::Allocation)?;
    // Preflight all known-width targets, even pending relocations; unknown widths are never written.
    for (i, r) in plan.relocations().iter().enumerate() {
        if r.width != 0
            && !r.place.is_some_and(|v| {
                plan.segments().iter().any(|s| {
                    v.0 >= s.mapping.range.start.0
                        && v.0.checked_add(r.width as u64).is_some_and(|end| {
                            end <= s.mapping.range.start.0 + s.mapping.range.size
                        })
                })
            })
        {
            return Err(StagingError::InvalidRelocation { index: i });
        }
        if r.value.is_some()
            && !matches!(
                (r.action, r.width),
                (Action::PcRelative32, 4)
                    | (
                        Action::Absolute64
                            | Action::GlobDat64
                            | Action::JumpSlot64
                            | Action::Relative64,
                        8
                    )
            )
        {
            return Err(StagingError::InvalidRelocation { index: i });
        }
    }
    backend
        .reserve_image(plan.segments())
        .map_err(|error| StagingError::Backend {
            operation: "reserve",
            index: 0,
            error,
        })?;
    for (i, s) in plan.segments().iter().enumerate() {
        backend
            .map(&s.mapping)
            .map_err(|error| StagingError::Backend {
                operation: "map",
                index: i,
                error,
            })?;
        if let Some(c) = &s.mapping.copy {
            let bytes = plan
                .headers()
                .source()
                .read(&c.source)
                .map_err(|error| StagingError::Source { segment: i, error })?;
            backend
                .write(c.destination, bytes)
                .map_err(|error| StagingError::Backend {
                    operation: "copy",
                    index: i,
                    error,
                })?;
        }
    }
    let (mut applied, mut pending) = (0, 0);
    for (i, r) in plan.relocations().iter().enumerate() {
        let state = if r.action == Action::None {
            RelocationState::NoWrite
        } else if let Some(value) = r.value.filter(|_| r.resolution.is_none()) {
            // x86-64 byte stores permit unaligned data; no host pointer casts.
            backend
                .write(
                    r.place
                        .ok_or(StagingError::InvalidRelocation { index: i })?,
                    &value.to_le_bytes()[..r.width as usize],
                )
                .map_err(|error| StagingError::Backend {
                    operation: "relocation",
                    index: i,
                    error,
                })?;
            applied += 1;
            RelocationState::Applied
        } else {
            pending += 1;
            RelocationState::Pending
        };
        states.push(state);
    }
    let protections = backend
        .finalize(plan.segments())
        .map_err(|error| StagingError::Backend {
            operation: "finalize",
            index: 0,
            error,
        })?;
    // Explicitly supplied provider plans are not staged here, including Selected resolutions.
    let unresolved = plan.references().len();
    Ok((
        states,
        StagingSnapshot {
            status: if pending > 0 || unresolved > 0 {
                StageStatus::StagedWithPendingWork
            } else {
                StageStatus::Staged
            },
            mapped_segments: plan.segments().len(),
            mapped_bytes: total,
            copied_bytes: copied,
            zero_filled_bytes: total - copied,
            applied_relocations: applied,
            pending_relocations: pending,
            unresolved_references: unresolved,
            protections,
            ready_for_execution: false,
        },
    ))
}
