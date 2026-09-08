mod fixtures;
use astero_loader::{
    admission::{Rejection, admit},
    artifact::*,
    load_plan::plan,
    metadata::{AddressRange, VirtualAddress},
    modules::ModuleId,
};
use fixtures::{executable, linked};

#[test]
fn owned_bytes_and_identity_are_shared_without_label_or_content_hashing() {
    let mut original = vec![3, 1, 4, 1];
    let source = SourceArtifact::new(original.clone(), Some("label".into())).unwrap();
    original[0] = 99;
    let clone = source.clone();
    let range = source.checked_range(0, 4).unwrap();
    assert_eq!(source.len(), 4);
    assert!(!source.is_empty());
    assert_eq!(source.provenance(), Some("label"));
    assert_eq!(source.identity(), clone.identity());
    assert_eq!(source.read(&range).unwrap(), [3, 1, 4, 1]);
    assert_eq!(
        source.read(&range).unwrap().as_ptr(),
        clone.read(&range).unwrap().as_ptr()
    );
    // IDs follow object creation, not identical bytes or identical/different labels.
    for label in [Some("label".into()), Some("different".into()), None] {
        let other = SourceArtifact::new(vec![3, 1, 4, 1], label).unwrap();
        assert_ne!(source.identity(), other.identity());
    }
}

#[test]
fn reads_cover_full_partial_empty_and_exact_end_ranges() {
    let source = SourceArtifact::new(vec![10, 20, 30, 40], None).unwrap();
    for (offset, length, expected) in [
        (0, 4, &[10, 20, 30, 40][..]),
        (1, 2, &[20, 30][..]),
        (3, 1, &[40][..]),
        (0, 0, &[][..]),
        (2, 0, &[][..]),
        (4, 0, &[][..]),
    ] {
        let range = source.checked_range(offset, length).unwrap();
        assert_eq!(range.source_id(), source.identity());
        assert_eq!(source.read(&range).unwrap(), expected);
        assert_eq!(source.read(&range).unwrap(), source.read(&range).unwrap());
    }
    let empty = SourceArtifact::new(vec![], None).unwrap();
    assert!(empty.is_empty());
    assert_eq!(
        empty.read(&empty.checked_range(0, 0).unwrap()).unwrap(),
        &[]
    );
}

#[test]
fn range_errors_preserve_overflow_offset_and_extent_distinctions() {
    let source = SourceArtifact::new(vec![0; 4], None).unwrap();
    for (offset, length, expected) in [
        (
            4,
            1,
            SourceError::LengthOutOfBounds {
                offset: 4,
                length: 1,
                source_len: 4,
            },
        ),
        (
            5,
            0,
            SourceError::OffsetOutOfBounds {
                offset: 5,
                source_len: 4,
            },
        ),
        (
            u64::MAX,
            0,
            SourceError::OffsetOutOfBounds {
                offset: u64::MAX,
                source_len: 4,
            },
        ),
        (
            u64::MAX,
            1,
            SourceError::RangeOverflow {
                offset: u64::MAX,
                length: 1,
            },
        ),
        (
            1,
            u64::MAX,
            SourceError::RangeOverflow {
                offset: 1,
                length: u64::MAX,
            },
        ),
        (
            0,
            u64::MAX,
            SourceError::LengthOutOfBounds {
                offset: 0,
                length: u64::MAX,
                source_len: 4,
            },
        ),
    ] {
        assert_eq!(source.checked_range(offset, length), Err(expected));
    }
    let empty = SourceArtifact::new(vec![], None).unwrap();
    assert_eq!(
        empty.checked_range(0, 1),
        Err(SourceError::LengthOutOfBounds {
            offset: 0,
            length: 1,
            source_len: 0
        })
    );
}

#[test]
fn checked_tokens_cannot_be_rebound_to_identical_or_larger_sources() {
    let source = SourceArtifact::new(vec![5, 6], None).unwrap();
    let range = source.checked_range(0, 2).unwrap();
    let mut detached = range.extent();
    detached.size = u64::MAX;
    assert_eq!(range.extent().size, 2);
    assert_eq!(detached.size, u64::MAX);
    for bytes in [vec![5, 6], vec![5, 6, 7]] {
        let other = SourceArtifact::new(bytes, None).unwrap();
        assert_eq!(
            other.read(&range),
            Err(SourceError::IdentityMismatch {
                expected: other.identity(),
                actual: source.identity(),
            })
        );
    }
}

