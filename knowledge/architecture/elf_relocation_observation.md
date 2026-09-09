# M9: bounded relocation observation and symbol references

> A relocation record being structurally valid does not mean Astero knows how to apply it.

> A symbol index being in range does not establish import/export semantics.

## Scope and evidence

Primary ordinary-ELF sources reviewed 2026-09-09:
[gABI relocation format](https://gabi.xinuos.com/elf/06-reloc.html) and
[glibc dynamic-link.h alias commentary](https://raw.githubusercontent.com/bminor/glibc/master/elf/dynamic-link.h).
The former specifies ELF64 RELA offset/info/signed-addend fields and high/low 32-bit symbol/type
components. The latter documents legal JMPREL suffix aliases. These are format/layout evidence,
not code copied into Astero or a basis for PS5-specific semantics. No real binaries were inspected.

## Owners and immutable bounds

[relocations/](../../crates/astero-loader/src/elf/dynamic/relocations/mod.rs) is the ELF-specific owner:

- descriptor adapts private M5 descriptor storage into trusted extents and original dynamic tag indices;
- rela decodes exact 24-byte records explicitly in little endian;
- plt classifies provenance and supported aliases;
- symbol_reference requires same-source trusted bounds and M7 symbol observation;
- observation owns source lifetime, raw/validated reads and lazy canonical enumeration;
- error separates descriptor, source, index and symbol failures.

The existing format-independent loader/relocations/ work descriptors are unchanged. Parsed records
are not promoted into load-plan work, applied to memory or used to clear admission requirements.

RelocationTables::new runs M5 dynamic observation. Missing descriptors yield an empty inventory;
incomplete groups, duplicate singleton tags, unsupported widths/formats, invalid size divisibility,
arithmetic, BSS and other translation errors remain M5 failures, nested in RelocationError::Dynamic.
No descriptor is reconstructed from file remainder. DT_RELA/RELASZ/RELAENT grants 24-byte RELA;
JMPREL/PLTRELSZ/PLTREL grants either RELA or an explicitly deferred 16-byte REL descriptor.
Ordinary DT_REL, RELR and other unimplemented tag families remain outside this milestone.

TrustedRelocationExtent privately retains table kind, encoding, declared virtual address, source
token, exact entry width/count and original dynamic-entry provenance. Only adaptation from private
M5 storage can construct it. Returned values are read-only; cloning cannot alter source identity or
counts. A compile-fail doctest protects private count storage. Count = validated size / width.

RelocationTables shares immutable source ownership and retains at most two descriptors. Indexed
access requires an owned matching extent, rejects another source, checks index < count, checks
entry-offset arithmetic and obtains a fresh checked source token before decoding. Full-table bounds
plus the checked index prove the record remains inside the descriptor, not merely somewhere in the
source. No raw-pointer casts, unsafe operations, external dependencies or count-sized arrays.

## Raw versus validated observations

read_raw returns RawRelocation: source token/identity, selected descriptor kind/index, ordinary/PLT
index labels and the exact unsigned offset/info plus signed addend. Numeric type values have no
executor or semantic classification. The offset is not translated or interpreted as a write target.
RawRelocation deliberately makes no symbol-validation claim and has no runtime consumer.

read additionally requires a SymbolTable handle and a name-scan budget. Its M8 trusted extent must
exist and match this source. The extracted symbol index must be below the trusted count; the symbol
is then observed with M7, including bounded M6 name lookup. Failures remain errors rather than a
successful record labelled vaguely valid. RelocationObservation carries both raw record and the
successful symbol observation. Detached observation values are not trusted loader authorization.

Index zero is explicitly SymbolReferenceKind::Null. It still requires trusted extent evidence and
M7's all-zero null-entry validation. It never becomes an import, absolute relocation or error solely
because it is zero. Nonzero indices become Nonzero(index), without resolving anything. No relocation
type, even a familiar numeric value, receives an exemption from these structural checks. String
bytes remain byte-preserving borrowed views; UTF-8 conversion remains optional/fallible.

## JMPREL provenance, aliases and conflicts

Separate ordinary and PLT tables retain their original identities. Identical ranges or aligned
RELA JMPREL tails of the ordinary range are supported aliases when both virtual and file coordinates
agree. TailAlias records the first ordinary index and count. Individual reads of shared records
expose both index labels, regardless of which descriptor the caller selected.

Canonical enumeration emits the ordinary prefix and then the PLT table, visiting each shared record
once. Descriptor count still describes the full original table; alias metadata explains why its tail
is not emitted a second time. Identical tables emit just the PLT pass, with both labels attached.
Disjoint tables are emitted ordinary then PLT regardless of source order; this is navigation order,
not an application sequence. Empty ranges have no overlap. Other partial/unaligned overlaps and
mixed REL/RELA aliases fail DescriptorConflict. No claim is made to support every possible ABI alias.

Duplicate compatible and incompatible singleton descriptor tags both retain M5's DuplicateTag
rejection. Nothing silently overwrites, deduplicates or chooses a preferred pointer.

A source-backed PLT REL descriptor remains inspectable with DeferredRel format and exact width/count.
Raw decoding and complete enumeration explicitly return UnsupportedRel for it, including an empty
REL descriptor. It is never decoded as RELA or silently omitted from an allegedly complete stream.

## Enumeration and resources

RelocationLimits provides max_entries, max_name_scan_bytes and max_total_name_scan_bytes. Exact
canonical count is checked against max_entries before iteration; no prefix is silently substituted.
Enumeration requires same-source trusted symbol evidence even for empty inventories. Absent relocation
tables remain observable without that evidence through the descriptor API.

The iterator is lazy and fused. It stops after an error; earlier successful observations do not prove
whole-table correctness. Natural exhaustion means all canonical records were observed successfully
only if the consumer checked every Result. Entry-budget rejection is distinct from a symbol/name
failure; exhausting the name budget yields the existing structured scan failure. Repeated symbol
references charge repeated name scans including terminators. There is no cache, symbol/relocation
Vec allocation, implicit resolution or hidden default ceiling. Borrowed names cannot outlive their
SymbolTable, and source tables retain their immutable bytes independently of the original ELF report.

## Error boundaries and validation

RelocationError preserves M5 Dynamic errors, descriptor conflicts with both source ranges, extent
identity/ownership failures, deferred REL, indexed range/arithmetic/source failures, and defensive
truncated/decode failures. SymbolExtentUnavailable, SymbolSourceMismatch, SymbolIndex and nested
SymbolObservation failures are distinct; the latter retain relocation kind/index, symbol index and
original symbol/string diagnostics. EntryBudget gives exact canonical count and caller limit.
No linker or relocation-application failure is invented.

[Generated tests](../../crates/astero-loader/tests/relocation_observation.rs) cover 15 cases including
raw maximum packed values, signed extremes, null/highest-valid/out-of-range indices, source mismatch,
missing evidence, name failures, descriptor boundaries, REL deferral, duplicate/conflicting layouts,
alias provenance and canonical enumeration. Sweeps cover 74 table sizes, six entry indices and 100
symbol/type/addend combinations. See [validation](validation.md) for executed results and indexes.

## Deferred decisions

No import/export derivation, dependency or symbol resolution, NIDs, relocation semantics/application,
PLT/GOT patching, load bias, guest memory, HLE, SELF, PS5Rust migration or execution is introduced.
M5-M8 contracts required no semantic correction. M8 caller resource policy and conservative hash
extent support remain unchanged. Broader alias layouts and REL decoding require focused future work.
Recommended M10: explicit format-independent import/export candidate classification from trusted
symbol observations, with byte identities and evidence status, still without resolution/application.
