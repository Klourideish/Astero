# M3: immutable source binding

M4 now consumes this unchanged source API for [bounded ELF inspection](elf_inspection.md).
Statements below about absent parsing describe the M3 milestone boundary.

> Metadata may describe source-backed work, but actual source bytes are authoritative for identity and bounds.

SourceArtifact -> InputArtifact / inspect -> InspectedArtifact -> admit -> ValidatedTarget -> plan -> LoadPlan.
The existing [loader pipeline](loader_pipeline.md) keeps inspection separate from admission. Synthetic
observations are still caller-provided metadata, not claims verified by parsing bytes. Only identity,
source length and checked access become authoritative in M3; ELF/SELF parsing remains absent.

## Immutable ownership

[artifact/source.rs](../../crates/astero-loader/src/artifact/source.rs) owns SourceArtifact. Construction
consumes a Vec<u8>, places its bytes in a private Box<[u8]>, and shares a private SourceData through Arc.
Box conversion may resize the allocation once; source cloning, inspection, admission and planning do
not duplicate whole-file bytes. Length is checked for u64 representability and derived from that buffer.
There is no constructor accepting a size, identity, mutable alias or external storage provider.
Provenance is optional immutable display text. Debug output omits payload bytes.

An Arc around the complete source (rather than exposing Arc mutation helpers for the byte buffer)
keeps bytes, identity, derived length and label together. There is no interior mutation, public mutable
slice, copy-on-write replacement or source editing API. Borrowed parser/debugger views can use checked
reads later; multiple module descriptions can clone the same source without copying its bytes.

The inspected artifact, admitted target and load plan retain source handles. A plan remains readable
after original input, inspection and target handles are dropped. A copy operation stores only a small
bound range token, not another byte buffer or Arc per operation. Dropping the final owner frees the bytes;
a token alone does not retain storage and cannot be dereferenced without a matching source owner.

## Identity choice

SourceId is an opaque generated u64 object ID, allocated by a checked atomic sequence. IDs never wrap
or recycle within this process's loader instance. Allocation exhaustion returns IdentityExhausted;
it is tested with an isolated counter, without modifying global state. get() is diagnostic-only and
has no inverse public constructor. SourceArtifact equality compares identity, not byte contents.

Clones/references to one object have identical identity. Independent constructions receive distinct
IDs even for identical bytes or labels. Labels have no role in allocation or validation. Repeated
planning of the same bound input is deterministic; reconstructing a source is a different input.
Numeric IDs are not stable across runs and are not authenticity, content-equivalence or persistence
claims. A serialized numeric ID would not by itself authorize reading a source.

M3 needs object binding and lifetime, not deduplication or persistent cache keys. Therefore no
cryptographic or other content hashing is introduced. Content identity can be considered separately
when persisted plans/caches or external interchange actually require it, with explicit algorithms,
versioning and collision/trust rules. It must not silently replace source object identity.

## Checked ranges and errors

metadata::SourceRange remains an untrusted observation. Only checked_range(offset, length) constructs
BoundSourceRange, whose identity and extent fields are private. Validation order is:

1. Checked offset + length; failure is RangeOverflow with offset/length.
2. Offset <= actual length; failure is OffsetOutOfBounds.
3. End <= actual length; failure is LengthOutOfBounds.

Empty ranges are legal from offset zero through EOF; an empty range beyond EOF is rejected.
read(&token) first checks source identity (IdentityMismatch includes expected/actual IDs), then checks
bounds and returns a borrowed slice through checked indexing. Index conversion is safe because the
validated values fit the actual usize-backed buffer. Malformed external extents do not panic.
An extent() result is a detached raw value: editing it cannot change the token. Reading a token against
a different source fails even if its bytes are identical or its buffer is larger.

SourceError implements std::error::Error; no dependency is needed. LengthUnrepresentable protects the
usize-to-u64 boundary at source construction. Ordinary host allocator exhaustion remains standard
Vec/Box/Arc behavior, not a recoverable error guarantee. No arbitrary source-size cap is invented.

## Admission and plan integration

InputArtifact now requires a SourceArtifact and a description; independent identity/source_size/label
fields were removed. InspectedArtifact derives those observations from its private bound source.
No unbound compatibility constructor or test bypass remains. All M2 fixtures now own actual bytes.

Admission validates each region's source extent through the bound source and retains the resulting
token in ValidatedRegion. Rejection::SourceRange attaches the original region index and nests the
precise SourceError, also exposed through Error::source. VirtualRangeOverflow remains a separate
admission error. The old source overflow/out-of-bounds variants are replaced, not duplicated.

M2 metadata, imports/exports and relocation descriptors contain no additional source-file extents;
their addresses are virtual intents and keep their M2 checks. Future source-backed metadata must
use this checked boundary too. Synthetic declarations cannot enlarge the actual source.

CopyIntent.source is now BoundSourceRange, containing source identity and validated extent; destination
remains virtual-address intent. LoadPlan retains one matching SourceArtifact. TargetMetadata's identity,
size and label are derived snapshots, exposed read-only through target/plan getters. Detached metadata
copies cannot change the owner or construct a valid target/plan. Zero-fill work remains a separate
address range and never causes a source read. Planning only clones handles and descriptions: it does
not allocate guest memory, evaluate relocations, resolve symbols or mutate source/session/runtime state.

## Evidence and deferred work

The 16 M2 integration tests and three immutable-boundary doctests remain, adapted to source binding and
structured errors. Nine source-binding integration tests cover ownership, equality, lifetimes, concurrent
reads/ID creation, truncation, mismatch, EOF, extreme ranges and a 6,137-case bounded slice-oracle sweep.
One unit test proves no ID wraparound. Five new compile-fail doctests prevent ID forging, byte/token
mutation and independent input size/identity fields. See [validation](validation.md) for executed results.

No filesystem/mmap adapter, format parser, real input, emulator migration or runtime application exists.
Filesystem acquisition must eventually freeze bytes into this owner or explicitly preserve equivalent
immutability and lifetime guarantees; mmap is not implicitly immutable just because a view is read-only.
Future hostile-input policy must address maximum source size, region/descriptor counts, allocation
ceilings and parser work/recursion limits. No parser exists to justify those limits yet; quadratic M2
metadata checks remain a documented scaling pressure. Process-local identity also limits persistence.

Recommended M4: bounded ELF header/program-header inspection into these contracts using generated
in-memory fixtures and primary format evidence, keeping admission policy explicit. Supporting an ELF
family must be a deliberate next decision; no SELF, real binaries or runtime application is implied.

M5 retains this source API unchanged. The [ELF translator](dynamic_elf_observation.md) validates
PT_LOAD source prefixes and returns existing BoundSourceRange tokens. Dynamic reports retain the
same SourceArtifact owner; tokens never represent BSS as stored source bytes.

M6 string views also retain this unchanged byte owner. Bounded substring tokens exclude the verified
NUL terminator; generic dependency names own only their small byte sequences. No whole table is copied.
See [dynamic strings](dynamic_strings.md).

M14 adds a separate [filesystem adapter](filesystem_input.md). SourceArtifact ownership and constructors remain unchanged; filesystem metadata never replaces actual byte-derived source length.
