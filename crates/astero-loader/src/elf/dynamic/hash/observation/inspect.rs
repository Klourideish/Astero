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
    let DynamicObservation::Present(table) = dynamic else {
        return Ok(HashObservation {
            source: source.clone(),
            sysv: None,
            gnu: None,
            extent: None,
        });
    };
    let translator = AddressTranslator::new(elf).map_err(HashError::Mapping)?;
    collect(
        source,
        table.entries(),
        table.descriptors().symbols.as_ref(),
        &translator,
        limits,
    )
}
/// Shared M8 decoders and proof rules; no symbol entry or name consumer is called.
pub(in crate::elf::dynamic::hash) fn collect(
    source: &crate::artifact::SourceArtifact,
    entries: &[crate::elf::dynamic::entries::DynamicEntry],
    symbols: Option<&crate::elf::dynamic::observation::SymbolTableDescriptor>,
    translator: &AddressTranslator<'_>,
    limits: HashLimits,
) -> Result<HashObservation, HashError> {
    let mut result = HashObservation {
        source: source.clone(),
        sysv: None,
        gnu: None,
        extent: None,
    };
    let mut remaining = limits.max_words;
    // Stable SysV then GNU order, independent of dynamic tag ordering.
    for (tag, kind) in [
        (DynamicTag::Hash, HashKind::SysV),
        (DynamicTag::GnuHash, HashKind::Gnu),
    ] {
        if let Some(entry) = entries.iter().find(|e| e.tag == tag) {
            let mut reader = HashReader {
                source,
                translator,
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
        symbols,
        translator,
    )?;
    Ok(result)
}
