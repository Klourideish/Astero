use super::DependencyName;
use crate::modules::ModuleId;
/// Required dependency declaration, never evidence of availability or resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dependency {
    /// Explicit caller graph identity; subject to the original unique/non-self module rules.
    Module(ModuleId),
    /// Unresolved byte name. Ordering and repeated declarations are significant observations.
    Named(DependencyName),
}
