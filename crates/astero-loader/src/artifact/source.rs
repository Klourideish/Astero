use crate::metadata::{FileOffset, SourceRange};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

/// Opaque process-local object identity, not a content hash or persistent identifier.
/// ```compile_fail
/// use astero_loader::artifact::SourceId;
/// let forged = SourceId(7);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceId(u64);
impl SourceId {
    /// Diagnostic value only; there is intentionally no inverse constructor.
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    IdentityExhausted,
    LengthUnrepresentable,
    RangeOverflow {
        offset: u64,
        length: u64,
    },
    OffsetOutOfBounds {
        offset: u64,
        source_len: u64,
    },
    LengthOutOfBounds {
        offset: u64,
        length: u64,
        source_len: u64,
    },
    IdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
}
impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source error: {self:?}")
    }
}
impl std::error::Error for SourceError {}

static NEXT_SOURCE: AtomicU64 = AtomicU64::new(1);
fn allocate_id(counter: &AtomicU64) -> Result<SourceId, SourceError> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map(SourceId)
        .map_err(|_| SourceError::IdentityExhausted)
}
struct SourceData {
    identity: SourceId,
    bytes: Box<[u8]>,
    length: u64,
    provenance: Option<String>,
}
/// Shared immutable storage. Consuming a Vec transfers ownership; clones share its boxed bytes.
/// No API returns mutable bytes or permits replacing the storage behind an identity.
/// ```compile_fail
/// use astero_loader::artifact::SourceArtifact;
/// let source = SourceArtifact::new(vec![1], None).unwrap();
/// let range = source.checked_range(0, 1).unwrap();
/// source.read(&range).unwrap()[0] = 2;
/// ```
#[derive(Clone)]
pub struct SourceArtifact {
    inner: Arc<SourceData>,
}
impl SourceArtifact {
    pub fn new(bytes: Vec<u8>, provenance: Option<String>) -> Result<Self, SourceError> {
        let length = u64::try_from(bytes.len()).map_err(|_| SourceError::LengthUnrepresentable)?;
        let identity = allocate_id(&NEXT_SOURCE)?;
        Ok(Self {
            inner: Arc::new(SourceData {
                identity,
                bytes: bytes.into_boxed_slice(),
                length,
                provenance,
            }),
        })
    }
    pub fn identity(&self) -> SourceId {
        self.inner.identity
    }
    /// Derived at construction from the actual owned byte buffer, never caller metadata.
    pub fn len(&self) -> u64 {
        self.inner.length
    }
    pub fn is_empty(&self) -> bool {
        self.inner.bytes.is_empty()
    }
    pub fn provenance(&self) -> Option<&str> {
        self.inner.provenance.as_deref()
    }
    /// Half-open range. Empty ranges are valid at offsets 0 through len, including EOF.
    /// Overflow is reported before offset bounds, then extent bounds.
    pub fn checked_range(&self, offset: u64, length: u64) -> Result<BoundSourceRange, SourceError> {
        let end = offset
            .checked_add(length)
            .ok_or(SourceError::RangeOverflow { offset, length })?;
        if offset > self.len() {
            return Err(SourceError::OffsetOutOfBounds {
                offset,
                source_len: self.len(),
            });
        }
        if end > self.len() {
            return Err(SourceError::LengthOutOfBounds {
                offset,
                length,
                source_len: self.len(),
            });
        }
        Ok(BoundSourceRange {
            identity: self.identity(),
            extent: SourceRange {
                offset: FileOffset(offset),
                size: length,
            },
        })
    }
    /// Borrow bytes without copying. A token from a different object is always rejected.
    pub fn read(&self, range: &BoundSourceRange) -> Result<&[u8], SourceError> {
        if range.identity != self.identity() {
            return Err(SourceError::IdentityMismatch {
                expected: self.identity(),
                actual: range.identity,
            });
        }
        let offset = range.extent.offset.0;
        let length = range.extent.size;
        self.checked_range(offset, length)?;
        // Checked bounds prove both indices fit the actual Vec/Box usize length.
        self.inner
            .bytes
            .get(offset as usize..(offset + length) as usize)
            .ok_or(SourceError::LengthOutOfBounds {
                offset,
                length,
                source_len: self.len(),
            })
    }
}
impl PartialEq for SourceArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
impl Eq for SourceArtifact {}
impl std::fmt::Debug for SourceArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceArtifact")
            .field("identity", &self.identity())
            .field("length", &self.len())
            .field("provenance", &self.provenance())
            .finish_non_exhaustive()
    }
}
/// Validated source extent, constructible only through SourceArtifact::checked_range.
/// A token does not own bytes; its source (retained by target/plan) provides checked access.
/// ```compile_fail
/// use astero_loader::artifact::BoundSourceRange;
/// fn alter(range: &mut BoundSourceRange) { range.extent.size = u64::MAX; }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundSourceRange {
    identity: SourceId,
    extent: SourceRange,
}
impl BoundSourceRange {
    pub fn source_id(&self) -> SourceId {
        self.identity
    }
    /// Detached raw observation; editing this copy cannot change the bound token.
    pub fn extent(&self) -> SourceRange {
        self.extent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn id_exhaustion_never_wraps_or_reuses_an_identity() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(allocate_id(&counter), Ok(SourceId(u64::MAX - 1)));
        assert_eq!(allocate_id(&counter), Err(SourceError::IdentityExhausted));
        assert_eq!(allocate_id(&counter), Err(SourceError::IdentityExhausted));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
