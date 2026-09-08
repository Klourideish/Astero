use super::{HashLimits, HashObservation, HashReader};
use crate::elf::{
    ElfInspection,
    address_translation::AddressTranslator,
    dynamic::{
        self, DynamicObservation, ObservationLimits,
        hash::{
            error::{HashError, HashKind},
            extent, gnu, sysv,
        },
        tags::DynamicTag,
    },
};
pub fn observe(
    elf: &ElfInspection,
    dynamic_limits: ObservationLimits,
    limits: HashLimits,
) -> Result<HashObservation, HashError> {
    let dynamic = dynamic::observe(elf, dynamic_limits).map_err(HashError::Dynamic)?;
    let source = elf.artifact().source();
    let mut result = HashObservation {
        source: source.clone(),
        sysv: None,
        gnu: None,
        extent: None,
    };
    let DynamicObservation::Present(table) = dynamic else {
        return Ok(result);
    };
    let translator = AddressTranslator::new(elf).map_err(HashError::Mapping)?;
    let mut remaining = limits.max_words;
    // Stable SysV then GNU order, independent of dynamic tag ordering.
    for (tag, kind) in [
        (DynamicTag::Hash, HashKind::SysV),
        (DynamicTag::GnuHash, HashKind::Gnu),
    ] {
        if let Some(entry) = table.entries().iter().find(|e| e.tag == tag) {
            let mut reader = HashReader {
                source,
                translator: &translator,
                base: entry.value,
                kind,
                remaining: &mut remaining,
            };
            match kind {
                HashKind::SysV => result.sysv = Some(sysv::inspect(&mut reader)?),
                HashKind::Gnu => result.gnu = Some(gnu::inspect(&mut reader)?),
            }
        }
    }
    result.extent = extent::derive(
        result.sysv.as_ref(),
        result.gnu.as_ref(),
        table.descriptors().symbols.as_ref(),
        &translator,
    )?;
    Ok(result)
}
