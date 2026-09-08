mod dynamic_fixtures;
use astero_loader::elf::{
    address_translation::TranslationError,
    dynamic::{
        ObservationLimits,
        error::DynamicError,
        string_table::StringTableError,
        symbol_table::{Binding, Section, SymbolError, SymbolTable, SymbolType, Visibility},
    },
};
use dynamic_fixtures::{image, parsed, put16, put32, put64};
fn limits() -> ObservationLimits {
    ObservationLimits { max_entries: 32 }
}
fn fixture() -> Vec<u8> {
    let mut b = image(&[(6, 0x1400), (11, 24), (5, 0x1500), (10, 8), (0, 0)]);
    b[0x500..0x508].copy_from_slice(b"\0hello\0\xff");
    b
}
fn entry(b: &mut [u8], index: usize, name: u32, info: u8, other: u8, section: u16) {
    let at = 0x400 + 24 * index;
    put32(b, at, name);
    b[at + 4] = info;
    b[at + 5] = other;
    put16(b, at + 6, section);
    put64(b, at + 8, 0x123456789abcdef0);
    put64(b, at + 16, 0x1020304050607080);
}
#[test]
fn null_entry_and_unknown_count_do_not_enable_enumeration() {
    let elf = parsed(fixture());
    let table = SymbolTable::new(&elf, limits()).unwrap();
    let s = table.read_candidate(0, 0).unwrap();
    assert_eq!(s.name, None);
    assert_eq!(s.value, 0);
    assert_eq!(s.size, 0);
    assert_eq!(s.section, Section::Undefined);
    assert_eq!(table.symbol_count(), Err(SymbolError::CountUnavailable));
    // Even readable adjacent bytes do not establish total extent or membership.
    assert!(table.read_candidate(2, 8).is_ok());
    assert_eq!(table.symbol_count(), Err(SymbolError::CountUnavailable));
}
#[test]
fn fields_names_and_repeated_observation_preserve_source() {
    let mut b = fixture();
    entry(&mut b, 1, 1, 0x12, 0, 0);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    let s = table.read_candidate(1, 6).unwrap();
    assert_eq!(s, table.read_candidate(1, 6).unwrap());
    assert_eq!(s.name.unwrap().as_bytes(), b"hello");
    assert_eq!(s.binding, Binding::Global);
    assert_eq!(s.symbol_type, SymbolType::Function);
    assert_eq!(s.section, Section::Undefined);
    assert_eq!(s.value, 0x123456789abcdef0);
    assert_eq!(s.size, 0x1020304050607080);
    assert_eq!(
        s.source,
        elf.artifact().source().checked_range(0x418, 24).unwrap()
    );
    assert_eq!(
        s.name.unwrap().source_range(),
        elf.artifact().source().checked_range(0x501, 5).unwrap()
    );
    assert!(elf.artifact().description().imports.is_empty());
    assert!(elf.artifact().description().exports.is_empty());
}
#[test]
fn binding_type_visibility_and_sections_preserve_extensions() {
    for (info, binding, kind) in [
        (0, Binding::Local, SymbolType::NoType),
        (0x11, Binding::Global, SymbolType::Object),
        (0x22, Binding::Weak, SymbolType::Function),
        (0xab, Binding::Unknown(10), SymbolType::Unknown(11)),
    ] {
        for other in 0..=255u8 {
            let mut b = fixture();
            entry(&mut b, 1, 1, info, other, 1);
            put64(&mut b, 0x428, 0);
            let elf = parsed(b);
            let table = SymbolTable::new(&elf, limits()).unwrap();
            let s = table.read_candidate(1, 8).unwrap();
            assert_eq!(s.binding, binding);
            assert_eq!(s.symbol_type, kind);
            assert_eq!(s.other, other);
            assert_eq!(s.size, 0);
            assert_eq!(s.section, Section::Index(1));
            assert_eq!(
                s.visibility,
                match other & 7 {
                    0 => Visibility::Default,
                    1 => Visibility::Internal,
                    2 => Visibility::Hidden,
                    3 => Visibility::Protected,
                    n => Visibility::Unknown(n),
                }
            );
        }
    }
    for (raw, want) in [
        (0, Section::Undefined),
        (0xfff1, Section::Absolute),
        (0xfff2, Section::Common),
        (0xffff, Section::Extended),
        (0xff00, Section::Reserved(0xff00)),
    ] {
        let mut b = fixture();
        entry(&mut b, 1, 0, 0, 0, raw);
        let elf = parsed(b);
        let table = SymbolTable::new(&elf, limits()).unwrap();
        assert_eq!(table.read_candidate(1, 0).unwrap().section, want);
    }
}
#[test]
fn name_failures_are_string_errors_with_symbol_index() {
    for offset in [8, 9, u32::MAX] {
        let mut b = fixture();
        entry(&mut b, 1, offset, 0x12, 0, 1);
        let elf = parsed(b);
        let table = SymbolTable::new(&elf, limits()).unwrap();
        assert!(matches!(
            table.read_candidate(1, 8),
            Err(SymbolError::Name {
                index: 1,
                error: StringTableError::OffsetOutOfBounds { .. }
            })
        ));
    }
    let mut b = fixture();
    entry(&mut b, 1, 7, 0x12, 0, 1);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert!(matches!(
        table.read_candidate(1, 8),
        Err(SymbolError::Name {
            error: StringTableError::MissingTerminator { .. },
            ..
        })
    ));
    let mut b = fixture();
    entry(&mut b, 1, 1, 0x12, 0, 1);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert!(matches!(
        table.read_candidate(1, 5),
        Err(SymbolError::Name {
            error: StringTableError::ScanLimit { .. },
            ..
        })
    ));
}
#[test]
fn byte_names_empty_names_and_no_name_are_distinct() {
    let mut b = fixture();
    b[0x501] = 0xff;
    entry(&mut b, 1, 1, 0, 0, 1);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    let s = table.read_candidate(1, 8).unwrap();
    assert_eq!(s.name.unwrap().as_bytes(), b"\xffello");
    assert!(s.name.unwrap().as_utf8().is_err());
    let mut b = fixture();
    entry(&mut b, 1, 6, 0, 0, 1);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert_eq!(
        table.read_candidate(1, 1).unwrap().name.unwrap().as_bytes(),
        b""
    );
    let mut b = image(&[(6, 0x1400), (11, 24), (0, 0)]);
    entry(&mut b, 1, 0, 0, 0, 1);
    let elf = parsed(b.clone());
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert_eq!(table.read_candidate(1, 0).unwrap().name, None);
    put32(&mut b, 0x418, 1);
    let elf = parsed(b);
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert!(matches!(
        table.read_candidate(1, 8),
        Err(SymbolError::StringsUnavailable {
            index: 1,
            offset: 1
        })
    ));
}
#[test]
fn missing_incomplete_and_unsupported_descriptors_keep_dynamic_boundary() {
    for entries in [vec![(0, 0)], vec![(5, 0x1500), (10, 8), (0, 0)]] {
        let elf = parsed(image(&entries));
        assert!(matches!(
            SymbolTable::new(&elf, limits()),
            Err(SymbolError::TableUnavailable)
        ));
    }
    for entries in [vec![(6, 0x1400), (0, 0)], vec![(11, 24), (0, 0)]] {
        let elf = parsed(image(&entries));
        assert!(matches!(
            SymbolTable::new(&elf, limits()),
            Err(SymbolError::Dynamic(
                DynamicError::IncompleteDescriptor { .. }
            ))
        ));
    }
    for size in [0, 23, 25, u64::MAX] {
        let elf = parsed(image(&[(6, 0x1400), (11, size), (0, 0)]));
        assert!(matches!(
            SymbolTable::new(&elf, limits()),
            Err(SymbolError::Dynamic(
                DynamicError::UnsupportedEntrySize { .. }
            ))
        ));
    }
}
#[test]
fn translation_and_truncation_are_not_fabricated_symbols() {
    for addr in [0x15e9, 0x1600, 0x2000] {
        let elf = parsed(image(&[(6, addr), (11, 24), (0, 0)]));
        assert!(matches!(
            SymbolTable::new(&elf, limits()),
            Err(SymbolError::Dynamic(DynamicError::Translation { .. }))
        ));
    }
    let elf = parsed(fixture());
    let table = SymbolTable::new(&elf, limits()).unwrap();
    assert!(matches!(
        table.read_candidate(21, 8),
        Err(SymbolError::Translation {
            error: TranslationError::CrossesSourceBoundary { .. },
            ..
        })
    ));
    assert!(matches!(
        table.read_candidate(22, 8),
        Err(SymbolError::Translation {
            error: TranslationError::ZeroFill { .. },
            ..
        })
    ));
    for index in [u64::MAX, u64::MAX / 24] {
        assert_eq!(
            table.read_candidate(index, 8),
            Err(SymbolError::IndexOverflow { index })
        );
    }
}
#[test]
fn nonzero_null_entry_is_structurally_invalid() {
    for field in [0, 4, 5, 6, 8, 16] {
        let mut b = fixture();
        b[0x400 + field] = 1;
        let elf = parsed(b);
        let table = SymbolTable::new(&elf, limits()).unwrap();
        assert_eq!(
            table.read_candidate(0, 8),
            Err(SymbolError::InvalidNullSymbol)
        );
    }
}
#[test]
fn source_extent_sweep_requires_all_twenty_four_bytes() {
    for remaining in 0..=25u64 {
        let address = 0x1600 - remaining;
        let elf = parsed(image(&[(6, address), (11, 24), (0, 0)]));
        assert_eq!(SymbolTable::new(&elf, limits()).is_ok(), remaining >= 24);
    }
}
