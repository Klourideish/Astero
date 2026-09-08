use super::GnuHash;
use crate::elf::dynamic::hash::{
    error::{HashError, HashFailure},
    extent::CountClaim,
    observation::HashReader,
};
pub(in crate::elf::dynamic::hash) fn inspect(
    r: &mut HashReader<'_, '_>,
) -> Result<GnuHash, HashError> {
    r.prefix(16)?;
    let buckets = r.word(0)?;
    let symoffset = r.word(4)?;
    let bloom_size = r.word(8)?;
    let bloom_shift = r.word(12)?;
    if bloom_size == 0 || !bloom_size.is_power_of_two() {
        return Err(r.error(HashFailure::InvalidBloomSize(bloom_size)));
    }
    if symoffset == 0 {
        return Err(r.error(HashFailure::InvalidSymbolOffset(symoffset)));
    }
    let bucket_base = 16 + u64::from(bloom_size) * 8;
    let chain_base = bucket_base + u64::from(buckets) * 4;
    let mut source = r.prefix(chain_base)?;
    // Bloom words are retained as source bytes; charge their extent without interpreting lookup bits.
    r.charge(u64::from(bloom_size) * 2)?;
    let mut cursor = u64::from(symoffset);
    let mut empty = true;
    // GNU bucket order follows the sorted symbol suffix. No count-sized allocation or sorting.
    for bucket in 0..buckets {
        let start = r.word(bucket_base + u64::from(bucket) * 4)?;
        if start == 0 {
            continue;
        }
        empty = false;
        if u64::from(start) != cursor {
            return Err(r.error(HashFailure::InvalidBucketLayout {
                bucket,
                expected: cursor,
                observed: start,
            }));
        }
        loop {
            let offset = cursor
                .checked_sub(u64::from(symoffset))
                .and_then(|i| i.checked_mul(4))
                .and_then(|i| chain_base.checked_add(i))
                .ok_or_else(|| r.error(HashFailure::Overflow))?;
            let value = r.word(offset).map_err(|error| match error {
                HashError::At {
                    failure: HashFailure::Translation(error),
                    ..
                } => r.error(HashFailure::MissingChainTerminator {
                    symbol: cursor,
                    error,
                }),
                other => other,
            })?;
            cursor = cursor
                .checked_add(1)
                .ok_or_else(|| r.error(HashFailure::Overflow))?;
            source = r.prefix(offset + 4)?; // same whole-prefix proof; includes the terminating chain word
            if value & 1 != 0 {
                break;
            }
        }
    }
    let claim = if empty {
        CountClaim::LowerBound(cursor)
    } else {
        CountClaim::Exact(cursor)
    };
    Ok(GnuHash {
        source,
        buckets,
        symoffset,
        bloom_size,
        bloom_shift,
        claim,
    })
}
