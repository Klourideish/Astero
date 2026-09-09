use super::{HashMetadataFailure, HashMetadataLimits, HashMetadataOutcome, HashMetadataReport};
use crate::{
    artifact::SourceArtifact,
    elf::{
        address_translation::AddressTranslator,
        dynamic::{
            bounded::collect::discover,
            hash::{error::HashError, observation},
            observation::descriptors,
            tags::DynamicTag,
        },
    },
};
/// Independent request: raw discovery, selected descriptor checks, existing hash proof, STOP.
pub fn observe(source: SourceArtifact, limits: HashMetadataLimits) -> HashMetadataReport {
    let (raw, outcome) = match discover(&source, limits.dynamic) {
        Err(e) => (
            None,
            HashMetadataOutcome::Failed(HashMetadataFailure::Discovery(e)),
        ),
        Ok(None) => (None, HashMetadataOutcome::Unavailable),
        Ok(Some((headers, raw))) => {
            let result = (|| {
                if !raw
                    .entries()
                    .iter()
                    .any(|e| matches!(e.tag, DynamicTag::Hash | DynamicTag::GnuHash))
                {
                    return Ok(None);
                }
                let translator = AddressTranslator::from_program_headers(
                    &source,
                    headers.program_headers().iter().cloned().map(Ok),
                )
                .map_err(HashError::Mapping)?;
                // Reuse singleton/pairing rules, excluding strings, relocations and unrelated tags.
                // At most four selected tag keys; dynamic budget already bounds all examined entries.
                let selected = descriptors::collect(
                    raw.entries().iter().filter(|e| {
                        matches!(
                            e.tag,
                            DynamicTag::Hash
                                | DynamicTag::GnuHash
                                | DynamicTag::SymTab
                                | DynamicTag::SymEnt
                        )
                    }),
                    &translator,
                )
                .map_err(HashError::Dynamic)?;
                observation::collect(
                    &source,
                    raw.entries(),
                    selected.symbols.as_ref(),
                    &translator,
                    limits.hash,
                )
                .map(Some)
            })();
            let outcome = match result {
                Ok(Some(e)) => HashMetadataOutcome::Complete(e),
                Ok(None) => HashMetadataOutcome::Unavailable,
                Err(e) => HashMetadataOutcome::Failed(HashMetadataFailure::Hash(e)),
            };
            (Some(raw), outcome)
        }
    };
    HashMetadataReport {
        source,
        limits,
        raw,
        outcome,
    }
}
