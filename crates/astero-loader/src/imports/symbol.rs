use crate::modules::ModuleId;
/// Synthetic symbol name, deliberately not a NID or parser-specific symbol index.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolName(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub dependency: ModuleId,
    pub symbol: SymbolName,
}
