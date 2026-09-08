use crate::{imports::SymbolName, metadata::AddressRange};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportKind {
    Function,
    Data,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub symbol: SymbolName,
    pub range: AddressRange,
    pub kind: ExportKind,
}
