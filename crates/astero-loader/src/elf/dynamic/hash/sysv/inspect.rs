use super::SysVHash;
use crate::elf::dynamic::hash::{
    error::{HashError, HashFailure},
    observation::HashReader,
};
pub(in crate::elf::dynamic::hash) fn inspect(
    r: &mut HashReader<'_, '_>,
) -> Result<SysVHash, HashError> {
    r.prefix(8)?;
    let buckets = r.word(0)?;
    let chains = r.word(4)?;
    if buckets == 0 {
        return Err(r.error(HashFailure::ZeroBuckets));
    }
    if chains == 0 {
        return Err(r.error(HashFailure::ZeroSymbols));
    }
    // u32 counts widened before addition/multiplication: no u32 wrap or count-sized allocation.
    let words = u64::from(buckets) + u64::from(chains);
    let source = r.prefix(8 + 4 * words)?;
    for index in 0..words {
        let value = r.word(8 + 4 * index)?;
        if value >= chains {
            return Err(r.error(HashFailure::InvalidReference {
                index,
                value,
                count: chains,
            }));
        }
    }
    // Walk only index relationships, not symbols or names. Budget also charges repeated chain visits.
    let chain_base = 8 + u64::from(buckets) * 4;
    for bucket in 0..buckets {
        let mut current = r.word(8 + u64::from(bucket) * 4)?;
        let mut steps = 0;
        while current != 0 {
            if steps >= chains {
                return Err(r.error(HashFailure::Cycle { bucket }));
            }
            current = r.word(chain_base + u64::from(current) * 4)?;
            steps += 1;
        }
    }
    Ok(SysVHash {
        source,
        buckets,
        chains,
    })
}
