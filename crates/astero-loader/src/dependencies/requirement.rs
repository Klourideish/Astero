use crate::modules::ModuleId;
/// Required external module identity. This does not assert availability or load order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub module: ModuleId,
}
