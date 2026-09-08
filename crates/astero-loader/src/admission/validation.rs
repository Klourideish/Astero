use super::{
    DescriptorKind, MetadataField, MetadataProblem, RangeDomain, Rejection, TargetMetadata,
    ValidatedRegion, ValidatedTarget,
};
use crate::{
    artifact::{Architecture, ArtifactFamily, InspectedArtifact},
    exports::ExportKind,
    metadata::{AddressRange, RegionObservation},
    modules::{ArtifactRole, TargetRole},
    relocations::{RelocationKind, RelocationTarget},
};
use std::collections::BTreeSet;

/// Conservative M2 admission policy. Success means describable synthetic work, not executable code.
/// Errors identify original input indices; the first failure in stage/input order is returned.
pub fn admit(inspected: &InspectedArtifact) -> Result<ValidatedTarget, Rejection> {
    let d = inspected.description();
    if d.family != ArtifactFamily::Synthetic {
        return Err(Rejection::UnsupportedFormat(d.family));
    }
    if d.architecture != Architecture::X86_64 {
        return Err(Rejection::UnsupportedArchitecture(d.architecture));
    }
    let role = match d.role {
        ArtifactRole::Executable => TargetRole::Executable,
        ArtifactRole::Module => TargetRole::Module,
        other => return Err(Rejection::UnsupportedRole(other)),
    };
    if let Some(requirement) = d.requirements.first() {
        return Err(Rejection::UnsupportedRequirement(*requirement));
    }
    let module = d
        .module
        .as_ref()
        .ok_or(Rejection::MissingMetadata(MetadataField::Module))?;
    if module.name.trim().is_empty() {
        return Err(Rejection::MissingMetadata(MetadataField::ModuleName));
    }
    if d.regions.is_empty() {
        return Err(Rejection::MissingMetadata(MetadataField::Regions));
    }
    let mut regions = Vec::with_capacity(d.regions.len());
    for (i, r) in d.regions.iter().enumerate() {
        if r.memory_size == 0 {
            return Err(Rejection::EmptyRegion { region: i });
        }
        if r.source.size > r.memory_size {
            return Err(Rejection::InvalidSize {
                region: i,
                file_size: r.source.size,
                memory_size: r.memory_size,
            });
        }
        r.address
            .0
            .checked_add(r.memory_size)
            .ok_or(Rejection::RangeOverflow {
                region: i,
                domain: RangeDomain::Virtual,
            })?;
        let source_end =
            r.source
                .offset
                .0
                .checked_add(r.source.size)
                .ok_or(Rejection::RangeOverflow {
                    region: i,
                    domain: RangeDomain::Source,
                })?;
        if source_end > inspected.source_size() {
            return Err(Rejection::SourceOutOfBounds {
                region: i,
                end: source_end,
                source_size: inspected.source_size(),
            });
        }
        if !r.alignment.is_power_of_two() || !r.address.0.is_multiple_of(r.alignment) {
            return Err(Rejection::InvalidAlignment {
                region: i,
                alignment: r.alignment,
                address: r.address.0,
            });
        }
        for (j, earlier) in d.regions[..i].iter().enumerate() {
            if r.address.0 < earlier.address.0 + earlier.memory_size
                && earlier.address.0 < r.address.0 + r.memory_size
            {
                return Err(Rejection::OverlappingRegions {
                    first: j,
                    second: i,
                });
            }
        }
        regions.push(ValidatedRegion {
            range: AddressRange {
                start: r.address,
                size: r.memory_size,
            },
            source: r.source,
            alignment: r.alignment,
            permissions: r.permissions,
        });
    }
    match d.entry_point {
        None if role == TargetRole::Executable => {
            return Err(Rejection::InvalidEntryPoint { address: None });
        }
        Some(address)
            if !d.regions.iter().any(|r| {
                r.permissions.execute && contains(r.address.0, r.source.size, address.0, 1)
            }) =>
        {
            return Err(Rejection::InvalidEntryPoint {
                address: Some(address.0),
            });
        }
        _ => {}
    }
    let mut dependencies = BTreeSet::new();
    for (i, dependency) in d.dependencies.iter().enumerate() {
        if dependency.module == module.id {
            return Err(metadata(
                DescriptorKind::Dependency,
                i,
                MetadataProblem::SelfDependency,
            ));
        }
        if !dependencies.insert(dependency.module) {
            return Err(metadata(
                DescriptorKind::Dependency,
                i,
                MetadataProblem::DuplicateDependency,
            ));
        }
    }
    let mut imports = BTreeSet::new();
    for (i, import) in d.imports.iter().enumerate() {
        if import.symbol.0.trim().is_empty() {
            return Err(metadata(
                DescriptorKind::Import,
                i,
                MetadataProblem::EmptySymbol,
            ));
        }
        if !dependencies.contains(&import.dependency) {
            return Err(metadata(
                DescriptorKind::Import,
                i,
                MetadataProblem::UnknownDependency,
            ));
        }
        if !imports.insert((import.dependency, &import.symbol)) {
            return Err(metadata(
                DescriptorKind::Import,
                i,
                MetadataProblem::DuplicateImport,
            ));
        }
    }
    let mut exports = BTreeSet::new();
    for (i, export) in d.exports.iter().enumerate() {
        if export.symbol.0.trim().is_empty() {
            return Err(metadata(
                DescriptorKind::Export,
                i,
                MetadataProblem::EmptySymbol,
            ));
        }
        if !exports.insert(&export.symbol) {
            return Err(metadata(
                DescriptorKind::Export,
                i,
                MetadataProblem::DuplicateExport,
            ));
        }
        let owner =
            region_for(&d.regions, export.range.start.0, export.range.size).ok_or(metadata(
                DescriptorKind::Export,
                i,
                MetadataProblem::InvalidExportRange,
            ))?;
        if export.kind == ExportKind::Function && !owner.permissions.execute {
            return Err(metadata(
                DescriptorKind::Export,
                i,
                MetadataProblem::NonExecutableExport,
            ));
        }
    }
    for (i, relocation) in d.relocations.iter().enumerate() {
        if relocation.kind != RelocationKind::SyntheticAbsolute64 {
            return Err(Rejection::UnsupportedRelocation { index: i });
        }
        if region_for(&d.regions, relocation.site.0, 8).is_none() {
            return Err(metadata(
                DescriptorKind::Relocation,
                i,
                MetadataProblem::InvalidRelocationRange,
            ));
        }
        if d.relocations[..i]
            .iter()
            .any(|r| r.site.0 < relocation.site.0 + 8 && relocation.site.0 < r.site.0 + 8)
        {
            return Err(metadata(
                DescriptorKind::Relocation,
                i,
                MetadataProblem::ConflictingRelocations,
            ));
        }
        match relocation.target {
            RelocationTarget::ImportIndex(index) if index >= d.imports.len() => {
                return Err(metadata(
                    DescriptorKind::Relocation,
                    i,
                    MetadataProblem::UnknownImport,
                ));
            }
            RelocationTarget::LocalAddress(address)
                if region_for(&d.regions, address.0, 1).is_none() =>
            {
                return Err(metadata(
                    DescriptorKind::Relocation,
                    i,
                    MetadataProblem::InvalidLocalTarget,
                ));
            }
            _ => {}
        }
    }
    regions.sort_by_key(|r| r.range.start);
    Ok(ValidatedTarget {
        metadata: TargetMetadata {
            identity: inspected.identity(),
            family: d.family,
            source_size: inspected.source_size(),
            source_label: inspected.source_label().map(str::to_owned),
            architecture: d.architecture,
            module: module.clone(),
            role,
            entry_point: d.entry_point,
            dependencies: d.dependencies.clone(),
            imports: d.imports.clone(),
            exports: d.exports.clone(),
            relocations: d.relocations.clone(),
        },
        regions,
    })
}

fn metadata(kind: DescriptorKind, index: usize, problem: MetadataProblem) -> Rejection {
    Rejection::MalformedMetadata {
        kind,
        index,
        problem,
    }
}
fn contains(start: u64, size: u64, address: u64, length: u64) -> bool {
    length != 0
        && address >= start
        && address
            .checked_add(length)
            .zip(start.checked_add(size))
            .is_some_and(|(end, limit)| end <= limit)
}
fn region_for(
    regions: &[RegionObservation],
    address: u64,
    size: u64,
) -> Option<&RegionObservation> {
    regions
        .iter()
        .find(|r| contains(r.address.0, r.memory_size, address, size))
}
