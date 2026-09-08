use astero_loader::{
    artifact::*,
    dependencies::Dependency,
    exports::{Export, ExportKind},
    imports::{Import, SymbolName},
    metadata::*,
    modules::*,
    relocations::*,
};
pub fn executable() -> InputArtifact {
    InputArtifact {
        identity: ArtifactId(42),
        source_size: 64,
        source_label: Some("synthetic executable v1".into()),
        description: ArtifactDescription {
            family: ArtifactFamily::Synthetic,
            architecture: Architecture::X86_64,
            role: ArtifactRole::Executable,
            module: Some(ModuleMetadata {
                id: ModuleId(1),
                name: "fixture".into(),
            }),
            entry_point: Some(VirtualAddress(0x1000)),
            regions: vec![region(0x1000, 0, 16, 16, true)],
            dependencies: vec![],
            imports: vec![],
            exports: vec![],
            relocations: vec![],
            requirements: vec![],
        },
    }
}
pub fn region(
    address: u64,
    offset: u64,
    file: u64,
    memory: u64,
    execute: bool,
) -> RegionObservation {
    RegionObservation {
        address: VirtualAddress(address),
        source: SourceRange {
            offset: FileOffset(offset),
            size: file,
        },
        memory_size: memory,
        alignment: 16,
        permissions: Permissions {
            read: true,
            write: !execute,
            execute,
        },
    }
}
pub fn linked() -> InputArtifact {
    let mut input = executable();
    let d = &mut input.description;
    // Deliberately out of address order, with an initialized data prefix and BSS-like tail.
    d.regions.insert(0, region(0x2000, 16, 8, 32, false));
    d.dependencies = vec![
        Dependency {
            module: ModuleId(2),
        },
        Dependency {
            module: ModuleId(3),
        },
    ];
    d.imports = vec![Import {
        dependency: ModuleId(3),
        symbol: SymbolName("external".into()),
    }];
    d.exports = vec![
        Export {
            symbol: SymbolName("entry".into()),
            range: AddressRange {
                start: VirtualAddress(0x1000),
                size: 1,
            },
            kind: ExportKind::Function,
        },
        Export {
            symbol: SymbolName("data".into()),
            range: AddressRange {
                start: VirtualAddress(0x2000),
                size: 8,
            },
            kind: ExportKind::Data,
        },
    ];
    d.relocations = vec![
        Relocation {
            site: VirtualAddress(0x2000),
            kind: RelocationKind::SyntheticAbsolute64,
            target: RelocationTarget::ImportIndex(0),
            addend: -7,
        },
        Relocation {
            site: VirtualAddress(0x2010),
            kind: RelocationKind::SyntheticAbsolute64,
            target: RelocationTarget::LocalAddress(VirtualAddress(0x1000)),
            addend: i64::MAX,
        },
    ];
    input
}
