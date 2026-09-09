//! Conservative ordinary ELF linkage policy, not PS5 semantics.
mod derive;
pub(in crate::elf::dynamic::candidates) use derive::classify;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    LocalBinding,
    NonExternalVisibility,
    UnknownAttributes,
    UnsupportedType,
    SpecialSection,
    MissingName,
    EmptyName,
    ConflictingAttributes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
    ImportCandidate,
    ExportCandidate,
    InternalDefined(Reason),
    UndefinedUnclassified(Reason),
    Unclassified(Reason),
    Absolute,
    Common,
    Null,
}
