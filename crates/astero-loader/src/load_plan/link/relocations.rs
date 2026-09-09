use super::*;
use crate::{
    elf::dynamic::{
        candidates::workload::LinkageOutcome,
        symbol_table::{Section, SymbolType},
    },
    metadata::VirtualAddress,
};
pub(super) fn collect(p: &mut GuestLoadPlan, b: &mut Budget) -> Result<(), PlanningError> {
    let LinkageOutcome::Complete(e) = p.input.linkage().outcome() else {
        return Ok(());
    };
    for (i, raw) in e.relocations().iter().enumerate() {
        let r = raw.record;
        let t = r.relocation_type();
        let si = u64::from(r.symbol_index());
        let (action, width) = match t {
            0 => (Action::None, 0),
            1 => (Action::Absolute64, 8),
            2 => (Action::PcRelative32, 4),
            6 => (Action::GlobDat64, 8),
            7 => (Action::JumpSlot64, 8),
            8 => (Action::Relative64, 8),
            _ => (Action::Unsupported(t), 0),
        };
        let place = p.bias.0.checked_add(r.offset).map(VirtualAddress);
        let mut reason = None;
        let mut value = None;
        let mut resolution = None;
        if matches!(action, Action::Unsupported(_)) {
            reason = Some(RelocationProblem::Unsupported)
        } else if width != 0
            && !place.is_some_and(|v| {
                p.segments.iter().any(|s| {
                    v.0 >= s.mapping.range.start.0
                        && v.0.checked_add(width as u64).is_some_and(|end| {
                            end <= s.mapping.range.start.0 + s.mapping.range.size
                        })
                })
            })
        {
            reason = Some(RelocationProblem::TargetOutsideImage)
        } else if t == 8 {
            if si != 0 {
                reason = Some(RelocationProblem::NonzeroRelativeSymbol)
            } else {
                value = p.bias.0.checked_add_signed(r.addend);
                if value.is_none() {
                    reason = Some(RelocationProblem::Arithmetic)
                }
            }
        } else if width != 0 {
            let symbol = p
                .input
                .linkage()
                .symbols()
                .entries()
                .find(|(_, s)| s.fields().index == si)
                .map(|(_, s)| s.fields());
            let address = if si == 0 {
                None
            } else if let Some(s) = symbol.filter(|s| {
                matches!(
                    s.symbol_type,
                    SymbolType::NoType
                        | SymbolType::Function
                        | SymbolType::Object
                        | SymbolType::Section
                )
            }) {
                match s.section {
                    Section::Index(_) => p.bias.0.checked_add(s.value).filter(|v| {
                        p.segments.iter().any(|seg| {
                            *v >= seg.mapping.range.start.0
                                && v.checked_add(s.size.max(1)).is_some_and(|end| {
                                    end <= seg.mapping.range.start.0 + seg.mapping.range.size
                                })
                        })
                    }),
                    Section::Absolute => Some(s.value),
                    Section::Undefined => {
                        resolution = p.references.iter().position(|r| r.symbol == si);
                        match resolution.map(|i| &p.references[i].resolution) {
                            Some(PlannedResolution::Selected(ci)) => {
                                let c = &p.candidates[*ci];
                                match c.target {
                                    ProviderTarget::ArtifactSymbol {
                                        provider, value, ..
                                    } => p.providers[provider]
                                        .image_bias
                                        .and_then(|b| b.0.checked_add(value.0)),
                                    ProviderTarget::FutureHleDeclaration => None,
                                }
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            if let Some(address) = address {
                value = match t {
                    1 => address.checked_add_signed(r.addend),
                    2 => {
                        let delta = i128::from(address) + i128::from(r.addend)
                            - i128::from(place.expect("validated place").0);
                        i32::try_from(delta).ok().map(|x| u64::from(x as u32))
                    }
                    6 | 7 => Some(address),
                    _ => None,
                };
                if value.is_none() {
                    reason = Some(RelocationProblem::Arithmetic)
                }
            } else {
                reason = Some(RelocationProblem::SymbolUnavailable)
            }
        }
        if width != 0
            && let Some(place) = place
            && p.relocations.iter().any(|prior| {
                prior.width != 0
                    && prior.place.is_some_and(|v| {
                        u128::from(v.0) < u128::from(place.0) + u128::from(width)
                            && u128::from(place.0) < u128::from(v.0) + u128::from(prior.width)
                    })
            })
        {
            reason = Some(RelocationProblem::Overlap);
            value = None;
        }
        if let Some(reason) = reason {
            b.push(&mut p.blockers, Blocker::Relocation { index: i, reason })?;
            value = None;
        }
        b.push(
            &mut p.relocations,
            RelocationPlan {
                raw: raw.clone(),
                place,
                action,
                value,
                width,
                resolution,
            },
        )?;
    }
    Ok(())
}
