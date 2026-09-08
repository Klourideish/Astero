use super::{DynamicSymbolObservation, SymbolError, SymbolTable};
use crate::elf::{
    ElfInspection,
    dynamic::{
        ObservationLimits,
        hash::{self, HashLimits, extent::TrustedSymbolExtent},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumerationLimits {
    pub max_symbols: u64,
    pub max_name_scan_bytes: u64,
    pub max_total_name_scan_bytes: u64,
}
/// Lazy, fail-stop observation. Successful preceding entries do not imply whole-table semantic validity.
pub struct SymbolEnumeration<'t, 'e> {
    table: &'t SymbolTable<'e>,
    count: u64,
    next: u64,
    per_name: u64,
    remaining: u64,
}
impl<'e> SymbolTable<'e> {
    /// Derives evidence internally from this exact source; detached evidence cannot be injected.
    pub fn with_hash(
        elf: &'e ElfInspection,
        dynamic_limits: ObservationLimits,
        hash_limits: HashLimits,
    ) -> Result<Self, SymbolError> {
        let mut table = Self::new(elf, dynamic_limits)?;
        let observation =
            hash::observe(elf, dynamic_limits, hash_limits).map_err(SymbolError::Hash)?;
        table.extent = observation.extent().cloned();
        Ok(table)
    }
    pub fn extent(&self) -> Option<&TrustedSymbolExtent> {
        self.extent.as_ref()
    }
    pub fn enumerate(
        &self,
        limits: EnumerationLimits,
    ) -> Result<SymbolEnumeration<'_, 'e>, SymbolError> {
        let count = self.symbol_count()?;
        if count > limits.max_symbols {
            return Err(SymbolError::EnumerationLimit {
                count,
                limit: limits.max_symbols,
            });
        }
        Ok(SymbolEnumeration {
            table: self,
            count,
            next: 0,
            per_name: limits.max_name_scan_bytes,
            remaining: limits.max_total_name_scan_bytes,
        })
    }
}
impl<'t> Iterator for SymbolEnumeration<'t, '_> {
    type Item = Result<DynamicSymbolObservation<'t>, SymbolError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.count {
            return None;
        }
        let result = self
            .table
            .read_candidate(self.next, self.per_name.min(self.remaining));
        self.next += 1;
        match &result {
            Ok(symbol) => {
                if let Some(name) = symbol.name {
                    self.remaining -= name.scanned_bytes();
                }
            }
            Err(_) => self.next = self.count,
        }
        Some(result)
    }
}
impl std::iter::FusedIterator for SymbolEnumeration<'_, '_> {}
