use super::{CountClaim, ExtentEvidence, TrustedSymbolExtent};
use crate::elf::{
    address_translation::AddressTranslator,
    dynamic::{
        hash::{
            error::{HashError, HashKind},
            gnu::GnuHash,
            sysv::SysVHash,
        },
        observation::SymbolTableDescriptor,
    },
};
pub(in crate::elf::dynamic::hash) fn derive(
    sysv: Option<&SysVHash>,
    gnu: Option<&GnuHash>,
    symbols: Option<&SymbolTableDescriptor>,
    translator: &AddressTranslator<'_>,
) -> Result<Option<TrustedSymbolExtent>, HashError> {
    let mut evidence = Vec::new();
    let mut count = None;
    if let Some(s) = sysv {
        let n = u64::from(s.chain_count());
        count = Some(n);
        evidence.push(ExtentEvidence {
            kind: HashKind::SysV,
            source: s.source_range(),
            claim: CountClaim::Exact(n),
        });
    }
    if let Some(g) = gnu {
        let claim = g.count_claim();
        let (n, exact) = match claim {
            CountClaim::Exact(n) => (n, true),
            CountClaim::LowerBound(n) => (n, false),
        };
        if let Some(s) = count {
            if (exact && s != n) || (!exact && s < n) {
                return Err(HashError::ConflictingEvidence {
                    source: g.source_range().source_id(),
                    sysv: s,
                    gnu: n,
                    gnu_exact: exact,
                });
            }
        } else if exact {
            count = Some(n);
        }
        evidence.push(ExtentEvidence {
            kind: HashKind::Gnu,
            source: g.source_range(),
            claim,
        });
    }
    let Some(count) = count else {
        return Ok(None);
    };
    let symbols = symbols.ok_or(HashError::MissingSymbolDescriptor)?;
    let size = count
        .checked_mul(symbols.entry_size)
        .ok_or(HashError::SymbolExtentOverflow {
            count,
            entry_size: symbols.entry_size,
        })?;
    let range = translator
        .translate(symbols.address, size)
        .map_err(|error| HashError::SymbolExtent { count, error })?;
    Ok(Some(TrustedSymbolExtent {
        count,
        symbols: range,
        evidence,
    }))
}
