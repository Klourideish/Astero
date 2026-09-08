mod dynamic_fixtures;
use DynamicTag::*;
use astero_loader::{
    admission::{Rejection, admit},
    artifact::{DeferredRequirement, SourceArtifact, SourceError},
    elf::{
        self, ElfError,
        address_translation::{AddressTranslator, TranslationError},
        dynamic::{
            DynamicObservation, DynamicTable, ObservationLimits, error::DynamicError,
            observation::PltRelocationKind, observe, tags::DynamicTag,
        },
    },
    metadata::VirtualAddress,
};
use dynamic_fixtures::*;
fn observation(b: Vec<u8>) -> Result<DynamicObservation, DynamicError> {
    observe(&parsed(b), ObservationLimits { max_entries: 128 })
}
fn present(b: Vec<u8>) -> DynamicTable {
    match observation(b).unwrap() {
        DynamicObservation::Present(table) => *table,
        DynamicObservation::Absent => panic!("expected table"),
    }
}
fn single(tag: i64, value: u64) -> Vec<u8> {
    image(&[(tag, value), (0, 0)])
}

#[test]
fn translation_preserves_start_interior_end_and_source_identity() {
    let elf = parsed(image(&[(0, 0)]));
    let t = AddressTranslator::new(&elf).unwrap();
    for (address, size, offset) in [
        (0x1100, 1, 0x100),
        (0x1110, 8, 0x110),
        (0x15ff, 1, 0x5ff),
        (0x1600, 0, 0x600),
    ] {
        let r = t.translate(VirtualAddress(address), size).unwrap();
        assert_eq!(r.extent().offset.0, offset);
        assert_eq!(r.extent().size, size);
        assert_eq!(r.source_id(), elf.artifact().identity());
        assert_eq!(elf.artifact().source().read(&r).unwrap().len() as u64, size);
        assert_eq!(r, t.translate(VirtualAddress(address), size).unwrap());
    }
}
#[test]
fn zero_fill_crossings_and_unmapped_ranges_never_manufacture_source_bytes() {
    let elf = parsed(image(&[(0, 0)]));
    let t = AddressTranslator::new(&elf).unwrap();
    for (address, size) in [(0x1600, 1), (0x1601, 8), (0x16ff, 1), (0x1700, 0)] {
        assert!(matches!(
            t.translate(VirtualAddress(address), size),
            Err(TranslationError::ZeroFill { .. })
        ));
    }
    assert!(matches!(
        t.translate(VirtualAddress(0x15ff), 2),
        Err(TranslationError::CrossesSourceBoundary { .. })
    ));
    assert!(matches!(
        t.translate(VirtualAddress(0x16ff), 2),
        Err(TranslationError::CrossesMapping { .. })
    ));
    for address in [0x1000, 0x1700, u64::MAX] {
        assert!(matches!(
            t.translate(VirtualAddress(address), 1),
            Err(TranslationError::Unmapped { .. } | TranslationError::RequestOverflow { .. })
        ));
    }
    assert!(matches!(
        t.translate(VirtualAddress(u64::MAX), 1),
        Err(TranslationError::RequestOverflow {
            address: u64::MAX,
            size: 1
        })
    ));
}
#[test]
fn separate_load_segments_translate_but_ranges_cannot_be_stitched() {
    let mut b = image(&[(0, 0)]);
    program(&mut b, 1, 1, 0x40, 0x3000, 32, 48);
    let elf = parsed(b);
    let t = AddressTranslator::new(&elf).unwrap();
    assert_eq!(
        t.translate(VirtualAddress(0x3008), 8)
            .unwrap()
            .extent()
            .offset
            .0,
        0x48
    );
    assert!(matches!(
        t.translate(VirtualAddress(0x3020), 1),
        Err(TranslationError::ZeroFill { segment: 1, .. })
    ));
    let mut b = image(&[(0, 0)]);
    program(&mut b, 1, 1, 0x40, 0x1700, 32, 32);
    let elf = parsed(b);
    let t = AddressTranslator::new(&elf).unwrap();
    assert!(matches!(
        t.translate(VirtualAddress(0x16ff), 2),
        Err(TranslationError::CrossesMapping { .. })
    ));
    assert!(matches!(
        t.translate(VirtualAddress(0x1700), 0),
        Err(TranslationError::Ambiguous { .. })
    ));
    assert_eq!(
        t.translate(VirtualAddress(0x1700), 1)
            .unwrap()
            .extent()
            .offset
            .0,
        0x40
    );
}
#[test]
fn identical_and_partial_overlaps_are_ambiguous_including_mid_request() {
    for (start, address, size) in [
        (0x1100, 0x1100, 1),
        (0x1150, 0x1140, 32),
        (0x1150, 0x1150, 1),
    ] {
        let mut b = image(&[(0, 0)]);
        program(&mut b, 1, 1, 0x100, start, 64, 64);
        let elf = parsed(b);
        let t = AddressTranslator::new(&elf).unwrap();
        assert_eq!(
            t.translate(VirtualAddress(address), size),
            Err(TranslationError::Ambiguous {
                first: 0,
                second: 1,
                address,
                size
            })
        );
    }
}
#[test]
fn translator_rejects_malformed_load_extents_before_queries() {
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 96, 0x601);
    let elf = parsed(b);
    assert!(matches!(
        AddressTranslator::new(&elf),
        Err(TranslationError::InvalidLoadSize { segment: 0, .. })
    ));
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 80, u64::MAX);
    let elf = parsed(b);
    assert!(matches!(
        AddressTranslator::new(&elf),
        Err(TranslationError::LoadRangeOverflow { segment: 0, .. })
    ));
    for (offset, overflow) in [(u64::MAX, true), (0x601, false)] {
        let mut b = image(&[(0, 0)]);
        put64(&mut b, 72, offset);
        let elf = parsed(b);
        let error = AddressTranslator::new(&elf).err().unwrap();
        assert!(std::error::Error::source(&error).is_some());
        if overflow {
            assert!(matches!(
                error,
                TranslationError::Source {
                    error: SourceError::RangeOverflow { .. },
                    ..
                }
            ));
        } else {
            assert!(matches!(
                error,
                TranslationError::Source {
                    error: SourceError::OffsetOutOfBounds { .. },
                    ..
                }
            ));
        }
    }
}
#[test]
fn bounded_load_extent_sweep_matches_independent_interval_oracle() {
    for memory in 0..=8 {
        for file in 0..=memory {
            let mut b = image(&[(0, 0)]);
            program(&mut b, 0, 1, 0x100, 0x1100, file, memory);
            let elf = parsed(b);
            let t = AddressTranslator::new(&elf).unwrap();
            for offset in 0..=10 {
                for size in 0..=10 {
                    let r = t.translate(VirtualAddress(0x1100 + offset), size);
                    if offset > memory || (offset == memory && size != 0) {
                        assert!(matches!(r, Err(TranslationError::Unmapped { .. })));
                    } else if offset + size > memory {
                        assert!(matches!(r, Err(TranslationError::CrossesMapping { .. })));
                    } else if offset > file || (offset == file && size != 0) {
                        assert!(matches!(r, Err(TranslationError::ZeroFill { .. })));
                    } else if offset + size > file {
                        assert!(matches!(
                            r,
                            Err(TranslationError::CrossesSourceBoundary { .. })
                        ));
                    } else {
                        let r = r.unwrap();
                        assert_eq!(r.extent().offset.0, 0x100 + offset);
                        assert_eq!(elf.artifact().source().read(&r).unwrap().len() as u64, size);
                    }
                }
            }
        }
    }
}
#[test]
fn absent_dynamic_table_is_explicit_and_does_not_trigger_translation() {
    let mut b = image(&[]);
    put32(&mut b, 120, 0);
    let elf = parsed(b);
    assert_eq!(
        observe(&elf, ObservationLimits { max_entries: 0 }),
        Ok(DynamicObservation::Absent)
    );
    assert!(admit(elf.artifact()).is_ok());
}
#[test]
fn null_termination_retains_source_lifetime_and_does_not_change_admission() {
    let elf = parsed(image(&[(0, 77), (5, u64::MAX)]));
    let before = elf.clone();
    let observed = observe(&elf, ObservationLimits { max_entries: 1 }).unwrap();
    assert_eq!(
        observed,
        observe(&elf, ObservationLimits { max_entries: 1 }).unwrap()
    );
    assert_eq!(elf, before);
    assert_eq!(
        admit(elf.artifact()),
        Err(Rejection::UnsupportedRequirement(
            DeferredRequirement::UninspectedProgramSemantics
        ))
    );
    let id = elf.artifact().identity();
    drop(elf);
    drop(before);
    let DynamicObservation::Present(table) = observed else {
        panic!()
    };
    assert_eq!(table.source().identity(), id);
    assert_eq!(table.source_range().source_id(), id);
    assert_eq!(table.entries().len(), 1);
    assert_eq!(table.entries()[0].value, 77);
    assert_eq!(table.program_index(), 1);
    assert_eq!(
        table.source().read(&table.source_range()).unwrap().len(),
        32
    );
    assert!(table.descriptors().strings.is_none());
}
#[test]
fn paired_descriptors_and_repeated_needed_offsets_are_observed_without_interpretation() {
    let table = present(image(&[
        (5, 0x1400),
        (10, 32),
        (1, 0),
        (1, 31),
        (1, 0),
        (6, 0x1420),
        (11, 24),
        (7, 0x1450),
        (8, 48),
        (9, 24),
        (23, 0x1490),
        (2, 24),
        (20, 7),
        (12, u64::MAX),
        (13, 0),
        (25, 0x14c0),
        (27, 16),
        (26, 0x14d0),
        (28, 8),
        (0, 0),
    ]));
    let d = table.descriptors();
    assert_eq!(d.needed_offsets, [0, 31, 0]);
    let strings = d.strings.as_ref().unwrap();
    assert_eq!(strings.source.extent().offset.0, 0x400);
    assert_eq!(strings.size, 32);
    let sym = d.symbols.as_ref().unwrap();
    assert_eq!(sym.first_entry.extent().size, 24);
    assert_eq!(sym.first_entry.extent().offset.0, 0x420);
    let rela = d.rela.as_ref().unwrap();
    assert_eq!(rela.entry_size, Some(24));
    assert_eq!(rela.size, 48);
    assert_eq!(d.plt.as_ref().unwrap().kind, PltRelocationKind::Rela);
    assert_eq!(d.init, Some(VirtualAddress(u64::MAX)));
    assert_eq!(d.fini, Some(VirtualAddress(0)));
    assert_eq!(d.init_array.as_ref().unwrap().entry_size, Some(8));
    assert_eq!(d.fini_array.as_ref().unwrap().size, 8);
    for r in [
        strings.source,
        sym.first_entry,
        rela.source,
        d.init_array.as_ref().unwrap().source,
    ] {
        assert_eq!(r.source_id(), table.source().identity());
        assert!(table.source().read(&r).is_ok());
    }
}
#[test]
fn unknown_tags_preserve_signed_bits_order_duplicates_and_values() {
    let table = present(image(&[
        (0x60000001, 0xfedc),
        (0x70000001, u64::MAX),
        (-1, 5),
        (-1, 6),
        (0, 0),
    ]));
    assert_eq!(
        table
            .entries()
            .iter()
            .map(|e| (e.tag, e.value))
            .collect::<Vec<_>>(),
        vec![
            (Unknown(0x60000001), 0xfedc),
            (Unknown(0x70000001), u64::MAX),
            (Unknown(-1), 5),
            (Unknown(-1), 6),
            (Null, 0)
        ]
    );
    assert!(table.descriptors().strings.is_none());
}
#[test]
fn raw_elf_and_dynamic_source_failures_remain_separate() {
    let mut b = image(&[(0, 0)]);
    b.truncate(119);
    assert!(matches!(
        elf::inspect(SourceArtifact::new(b, None).unwrap(), None),
        Err(ElfError::Source { .. })
    ));
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 128, 0x600);
    let error = observation(b).unwrap_err();
    assert!(matches!(
        error,
        DynamicError::Source {
            segment: 1,
            error: SourceError::LengthOutOfBounds { .. }
        }
    ));
    assert!(std::error::Error::source(&error).is_some());
}
#[test]
fn dynamic_virtual_range_requires_actual_backing_and_agreeing_file_offset() {
    for (address, size, case) in [(0x1600, 16, 0), (0x15ff, 16, 1), (0x3000, 16, 2)] {
        let mut b = image(&[(0, 0)]);
        put64(&mut b, 136, address);
        put64(&mut b, 152, size);
        put64(&mut b, 160, size);
        let e = observation(b).unwrap_err();
        match case {
            0 => assert!(matches!(
                e,
                DynamicError::Translation {
                    tag: None,
                    error: TranslationError::ZeroFill { .. }
                }
            )),
            1 => assert!(matches!(
                e,
                DynamicError::Translation {
                    error: TranslationError::CrossesSourceBoundary { .. },
                    ..
                }
            )),
            _ => assert!(matches!(
                e,
                DynamicError::Translation {
                    error: TranslationError::Unmapped { .. },
                    ..
                }
            )),
        }
    }
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 136, 0x1210);
    assert_eq!(
        observation(b),
        Err(DynamicError::TableOffsetMismatch {
            segment: 1,
            declared: 0x200,
            translated: 0x210
        })
    );
}
#[test]
fn multiple_tables_and_invalid_dynamic_extent_are_rejected_explicitly() {
    let mut b = image(&[(0, 0)]);
    put16(&mut b, 56, 3);
    program(&mut b, 2, 2, 0x200, 0x1200, 16, 16);
    assert_eq!(
        observation(b),
        Err(DynamicError::MultipleTables {
            first: 1,
            second: 2
        })
    );
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 160, 15);
    assert!(matches!(
        observation(b),
        Err(DynamicError::InvalidTableSize { segment: 1, .. })
    ));
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 136, u64::MAX);
    assert!(matches!(
        observation(b),
        Err(DynamicError::TableAddressOverflow { segment: 1, .. })
    ));
}
#[test]
fn dynamic_entry_size_sweep_distinguishes_exhaustion_and_partial_entries() {
    for size in 0..=49 {
        let mut b = image(&[]);
        b[0x200..0x240].fill(0x5a);
        put64(&mut b, 152, size);
        put64(&mut b, 160, size);
        let e = observation(b).unwrap_err();
        if size % 16 == 0 {
            assert_eq!(e, DynamicError::MissingTerminator { entries: size / 16 });
        } else {
            assert_eq!(
                e,
                DynamicError::TruncatedEntry {
                    index: size / 16,
                    remaining: size % 16
                }
            );
        }
    }
    // A partial tail after DT_NULL is not another entry and is never decoded.
    let mut b = image(&[(0, 0)]);
    put64(&mut b, 152, 17);
    put64(&mut b, 160, 17);
    assert_eq!(present(b).entries().len(), 1);
}
#[test]
fn caller_entry_budget_includes_terminator_without_count_based_allocation() {
    let elf = parsed(image(&[(-1, 1), (-1, 2), (0, 0)]));
    for limit in 0..3 {
        assert_eq!(
            observe(&elf, ObservationLimits { max_entries: limit }),
            Err(DynamicError::EntryLimit { limit })
        );
    }
    assert!(observe(&elf, ObservationLimits { max_entries: 3 }).is_ok());
    let elf = parsed(image(&[(-1, 1), (-1, 2)]));
    assert_eq!(
        observe(&elf, ObservationLimits { max_entries: 2 }),
        Err(DynamicError::MissingTerminator { entries: 2 })
    );
}
#[test]
fn singleton_duplicates_do_not_silently_overwrite_metadata() {
    for tag in [5, 10, 6, 11, 7, 8, 9, 23, 2, 20, 12, 13, 25, 27, 26, 28] {
        let e = observation(image(&[(tag, 1), (tag, 1), (0, 0)])).unwrap_err();
        assert_eq!(
            e,
            DynamicError::DuplicateTag {
                tag: DynamicTag::from_raw(tag),
                first: 0,
                second: 1
            }
        );
    }
}
#[test]
fn incomplete_descriptor_groups_are_never_filled_with_invented_defaults() {
    for tag in [5, 10, 6, 11, 7, 8, 9, 23, 2, 20, 25, 27, 26, 28] {
        assert!(matches!(
            observation(single(tag, 0x1400)),
            Err(DynamicError::IncompleteDescriptor { .. })
        ));
    }
    assert_eq!(
        observation(single(1, 0)),
        Err(DynamicError::IncompleteDescriptor {
            present: Needed,
            missing: StrTab
        })
    );
}
#[test]
fn descriptor_entry_sizes_and_array_divisibility_are_checked() {
    for width in [0, 16, 32, u64::MAX] {
        assert_eq!(
            observation(image(&[(6, 0x1400), (11, width), (0, 0)])),
            Err(DynamicError::UnsupportedEntrySize {
                tag: SymEnt,
                observed: width,
                expected: 24
            })
        );
        assert_eq!(
            observation(image(&[(7, 0x1400), (8, 24), (9, width), (0, 0)])),
            Err(DynamicError::UnsupportedEntrySize {
                tag: RelaEnt,
                observed: width,
                expected: 24
            })
        );
    }
    for size in [1, 23, 25] {
        assert_eq!(
            observation(image(&[(7, 0x1400), (8, size), (9, 24), (0, 0)])),
            Err(DynamicError::InvalidDescriptorSize {
                tag: Rela,
                size,
                entry_size: 24
            })
        );
    }
    assert!(matches!(
        observation(image(&[(25, 0x1400), (27, 7), (0, 0)])),
        Err(DynamicError::InvalidDescriptorSize { tag: InitArray, .. })
    ));
    assert_eq!(
        observation(image(&[(23, 0x1400), (2, 24), (20, 99), (0, 0)])),
        Err(DynamicError::UnsupportedPltRelocationKind(99))
    );
}
#[test]
fn descriptor_pointers_retain_translation_error_context() {
    for (address, size) in [
        (0x15ff, 2),
        (0x1600, 1),
        (0x3000, 1),
        (u64::MAX, 1),
        (1, u64::MAX),
    ] {
        let elf = parsed(image(&[(5, address), (10, size), (0, 0)]));
        let error = AddressTranslator::new(&elf)
            .unwrap()
            .translate(VirtualAddress(address), size)
            .unwrap_err();
        assert_eq!(
            observe(&elf, ObservationLimits { max_entries: 128 }),
            Err(DynamicError::Translation {
                tag: Some(StrTab),
                error
            })
        );
    }
    assert!(matches!(
        observation(image(&[(6, 0x15f0), (11, 24), (0, 0)])),
        Err(DynamicError::Translation {
            tag: Some(SymTab),
            error: TranslationError::CrossesSourceBoundary { .. }
        })
    ));
}
#[test]
fn plt_descriptors_distinguish_rel_and_rela_without_reading_entries() {
    for (value, width, kind) in [
        (17, 16, PltRelocationKind::Rel),
        (7, 24, PltRelocationKind::Rela),
    ] {
        let table = present(image(&[(23, 0x1400), (2, width * 2), (20, value), (0, 0)]));
        let plt = table.descriptors().plt.as_ref().unwrap();
        assert_eq!(plt.kind, kind);
        assert_eq!(plt.table.entry_size, Some(width));
        assert_eq!(plt.table.source.extent().size, width * 2);
    }
}
#[test]
fn needed_offsets_are_bounded_but_names_are_not_read() {
    assert_eq!(
        observation(image(&[(5, 0x1400), (10, 8), (1, 8), (0, 0)])),
        Err(DynamicError::NeededOffsetOutOfBounds {
            index: 0,
            offset: 8,
            string_size: 8
        })
    );
    let mut b = image(&[(5, 0x1400), (10, 8), (1, 0), (0, 0)]);
    b[0x400..0x408].fill(0xff);
    let t = present(b);
    assert_eq!(t.descriptors().needed_offsets, [0]);
}
#[test]
fn zero_size_descriptors_validate_empty_source_range_at_file_end() {
    let t = present(image(&[
        (5, 0x1600),
        (10, 0),
        (25, 0x1600),
        (27, 0),
        (7, 0x1600),
        (8, 0),
        (9, 24),
        (0, 0),
    ]));
    for r in [
        t.descriptors().strings.as_ref().unwrap(),
        t.descriptors().init_array.as_ref().unwrap(),
        t.descriptors().rela.as_ref().unwrap(),
    ] {
        assert_eq!(r.source.extent().offset.0, 0x600);
        assert_eq!(r.source.extent().size, 0);
    }
}
#[test]
fn pointer_size_sweep_uses_translation_without_wrapping() {
    for address in [0x15fe, 0x15ff, 0x1600, 0x1601, u64::MAX] {
        for size in [0, 1, 2, 24, u64::MAX] {
            let result = observation(image(&[(5, address), (10, size), (0, 0)]));
            if address.checked_add(size).is_none() {
                assert!(matches!(
                    result,
                    Err(DynamicError::Translation {
                        error: TranslationError::RequestOverflow { .. },
                        ..
                    })
                ));
            } else if address > 0x1700 {
                assert!(matches!(
                    result,
                    Err(DynamicError::Translation {
                        error: TranslationError::Unmapped { .. },
                        ..
                    })
                ));
            } else if address + size > 0x1700 {
                assert!(matches!(
                    result,
                    Err(DynamicError::Translation {
                        error: TranslationError::CrossesMapping { .. },
                        ..
                    })
                ));
            } else if address > 0x1600 || (address == 0x1600 && size > 0) {
                assert!(matches!(
                    result,
                    Err(DynamicError::Translation {
                        error: TranslationError::ZeroFill { .. },
                        ..
                    })
                ));
            } else if address + size > 0x1600 {
                assert!(matches!(
                    result,
                    Err(DynamicError::Translation {
                        error: TranslationError::CrossesSourceBoundary { .. },
                        ..
                    })
                ));
            } else {
                assert!(result.is_ok());
            }
        }
    }
}