#[test]
fn inspection_cannot_turn_truncated_bytes_into_an_admitted_source() {
    let full = executable();
    let observed = inspect(full.clone());
    assert_eq!(observed.identity(), full.source.identity());
    assert_eq!(observed.source_size(), full.source.len());
    assert!(admit(&observed).is_ok());
    let mut truncated = full;
    truncated.source =
        SourceArtifact::new(vec![0; 15], Some("same advertised artifact".into())).unwrap();
    let truncated = inspect(truncated);
    assert_eq!(truncated.description(), observed.description());
    assert_ne!(truncated.identity(), observed.identity());
    assert_eq!(truncated.source_size(), 15);
    let error = admit(&truncated).unwrap_err();
    assert_eq!(
        error,
        Rejection::SourceRange {
            region: 0,
            error: SourceError::LengthOutOfBounds {
                offset: 0,
                length: 16,
                source_len: 15
            }
        }
    );
    assert!(std::error::Error::source(&error).is_some());
    let mut inflated = executable();
    inflated.source = SourceArtifact::new(vec![0; 0x1000], None).unwrap();
    inflated.description.regions[0].source.size = 0x5000;
    inflated.description.regions[0].memory_size = 0x5000;
    assert_eq!(
        admit(&inspect(inflated)),
        Err(Rejection::SourceRange {
            region: 0,
            error: SourceError::LengthOutOfBounds {
                offset: 0,
                length: 0x5000,
                source_len: 0x1000
            }
        })
    );
}

#[test]
fn plans_retain_source_lifetime_and_never_copy_zero_fill_from_input() {
    let (result, identity, data_pointer) = {
        let observed = inspect(linked());
        let before = observed.clone();
        let target = admit(&observed).unwrap();
        let result = plan(&target);
        assert_eq!(result, plan(&target));
        assert_eq!(observed, before);
        let copy = result.mappings()[1].copy.as_ref().unwrap();
        assert_eq!(copy.source.source_id(), observed.identity());
        let pointer = observed.source().read(&copy.source).unwrap().as_ptr();
        (result, observed.identity(), pointer)
    }; // All original input/inspection/target handles are now gone.
    assert_eq!(result.source().identity(), identity);
    assert_eq!(result.metadata().identity, identity);
    assert_eq!(result.metadata().source_size, result.source().len());
    for mapping in result.mappings() {
        if let Some(copy) = &mapping.copy {
            assert_eq!(copy.source.source_id(), identity);
            assert_eq!(
                result.source().read(&copy.source).unwrap().len() as u64,
                copy.source.extent().size
            );
        }
    }
    let data = &result.mappings()[1];
    let copy = data.copy.as_ref().unwrap();
    assert_eq!(
        result.source().read(&copy.source).unwrap(),
        &[16, 17, 18, 19, 20, 21, 22, 23]
    );
    assert_eq!(
        result.source().read(&copy.source).unwrap().as_ptr(),
        data_pointer
    );
    assert_eq!(
        data.zero_fill,
        Some(AddressRange {
            start: VirtualAddress(0x2008),
            size: 24
        })
    );
    assert_eq!(copy.destination, VirtualAddress(0x2000));
}

#[test]
fn multiple_modules_can_share_a_source_without_sharing_target_metadata() {
    let first = executable();
    let mut second = first.clone();
    second.description.module.as_mut().unwrap().id = ModuleId(2);
    let first = plan(&admit(&inspect(first)).unwrap());
    let second = plan(&admit(&inspect(second)).unwrap());
    assert_eq!(first.source().identity(), second.source().identity());
    assert_ne!(first.metadata().module.id, second.metadata().module.id);
    let range = first.mappings()[0].copy.as_ref().unwrap().source;
    assert_eq!(
        first.source().read(&range).unwrap(),
        second.source().read(&range).unwrap()
    );
}

#[test]
fn shared_reads_and_concurrent_creation_keep_identity_consistent() {
    let source = SourceArtifact::new(vec![7; 8], None).unwrap();
    let range = source.checked_range(0, 8).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let shared = source.clone();
            std::thread::spawn(move || {
                assert_eq!(shared.read(&range).unwrap(), &[7; 8]);
                assert_eq!(shared.identity(), range.source_id());
                SourceArtifact::new(vec![7; 8], None).unwrap().identity()
            })
        })
        .collect();
    let identities: std::collections::BTreeSet<_> =
        handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(identities.len(), 8);
    assert!(!identities.contains(&source.identity()));
}

#[test]
fn bounded_range_sweep_matches_native_slice_bounds() {
    for source_len in 0..=16usize {
        let bytes: Vec<u8> = (0..source_len as u8).collect();
        let source = SourceArtifact::new(bytes.clone(), None).unwrap();
        for offset in 0..=18usize {
            for length in 0..=18usize {
                let expected = bytes.get(offset..offset + length);
                let range = source.checked_range(offset as u64, length as u64);
                match expected {
                    Some(bytes) => assert_eq!(source.read(&range.unwrap()).unwrap(), bytes),
                    None => assert!(
                        range.is_err(),
                        "accepted {offset}+{length} against {source_len}"
                    ),
                }
            }
        }
    }
}
