//! Data declarations are not handlers. Residency is retained by the composing runtime.
use crate::dispatch::prepared::ProviderKey;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataExport {
    pub key: ProviderKey,
    pub address: u64,
    pub size: u64,
    pub read_only: bool,
}
impl DataExport {
    pub fn address_with_addend(&self, addend: i64) -> Option<u64> {
        let offset = u64::try_from(addend).ok()?;
        if offset >= self.size {
            return None;
        }
        self.address.checked_add(offset)
    }
}
