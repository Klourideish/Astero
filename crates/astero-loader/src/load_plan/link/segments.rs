use super::*;
use crate::{
    elf::inspect::bounded::InspectionOutcome,
    load_plan::{CopyIntent, MappingIntent},
    metadata::{AddressRange, Permissions, VirtualAddress},
};
pub(super) fn collect(p: &mut GuestLoadPlan, b: &mut Budget) -> Result<(), PlanningError> {
    let InspectionOutcome::Complete(h) = p.headers.outcome() else {
        return b.push(&mut p.blockers, Blocker::EvidenceUnavailable);
    };
    if h.header().machine != 62 {
        b.push(&mut p.blockers, Blocker::Machine(h.header().machine))?;
    }
    if !matches!(h.header().object_type, 2 | 3 | 0xfe00 | 0xfe10 | 0xfe18) {
        b.push(&mut p.blockers, Blocker::ObjectType(h.header().object_type))?;
    }
    p.experimental |= h.header().object_type >= 0xfe00
        || h.header().identification.os_abi != 0
        || h.header().flags != 0;
    for (index, raw) in h.program_headers().iter().enumerate() {
        if raw.kind != 1 {
            // Empty TLS advertises no template or storage to instantiate.
            if raw.kind == 7 && raw.file_size == 0 && raw.memory_size == 0 {
                continue;
            }
            if !matches!(
                raw.kind,
                0 | 2 | 4 | 6 | 0x6474e550 | 0x6474e551 | 0x61000000
            ) {
                b.push(
                    &mut p.blockers,
                    Blocker::ProgramSemantics {
                        index,
                        kind: raw.kind,
                    },
                )?;
            }
            continue;
        }
        let a = raw.alignment.max(1);
        let reason = if raw.file_size > raw.memory_size || raw.memory_size == 0 {
            Some(SegmentProblem::Size)
        } else if !a.is_power_of_two()
            || raw.virtual_address % a != raw.file_offset % a
            || !p.bias.0.is_multiple_of(a)
        {
            Some(SegmentProblem::Alignment)
        } else if raw.flags & !7 != 0 {
            Some(SegmentProblem::Flags)
        } else {
            None
        };
        if let Some(reason) = reason {
            b.push(&mut p.blockers, Blocker::Segment { index, reason })?;
            continue;
        }
        let Some(start) = p.bias.0.checked_add(raw.virtual_address) else {
            b.push(
                &mut p.blockers,
                Blocker::Segment {
                    index,
                    reason: SegmentProblem::Overflow,
                },
            )?;
            continue;
        };
        if start.checked_add(raw.memory_size).is_none() {
            b.push(
                &mut p.blockers,
                Blocker::Segment {
                    index,
                    reason: SegmentProblem::Overflow,
                },
            )?;
            continue;
        }
        let source = p
            .input
            .linkage()
            .source()
            .checked_range(raw.file_offset, raw.file_size)
            .map_err(|error| PlanningError::Source {
                source_id: p.input.linkage().source().identity(),
                segment: index,
                error,
            })?;
        let mapping = MappingIntent {
            range: AddressRange {
                start: VirtualAddress(start),
                size: raw.memory_size,
            },
            alignment: a,
            permissions: Permissions {
                read: raw.flags & 4 != 0,
                write: raw.flags & 2 != 0,
                execute: raw.flags & 1 != 0,
            },
            copy: (raw.file_size != 0).then_some(CopyIntent {
                source,
                destination: VirtualAddress(start),
            }),
            zero_fill: (raw.memory_size > raw.file_size).then_some(AddressRange {
                start: VirtualAddress(start + raw.file_size),
                size: raw.memory_size - raw.file_size,
            }),
        };
        for prior in &p.segments {
            let r = prior.mapping.range;
            if start < r.start.0 + r.size && r.start.0 < start + raw.memory_size {
                b.push(
                    &mut p.blockers,
                    Blocker::SegmentOverlap {
                        first: prior.program_index,
                        second: index,
                    },
                )?;
            }
        }
        b.push(
            &mut p.segments,
            SegmentPlan {
                program_index: index,
                raw: raw.clone(),
                mapping,
            },
        )?;
    }
    if p.segments.is_empty() {
        b.push(&mut p.blockers, Blocker::NoSegments)?;
    }
    p.entry = if h.header().entry == 0 {
        None
    } else {
        p.bias.0.checked_add(h.header().entry).map(VirtualAddress)
    };
    if let Some(entry) = p.entry {
        if !p.segments.iter().any(|s| {
            s.mapping.permissions.execute
                && entry.0 >= s.mapping.range.start.0
                && entry.0 - s.mapping.range.start.0 < s.raw.file_size
        }) {
            b.push(&mut p.blockers, Blocker::EntryOutsideExecutable)?;
        }
    } else {
        b.push(&mut p.blockers, Blocker::EntryUnavailable)?;
    }
    Ok(())
}
