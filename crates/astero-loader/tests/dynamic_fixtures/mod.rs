//! Generated ELF bytes only. Numeric offsets are independently specified fixture layout.
pub use astero_loader::elf::dynamic::synthetic::*;
use astero_loader::{
    artifact::SourceArtifact,
    elf::{ElfInspection, inspect},
    modules::{ModuleId, ModuleMetadata},
};
pub fn parsed(b: Vec<u8>) -> ElfInspection {
    inspect(
        SourceArtifact::new(b, Some("generated M5".into())).unwrap(),
        Some(ModuleMetadata {
            id: ModuleId(5),
            name: "fixture".into(),
        }),
    )
    .unwrap()
}
