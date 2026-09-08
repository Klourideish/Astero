mod elf_fixtures;
use astero_loader::{
    admission::{MetadataField, Rejection, admit},
    artifact::{Architecture, ArtifactFamily, DeferredRequirement, SourceError},
    elf::{ElfError, error::Structure, inspect},
    load_plan::plan,
    metadata::VirtualAddress,
    modules::{ArtifactRole, TargetRole},
};
use elf_fixtures::*;
fn parse_error(b: Vec<u8>) -> ElfError {
    inspect(source(b), context()).unwrap_err()
}
fn rejection(b: Vec<u8>) -> Rejection {
    admit(inspect(source(b), context()).unwrap().artifact()).unwrap_err()
}

#[test]
fn header_only_inspects_without_implying_admission() {
    let report = inspect(source(header(0)), context()).unwrap();
    assert_eq!(report.header().program_count, 0);
    assert_eq!(report.program_headers().count(), 0);
    assert_eq!(
        admit(report.artifact()),
        Err(Rejection::MissingMetadata(MetadataField::Regions))
    );
}
#[test]
fn executable_pipeline_preserves_identity_bytes_and_entry() {
    let source = source(executable());
    let id = source.identity();
    let report = inspect(source.clone(), context()).unwrap();
    let before = report.clone();
    let target = admit(report.artifact()).unwrap();
    let result = plan(&target);
    assert_eq!(result, plan(&target));
    assert_eq!(report, before);
    assert_eq!(result.metadata().family, ArtifactFamily::Elf);
    assert_eq!(result.metadata().role, TargetRole::Executable);
    assert_eq!(result.metadata().entry_point, Some(VirtualAddress(0x1000)));
    assert_eq!(report.artifact().identity(), id);
    assert_eq!(result.source().identity(), id);
    let copy = result.mappings()[0].copy.as_ref().unwrap();
    assert_eq!(copy.source.source_id(), id);
    assert_eq!(copy.source.extent().offset.0, 0x100);
    assert_eq!(result.source().read(&copy.source).unwrap(), [0xa5; 16]);
    assert_eq!(
        source.read(&copy.source).unwrap().as_ptr(),
        result.source().read(&copy.source).unwrap().as_ptr()
    );
    assert!(result.metadata().imports.is_empty());
    assert!(result.metadata().relocations.is_empty());
}
#[test]
fn multi_region_plan_preserves_permissions_bss_and_unused_headers() {
    let report = inspect(source(multiple()), context()).unwrap();
    let entries = report
        .program_headers()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1].kind, 0);
    assert_eq!(entries[1].file_offset, u64::MAX);
    assert_eq!(entries[0].physical_address, 0xfeed);
    let result = plan(&admit(report.artifact()).unwrap());
    assert_eq!(result.mappings().len(), 2);
    let rx = &result.mappings()[0];
    let rw = &result.mappings()[1];
    assert!(rx.permissions.read && rx.permissions.execute && !rx.permissions.write);
    assert!(rw.permissions.read && rw.permissions.write && !rw.permissions.execute);
    assert_eq!(rx.alignment, 16);
    assert_eq!(rw.copy.as_ref().unwrap().source.extent().size, 8);
    assert_eq!(rw.zero_fill.unwrap().start, VirtualAddress(0x2008));
    assert_eq!(rw.zero_fill.unwrap().size, 24);
    assert_eq!(result, plan(&admit(report.artifact()).unwrap()));
}
#[test]
fn shared_object_and_zero_alignment_are_observed_without_base_placement() {
    let mut b = executable();
    put16(&mut b, 16, 3);
    put64(&mut b, 24, 0);
    put64(&mut b, 112, 0);
    let report = inspect(source(b), context()).unwrap();
    assert_eq!(report.header().object_type, 3);
    assert_eq!(
        report.program_headers().next().unwrap().unwrap().alignment,
        0
    );
    let target = admit(report.artifact()).unwrap();
    assert_eq!(target.metadata().role, TargetRole::Module);
    assert_eq!(target.metadata().entry_point, None);
    assert_eq!(target.regions()[0].alignment, 1);
    assert_eq!(target.regions()[0].range.start, VirtualAddress(0x1000));
}
#[test]
fn identification_errors_are_distinct_from_truncation() {
    assert!(matches!(
        parse_error(vec![0; 15]),
        ElfError::Source {
            structure: Structure::Identification,
            ..
        }
    ));
    let mut b = executable();
    b[0] = 0;
    assert_eq!(
        parse_error(b),
        ElfError::InvalidMagic {
            observed: [0, b'E', b'L', b'F']
        }
    );
    for (at, value, error) in [
        (4, 1, ElfError::UnsupportedClass(1)),
        (5, 2, ElfError::UnsupportedByteOrder(2)),
        (
            6,
            2,
            ElfError::UnsupportedVersion {
                structure: Structure::Identification,
                version: 2,
            },
        ),
    ] {
        let mut b = executable();
        b[at] = value;
        assert_eq!(parse_error(b), error);
    }
}
#[test]
fn header_sizes_versions_and_deferred_section_fields() {
    let mut b = header(0);
    b.truncate(63);
    assert!(matches!(
        parse_error(b),
        ElfError::Source {
            structure: Structure::Header,
            ..
        }
    ));
    for size in [0, 63, 65, u16::MAX] {
        let mut b = header(0);
        put16(&mut b, 52, size);
        assert_eq!(parse_error(b), ElfError::UnsupportedHeaderSize(size));
    }
    let mut b = header(0);
    put32(&mut b, 20, 2);
    assert_eq!(
        parse_error(b),
        ElfError::UnsupportedVersion {
            structure: Structure::Header,
            version: 2
        }
    );
    let mut b = executable();
    put64(&mut b, 40, u64::MAX);
    put16(&mut b, 58, 123);
    put16(&mut b, 60, 456);
    put16(&mut b, 62, 0xffff);
    let report = inspect(source(b), context()).unwrap();
    let h = report.header();
    assert_eq!(
        (
            h.section_offset,
            h.section_entry_size,
            h.section_count,
            h.section_name_index
        ),
        (u64::MAX, 123, 456, 0xffff)
    );
    assert!(admit(report.artifact()).is_ok()); // Section metadata is not followed or certified.
}
#[test]
fn table_shape_overflow_and_extended_count_are_structured() {
    for offset in [0, 1, 63] {
        let mut b = executable();
        put64(&mut b, 32, offset);
        assert_eq!(
            parse_error(b),
            ElfError::InvalidTableOffset { offset, count: 1 }
        );
    }
    let mut b = header(0);
    put64(&mut b, 32, 64);
    assert_eq!(
        parse_error(b),
        ElfError::InvalidTableOffset {
            offset: 64,
            count: 0
        }
    );
    for size in [0, 55, 57, u16::MAX] {
        let mut b = executable();
        put16(&mut b, 54, size);
        assert_eq!(parse_error(b), ElfError::UnsupportedProgramHeaderSize(size));
    }
    let mut b = executable();
    put16(&mut b, 56, 0xffff);
    assert_eq!(parse_error(b), ElfError::UnsupportedExtendedProgramCount);
    let mut b = executable();
    put64(&mut b, 32, u64::MAX - 55);
    assert_eq!(
        parse_error(b),
        ElfError::TableOverflow {
            offset: u64::MAX - 55,
            count: 1,
            entry_size: 56
        }
    );
    let mut b = executable();
    put16(&mut b, 56, 65534);
    put64(&mut b, 32, u64::MAX - 100);
    assert!(matches!(
        parse_error(b),
        ElfError::TableOverflow { count: 65534, .. }
    ));
}
#[test]
fn incomplete_individual_entries_fail_at_whole_table_boundary() {
    for length in [64, 65, 119] {
        let mut b = executable();
        b.truncate(length);
        let e = parse_error(b);
        assert!(matches!(
            e,
            ElfError::Source {
                structure: Structure::ProgramHeaderTable,
                error: SourceError::LengthOutOfBounds { .. }
            }
        ));
        assert!(std::error::Error::source(&e).is_some());
    }
    let mut b = executable();
    put64(&mut b, 32, 0x111);
    assert!(matches!(
        parse_error(b),
        ElfError::Source {
            structure: Structure::ProgramHeaderTable,
            error: SourceError::OffsetOutOfBounds { .. }
        }
    ));
}
#[test]
fn region_defects_reach_admission_instead_of_byte_errors() {
    let mut b = executable();
    put64(&mut b, 72, 0x110);
    assert!(matches!(
        rejection(b),
        Rejection::SourceRange {
            region: 0,
            error: SourceError::LengthOutOfBounds { .. }
        }
    ));
    let mut b = executable();
    put64(&mut b, 96, 17);
    assert!(matches!(
        rejection(b),
        Rejection::InvalidSize { region: 0, .. }
    ));
    let mut b = executable();
    put64(&mut b, 112, 3);
    assert!(matches!(
        rejection(b),
        Rejection::InvalidAlignment { region: 0, .. }
    ));
    let mut b = executable();
    put64(&mut b, 24, 0x1010);
    assert_eq!(
        rejection(b),
        Rejection::InvalidEntryPoint {
            address: Some(0x1010)
        }
    );
    let mut b = executable();
    put64(&mut b, 24, 0);
    assert_eq!(rejection(b), Rejection::InvalidEntryPoint { address: None });
    let mut b = multiple();
    put64(&mut b, 80, 0x1000);
    assert!(matches!(
        rejection(b),
        Rejection::OverlappingRegions {
            first: 0,
            second: 1
        }
    ));
    let mut b = executable();
    put64(&mut b, 80, u64::MAX);
    assert_eq!(rejection(b), Rejection::VirtualRangeOverflow { region: 0 });
}
#[test]
fn classifications_and_missing_context_are_admission_decisions() {
    for (machine, arch) in [
        (183, Architecture::Aarch64),
        (0xffff, Architecture::Unknown),
    ] {
        let mut b = executable();
        put16(&mut b, 18, machine);
        let r = inspect(source(b), context()).unwrap();
        assert_eq!(r.header().machine, machine);
        assert_eq!(
            admit(r.artifact()),
            Err(Rejection::UnsupportedArchitecture(arch))
        );
    }
    for role in [0, 1, 4, 0xfe00] {
        let mut b = executable();
        put16(&mut b, 16, role);
        assert_eq!(
            rejection(b),
            Rejection::UnsupportedRole(ArtifactRole::Unknown)
        );
    }
    let r = inspect(source(executable()), None).unwrap();
    assert_eq!(
        admit(r.artifact()),
        Err(Rejection::MissingMetadata(MetadataField::Module))
    );
}
#[test]
fn uninspected_semantics_cannot_silently_become_a_complete_plan() {
    for kind in [2, 3, 4, 6, 7, 0x60000000, 0xffffffff] {
        let mut b = multiple();
        put32(&mut b, 120, kind);
        let r = inspect(source(b), context()).unwrap();
        assert_eq!(r.program_headers().nth(1).unwrap().unwrap().kind, kind);
        assert_eq!(
            admit(r.artifact()),
            Err(Rejection::UnsupportedRequirement(
                DeferredRequirement::UninspectedProgramSemantics
            ))
        );
    }
    for (at, value) in [(7, 9), (8, 1), (48, 1), (68, 0x85)] {
        let mut b = executable();
        b[at] = value;
        assert_eq!(
            rejection(b),
            Rejection::UnsupportedRequirement(DeferredRequirement::PlatformSemantics)
        );
    }
}
#[test]
fn source_lengths_around_identification_header_and_table_boundaries() {
    for length in 0..=128 {
        let mut b = executable();
        b.truncate(length);
        let result = inspect(source(b), context());
        match length {
            0..=15 => assert!(matches!(
                result,
                Err(ElfError::Source {
                    structure: Structure::Identification,
                    ..
                })
            )),
            16..=63 => assert!(matches!(
                result,
                Err(ElfError::Source {
                    structure: Structure::Header,
                    ..
                })
            )),
            64..=119 => assert!(matches!(
                result,
                Err(ElfError::Source {
                    structure: Structure::ProgramHeaderTable,
                    ..
                })
            )),
            _ => assert!(matches!(
                admit(result.unwrap().artifact()),
                Err(Rejection::SourceRange { .. })
            )),
        }
    }
}
#[test]
fn table_extent_sweep_matches_checked_arithmetic_oracle() {
    for offset in [
        0,
        1,
        63,
        64,
        65,
        119,
        120,
        200,
        272,
        u64::MAX - 56,
        u64::MAX - 55,
        u64::MAX,
    ] {
        for count in [0, 1, 2, 3, 65534, 65535] {
            let mut b = executable();
            put64(&mut b, 32, offset);
            put16(&mut b, 56, count);
            let result = inspect(source(b), context());
            if count == 65535 {
                assert_eq!(
                    result.unwrap_err(),
                    ElfError::UnsupportedExtendedProgramCount
                );
            } else if (count == 0 && offset != 0) || (count != 0 && offset < 64) {
                assert!(matches!(result, Err(ElfError::InvalidTableOffset { .. })));
            } else if offset.checked_add(u64::from(count) * 56).is_none() {
                assert!(matches!(result, Err(ElfError::TableOverflow { .. })));
            } else if offset + u64::from(count) * 56 > 272 {
                assert!(matches!(
                    result,
                    Err(ElfError::Source {
                        structure: Structure::ProgramHeaderTable,
                        ..
                    })
                ));
            } else {
                assert!(result.is_ok());
            }
        }
    }
}
#[test]
fn source_segment_sweep_matches_actual_bounds_and_zero_fill() {
    for offset in 268..=274 {
        for size in 0..=6 {
            let mut b = executable();
            put16(&mut b, 16, 3);
            put64(&mut b, 24, 0);
            put64(&mut b, 72, offset);
            put64(&mut b, 96, size);
            let r = inspect(source(b), context()).unwrap();
            let t = admit(r.artifact());
            if offset + size <= 272 {
                let p = plan(&t.unwrap());
                let m = &p.mappings()[0];
                assert_eq!(m.copy.as_ref().map_or(0, |c| c.source.extent().size), size);
                assert_eq!(m.zero_fill.unwrap().size, 16 - size);
            } else {
                assert!(matches!(t, Err(Rejection::SourceRange { .. })));
            }
        }
    }
}
#[test]
fn maximum_nonextended_table_is_iterated_without_untrusted_reservation() {
    let b = header(65534);
    let r = inspect(source(b), context()).unwrap();
    assert_eq!(r.program_headers().count(), 65534);
    assert!(r.artifact().description().regions.is_empty());
}
