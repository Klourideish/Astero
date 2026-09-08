mod fixtures;
use astero_loader::{
    admission::*, artifact::*, dependencies::Dependency, exports::ExportKind, imports::SymbolName,
    load_plan::plan, metadata::*, modules::*, relocations::*,
};
use fixtures::{executable, linked, region};
fn reject(input: InputArtifact) -> Rejection {
    admit(&inspect(input)).unwrap_err()
}
fn malformed(kind: DescriptorKind, index: usize, problem: MetadataProblem) -> Rejection {
    Rejection::MalformedMetadata {
        kind,
        index,
        problem,
    }
}

#[test]
fn inspection_preserves_observations_without_admitting_them() {
    let mut raw = linked();
    raw.description.architecture = Architecture::Unknown;
    raw.description.regions[0].alignment = 3;
    let observed = inspect(raw.clone());
    assert_eq!(observed.description(), &raw.description);
    assert_eq!(observed.identity(), raw.identity);
    assert_eq!(observed.source_size(), raw.source_size);
    assert_eq!(observed.source_label(), raw.source_label.as_deref());
    assert_eq!(
        admit(&observed),
        Err(Rejection::UnsupportedArchitecture(Architecture::Unknown))
    );
}
#[test]
fn minimal_executable_and_module_have_distinct_entry_contracts() {
    let executable = executable();
    let target = admit(&inspect(executable.clone())).unwrap();
    assert_eq!(target.metadata().role, TargetRole::Executable);
    assert_eq!(
        plan(&target).metadata().entry_point,
        Some(VirtualAddress(0x1000))
    );
    let mut module = executable;
    module.description.role = ArtifactRole::Module;
    module.description.entry_point = None;
    let target = admit(&inspect(module)).unwrap();
    assert_eq!(target.metadata().role, TargetRole::Module);
    assert_eq!(plan(&target).metadata().entry_point, None);
}
#[test]
fn multi_region_plan_preserves_linking_and_mapping_intent() {
    let input = linked();
    let observed = inspect(input.clone());
    let target = admit(&observed).unwrap();
    let result = plan(&target);
    assert_eq!(result.metadata().identity, input.identity);
    assert_eq!(result.metadata().family, ArtifactFamily::Synthetic);
    assert_eq!(result.metadata().module, input.description.module.unwrap());
    assert_eq!(
        result.metadata().dependencies,
        input.description.dependencies
    );
    assert_eq!(result.metadata().imports, input.description.imports);
    assert_eq!(result.metadata().exports, input.description.exports);
    assert_eq!(result.metadata().relocations, input.description.relocations);
    assert_eq!(result.metadata().source_size, 64);
    assert_eq!(result.mappings().len(), 2);
    let text = &result.mappings()[0];
    let data = &result.mappings()[1];
    assert_eq!(
        text.range,
        AddressRange {
            start: VirtualAddress(0x1000),
            size: 16
        }
    );
    assert_eq!(
        text.permissions,
        Permissions {
            read: true,
            write: false,
            execute: true
        }
    );
    assert_eq!(text.alignment, 16);
    assert_eq!(text.zero_fill, None);
    assert_eq!(
        data.permissions,
        Permissions {
            read: true,
            write: true,
            execute: false
        }
    );
    assert_eq!(data.alignment, 16);
    let copy = data.copy.as_ref().unwrap();
    assert_eq!(
        copy.source,
        SourceRange {
            offset: FileOffset(16),
            size: 8
        }
    );
    assert_eq!(copy.destination, VirtualAddress(0x2000));
    assert_eq!(
        data.zero_fill,
        Some(AddressRange {
            start: VirtualAddress(0x2008),
            size: 24
        })
    );
    assert_eq!(plan(&target), result);
    assert_eq!(admit(&observed).unwrap(), target);
    // Reordering disjoint observed regions does not change their canonical mapping plan.
    let mut reordered = observed.description().clone();
    reordered.regions.reverse();
    let mut raw = linked();
    raw.description = reordered;
    assert_eq!(plan(&admit(&inspect(raw)).unwrap()), result);
}
#[test]
fn pure_bss_and_adjacent_regions_are_explicit() {
    let mut raw = executable();
    raw.description
        .regions
        .push(region(0x1010, 64, 0, 16, false));
    let result = plan(&admit(&inspect(raw)).unwrap());
    let bss = &result.mappings()[1];
    assert_eq!(bss.copy, None);
    assert_eq!(bss.zero_fill, Some(bss.range));
}
#[test]
fn detached_values_cannot_modify_admitted_target_or_reexecute_work() {
    let observed = inspect(linked());
    let before = observed.clone();
    let target = admit(&observed).unwrap();
    let before_target = target.clone();
    let first = plan(&target);
    let mut detached = first.metadata().clone();
    detached.relocations.clear();
    let mut mapping = first.mappings()[0].clone();
    mapping.alignment = 0;
    assert_eq!(observed, before);
    assert_eq!(target, before_target);
    assert_eq!(plan(&target), first);
    assert_eq!(first.metadata().relocations[0].addend, -7);
    assert_eq!(first.metadata().relocations[1].addend, i64::MAX);
    assert_eq!(detached.relocations.len(), 0);
    assert_eq!(mapping.alignment, 0);
}
#[test]
fn unsupported_classifications_and_requirements_are_explicit() {
    for family in [
        ArtifactFamily::Elf,
        ArtifactFamily::SelfFormat,
        ArtifactFamily::Unknown,
    ] {
        let mut raw = executable();
        raw.description.family = family;
        assert_eq!(reject(raw), Rejection::UnsupportedFormat(family));
    }
    for arch in [Architecture::Aarch64, Architecture::Unknown] {
        let mut raw = executable();
        raw.description.architecture = arch;
        assert_eq!(reject(raw), Rejection::UnsupportedArchitecture(arch));
    }
    let mut raw = executable();
    raw.description.role = ArtifactRole::Unknown;
    assert_eq!(
        reject(raw),
        Rejection::UnsupportedRole(ArtifactRole::Unknown)
    );
    for requirement in [
        DeferredRequirement::ThreadLocalStorage,
        DeferredRequirement::InitializationCallbacks,
        DeferredRequirement::DynamicPlacement,
    ] {
        let mut raw = executable();
        raw.description.requirements.push(requirement);
        assert_eq!(reject(raw), Rejection::UnsupportedRequirement(requirement));
    }
}
#[test]
fn required_metadata_cannot_be_omitted() {
    let mut raw = executable();
    raw.description.module = None;
    assert_eq!(
        reject(raw),
        Rejection::MissingMetadata(MetadataField::Module)
    );
    let mut raw = executable();
    raw.description.module.as_mut().unwrap().name = " ".into();
    assert_eq!(
        reject(raw),
        Rejection::MissingMetadata(MetadataField::ModuleName)
    );
    let mut raw = executable();
    raw.description.regions.clear();
    assert_eq!(
        reject(raw),
        Rejection::MissingMetadata(MetadataField::Regions)
    );
}
#[test]
fn alignment_checks_both_shape_and_address() {
    for alignment in [0, 3, 8192] {
        let mut raw = executable();
        raw.description.regions[0].alignment = alignment;
        assert_eq!(
            reject(raw),
            Rejection::InvalidAlignment {
                region: 0,
                alignment,
                address: 0x1000
            }
        );
    }
}
#[test]
fn sizes_and_source_boundaries_are_checked() {
    let mut raw = executable();
    raw.description.regions[0].memory_size = 0;
    assert_eq!(reject(raw), Rejection::EmptyRegion { region: 0 });
    let mut raw = executable();
    raw.description.regions[0].memory_size = 8;
    assert_eq!(
        reject(raw),
        Rejection::InvalidSize {
            region: 0,
            file_size: 16,
            memory_size: 8
        }
    );
    let mut raw = executable();
    raw.source_size = 15;
    assert_eq!(
        reject(raw),
        Rejection::SourceOutOfBounds {
            region: 0,
            end: 16,
            source_size: 15
        }
    );
    let mut raw = executable();
    raw.description.regions[0].source = SourceRange {
        offset: FileOffset(65),
        size: 0,
    };
    assert_eq!(
        reject(raw),
        Rejection::SourceOutOfBounds {
            region: 0,
            end: 65,
            source_size: 64
        }
    );
}
#[test]
fn arithmetic_overflow_is_rejected_before_comparisons() {
    let mut raw = executable();
    raw.description.regions[0].address = VirtualAddress(u64::MAX - 15);
    assert_eq!(
        reject(raw),
        Rejection::RangeOverflow {
            region: 0,
            domain: RangeDomain::Virtual
        }
    );
    let mut raw = executable();
    raw.description.regions[0].source.offset = FileOffset(u64::MAX);
    assert_eq!(
        reject(raw),
        Rejection::RangeOverflow {
            region: 0,
            domain: RangeDomain::Source
        }
    );
}
#[test]
fn all_virtual_overlaps_are_rejected_even_when_permissions_match() {
    for address in [0x1000, 0x1008] {
        let mut raw = executable();
        let mut second = region(address, 16, 8, 16, true);
        second.alignment = 1;
        raw.description.regions.push(second);
        assert_eq!(
            reject(raw),
            Rejection::OverlappingRegions {
                first: 0,
                second: 1
            }
        );
    }
}
#[test]
fn entry_must_be_present_for_executable_and_source_backed_executable() {
    for address in [None, Some(0x1010), Some(0x2000), Some(u64::MAX)] {
        let mut raw = executable();
        raw.description.entry_point = address.map(VirtualAddress);
        assert_eq!(reject(raw), Rejection::InvalidEntryPoint { address });
    }
    let mut raw = executable();
    raw.description.regions[0].permissions.execute = false;
    assert_eq!(
        reject(raw),
        Rejection::InvalidEntryPoint {
            address: Some(0x1000)
        }
    );
    let mut raw = executable();
    raw.description.regions[0].memory_size = 32;
    raw.description.entry_point = Some(VirtualAddress(0x1010));
    assert_eq!(
        reject(raw),
        Rejection::InvalidEntryPoint {
            address: Some(0x1010)
        }
    );
    let mut raw = executable();
    raw.description.role = ArtifactRole::Module;
    raw.description.entry_point = Some(VirtualAddress(0));
    assert_eq!(
        reject(raw),
        Rejection::InvalidEntryPoint { address: Some(0) }
    );
}
#[test]
fn dependency_and_import_metadata_is_consistent() {
    let mut raw = linked();
    raw.description.dependencies.push(Dependency {
        module: ModuleId(1),
    });
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Dependency,
            2,
            MetadataProblem::SelfDependency
        )
    );
    let mut raw = linked();
    raw.description
        .dependencies
        .push(raw.description.dependencies[0].clone());
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Dependency,
            2,
            MetadataProblem::DuplicateDependency
        )
    );
    let mut raw = linked();
    raw.description.imports[0].symbol = SymbolName(" ".into());
    assert_eq!(
        reject(raw),
        malformed(DescriptorKind::Import, 0, MetadataProblem::EmptySymbol)
    );
    let mut raw = linked();
    raw.description.imports[0].dependency = ModuleId(99);
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Import,
            0,
            MetadataProblem::UnknownDependency
        )
    );
    let mut raw = linked();
    raw.description
        .imports
        .push(raw.description.imports[0].clone());
    assert_eq!(
        reject(raw),
        malformed(DescriptorKind::Import, 1, MetadataProblem::DuplicateImport)
    );
}
#[test]
fn export_metadata_names_extents_and_kinds_are_checked() {
    let mut raw = linked();
    raw.description.exports[0].symbol = SymbolName("".into());
    assert_eq!(
        reject(raw),
        malformed(DescriptorKind::Export, 0, MetadataProblem::EmptySymbol)
    );
    let mut raw = linked();
    raw.description
        .exports
        .push(raw.description.exports[0].clone());
    assert_eq!(
        reject(raw),
        malformed(DescriptorKind::Export, 2, MetadataProblem::DuplicateExport)
    );
    for (start, size) in [(0x1000, 0), (0x100f, 2), (u64::MAX, 2)] {
        let mut raw = linked();
        raw.description.exports[0].range = AddressRange {
            start: VirtualAddress(start),
            size,
        };
        assert_eq!(
            reject(raw),
            malformed(
                DescriptorKind::Export,
                0,
                MetadataProblem::InvalidExportRange
            )
        );
    }
    let mut raw = linked();
    raw.description.exports[1].kind = ExportKind::Function;
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Export,
            1,
            MetadataProblem::NonExecutableExport
        )
    );
}
#[test]
fn relocation_work_is_validated_without_evaluation() {
    let mut raw = linked();
    raw.description.relocations[0].kind = RelocationKind::Unsupported;
    assert_eq!(reject(raw), Rejection::UnsupportedRelocation { index: 0 });
    for site in [0x2019, u64::MAX] {
        let mut raw = linked();
        raw.description.relocations[0].site = VirtualAddress(site);
        assert_eq!(
            reject(raw),
            malformed(
                DescriptorKind::Relocation,
                0,
                MetadataProblem::InvalidRelocationRange
            )
        );
    }
    let mut raw = linked();
    raw.description.relocations[0].target = RelocationTarget::ImportIndex(1);
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Relocation,
            0,
            MetadataProblem::UnknownImport
        )
    );
    let mut raw = linked();
    raw.description.relocations[0].target = RelocationTarget::LocalAddress(VirtualAddress(0x3000));
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Relocation,
            0,
            MetadataProblem::InvalidLocalTarget
        )
    );
    let mut raw = linked();
    raw.description.relocations[1].site = VirtualAddress(0x2004);
    assert_eq!(
        reject(raw),
        malformed(
            DescriptorKind::Relocation,
            1,
            MetadataProblem::ConflictingRelocations
        )
    );
}
#[test]
fn boundary_sweep_preserves_copy_zero_fill_partition() {
    for file_size in 0..=32 {
        for memory_size in 1..=32 {
            let mut raw = executable();
            raw.description.role = ArtifactRole::Module;
            raw.description.entry_point = None;
            raw.description.regions = vec![region(0x1000, 0, file_size, memory_size, false)];
            let result = admit(&inspect(raw));
            if file_size > memory_size {
                assert!(matches!(result, Err(Rejection::InvalidSize { .. })));
                continue;
            }
            let result = plan(&result.unwrap());
            let mapping = &result.mappings()[0];
            assert_eq!(
                mapping.copy.as_ref().map_or(0, |c| c.source.size),
                file_size
            );
            assert_eq!(
                mapping.zero_fill.map_or(0, |z| z.size),
                memory_size - file_size
            );
            if let Some(zero) = mapping.zero_fill {
                assert_eq!(zero.start.0, 0x1000 + file_size);
            }
            assert_eq!(mapping.range.size, memory_size);
        }
    }
}
