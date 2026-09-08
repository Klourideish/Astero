use super::super::{
    ElfError,
    header::{self, Elf64Header},
    program_headers::ProgramHeaders,
};
use crate::{
    artifact::{
        self, Architecture, ArtifactDescription, ArtifactFamily, DeferredRequirement,
        InputArtifact, InspectedArtifact, SourceArtifact,
    },
    metadata::{FileOffset, Permissions, RegionObservation, SourceRange, VirtualAddress},
    modules::{ArtifactRole, ModuleMetadata},
};
/// Parser-specific details stay in this read-only report, never in validated target/plan types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElfInspection {
    header: Elf64Header,
    artifact: InspectedArtifact,
}
impl ElfInspection {
    pub fn header(&self) -> &Elf64Header {
        &self.header
    }
    pub fn artifact(&self) -> &InspectedArtifact {
        &self.artifact
    }
    pub fn program_headers(&self) -> ProgramHeaders<'_> {
        ProgramHeaders::new(self.artifact.source(), &self.header)
    }
}
/// Inspect bytes, not execution support. Module metadata is explicit caller context, not ELF-derived.
/// Missing/invalid module context is preserved for the normal admission check.
pub fn inspect(
    source: SourceArtifact,
    module: Option<ModuleMetadata>,
) -> Result<ElfInspection, ElfError> {
    let header = header::decode(&source)?;
    let mut regions = Vec::new();
    let mut requirements = Vec::new();
    if header.identification.os_abi != 0
        || header.identification.abi_version != 0
        || header.flags != 0
    {
        requirements.push(DeferredRequirement::PlatformSemantics);
    }
    for p in ProgramHeaders::new(&source, &header) {
        let p = p?;
        if p.kind == 1 {
            if p.flags & !7 != 0 && !requirements.contains(&DeferredRequirement::PlatformSemantics)
            {
                requirements.push(DeferredRequirement::PlatformSemantics);
            }
            regions.push(RegionObservation {
                address: VirtualAddress(p.virtual_address),
                source: SourceRange {
                    offset: FileOffset(p.file_offset),
                    size: p.file_size,
                },
                memory_size: p.memory_size,
                // ELF 0/1 both mean no alignment; Astero's normalized minimum is 1.
                alignment: p.alignment.max(1),
                permissions: Permissions {
                    read: p.flags & 4 != 0,
                    write: p.flags & 2 != 0,
                    execute: p.flags & 1 != 0,
                },
            });
        } else if p.kind != 0
            && !requirements.contains(&DeferredRequirement::UninspectedProgramSemantics)
        {
            // PT_NULL is unused. Every other non-load kind needs later interpretation.
            requirements.push(DeferredRequirement::UninspectedProgramSemantics);
        }
    }
    let description = ArtifactDescription {
        family: ArtifactFamily::Elf,
        architecture: match header.machine {
            62 => Architecture::X86_64,
            183 => Architecture::Aarch64,
            _ => Architecture::Unknown,
        },
        role: match header.object_type {
            2 => ArtifactRole::Executable,
            3 => ArtifactRole::Module,
            _ => ArtifactRole::Unknown,
        },
        module,
        entry_point: (header.entry != 0).then_some(VirtualAddress(header.entry)),
        regions,
        dependencies: Vec::new(),
        imports: Vec::new(),
        exports: Vec::new(),
        relocations: Vec::new(),
        requirements,
    };
    Ok(ElfInspection {
        header,
        artifact: artifact::inspect(InputArtifact {
            source,
            description,
        }),
    })
}
