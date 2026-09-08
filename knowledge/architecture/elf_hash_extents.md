# M8: ELF hash metadata and trusted symbol extents

> A readable symbol entry is not proof of table membership. Enumeration requires trusted extent evidence.

> Hash structures provide metadata evidence, not runtime symbol resolution.

## Evidence and ownership

Standard-format evidence reviewed 2026-09-08:
[gABI hash table](https://gabi.xinuos.com/elf/08-dynamic.html#hash-table),
[GNU format author description](https://sourceware.org/pipermail/binutils/2006-October/049450.html),
and [glibc hash setup](https://raw.githubusercontent.com/bminor/glibc/master/elf/dl-setup_hash.c).
These support ordinary SysV nchain/count correspondence and GNU suffix, bloom and termination
layout. This is not Sony metadata evidence. No prototype code or binary catalogue was used.

Implementation lives in [hash/](../../crates/astero-loader/src/elf/dynamic/hash/mod.rs):
observation owns discovery, source reading and work accounting; sysv/gnu own structural validation;
extent owns provenance, agreement and symbol-range derivation; error owns failures. Existing
symbol_table/enumeration.rs owns the lazy symbol consumer. No generic helpers or new crates.

DT_HASH (4) and DT_GNU_HASH (0x6ffffef5) are recognized tags. The existing M5 singleton rule now
rejects repeated hash tags, including identical repeats. M5 observes the pointer values but only
explicit M8 hash observation interprets them. Other M5/M6/M7 operations do not silently enable
hash access. Unknown tags, including other potential count mechanisms, remain uninterpreted.

## SysV model

Header fields nbucket/nchain are little-endian u32, followed by those counts of u32 buckets/chains.
The full header and then full arrays must translate to a single unambiguous source-backed PT_LOAD.
Counts are widened to u64 before arithmetic. Both array kinds must contain indices below nchain.
Reachable bucket chains must terminate at zero; cycles are rejected, with work budgeting also
bounding repeated visits. No symbol lookup or hash/name consistency check is performed.

Astero's supported positive-symbol layout rejects zero buckets and zero nchain explicitly; there
must be space for the null symbol. This is a conservative inspection policy, not a PS5 restriction.
Unreachable chain slots receive index-range checks but are not traversed for cycle discovery.
SysVHash retains counts and a source token spanning header and raw arrays; HashObservation retains
the immutable source, allowing checked access to those raw bytes without array copies.
A structurally accepted nchain supplies exact declared-count evidence.

## GNU model and conservative support

The ELF64 form has four u32 header fields, bloom_size u64 bloom words, u32 buckets and u32 chains.
The bloom extent and bucket array are source-validated before visits. Positive power-of-two bloom
size is required by the supported GNU layout. bloom_shift is retained but never used for lookup,
so unusual shift values cause no shifts or fabricated bloom validation. symoffset must be positive.

Each nonzero bucket must start exactly at the next expected symbol index, beginning at symoffset.
Buckets are visited in their original order. Each consecutive chain word is source-checked, and its
low bit terminates that chain. No names or hash calculations are used to resolve symbols. A gap,
overlap, backwards reference or duplicate start fails explicitly. This conservative ordered,
contiguous suffix is the supported exact-evidence form. The complete prefix is retranslated for
each read, so chains cannot drift across mappings or into BSS. Reading to failure is never a count;
source exhaustion without a required terminator is an error, as is work-budget exhaustion.

After all nonempty buckets terminate, the next symbol index is the exact declared table extent.
If all buckets are empty (including nbuckets zero), the header/arrays remain observable with only
LowerBound(symoffset). M8 deliberately does not infer exact count from an empty GNU layout. No
arbitrary adjacent bytes, source EOF, relocation references, strings or sections establish a count.

Raw bloom/buckets/inspected chain words remain accessible through the retained source-range token.
Uninspected bytes after the final terminator are not claimed as hash data. Hash/name consistency,
bloom correctness and full linker conformance are outside this extent observation contract.

## Evidence, consistency and source binding

CountClaim explicitly distinguishes Exact and LowerBound. HashObservation exposes read-only SysV,
GNU and optional TrustedSymbolExtent records. TrustedSymbolExtent has private construction/storage,
retains the full symbol source range, exact count and up to two provenance records. Detached
ExtentEvidence values are diagnostics, not authorization. The compile-fail test guards count privacy.

SysV-only or supported GNU-only exact evidence can authorize an extent. With both present, exact
counts must agree; a partial GNU minimum must not exceed the SysV exact count. Both evidence sources
remain recorded. Malformed evidence fails the operation rather than being silently ignored in favor
of the other source. Partial-only or absent evidence returns no trusted extent. No fallback count.

An exact count requires the M5 validated SYMTAB/SYMENT descriptor. Checked count * entry_size and
full-range translation precede trusted extent creation; BSS, ambiguity, overflow and truncation fail.
The 24-byte entry-size rule remains owned by M5. Even correctly bounded hash data cannot authorize
symbol bytes outside the original SourceArtifact. A report retains source lifetime for all tokens.

## M7 integration and budgets

SymbolTable::new and read_candidate retain their original behavior; they grant no count or iterator.
SymbolTable::with_hash derives evidence internally from the same immutable ElfInspection and caller
budgets. It does not accept a caller count or detachable external evidence. Without exact evidence,
symbol_count/enumerate still return CountUnavailable. The current implementation reuses the M5
observation path twice during construction, a bounded simplicity choice rather than a new cache.

Enumeration first checks caller max_symbols and then returns a lazy, fused iterator. No symbol Vec
is allocated internally. It visits indices [0,count), reuses M7 decoding and M6 byte-name lookup,
and charges per-name plus aggregate scan budgets including NUL. It stops on the first error and
never presents a failed or partial batch as fully valid. Names borrow the table and preserve bytes.
Earlier successful entries establish only their own observations; source-backed extent is not a
certificate that every symbol's name or semantics is valid.

HashLimits.max_words is an explicit caller-selected shared budget for SysV and GNU, in that fixed
order. It charges header/array word reads, repeated chain visits and bloom extent (two words per
ELF64 bloom entry). There is no hidden default size cap, untrusted-count reservation or hash-array
allocation. Checked translations precede reads; source size bounds addressability, not symbol count.
Repeated SysV chain paths may consume budget before completion, a structured refusal rather than
unbounded quadratic work. Translation itself scales with existing PT_LOAD count; M5 header limits
and caller source/resource policy remain future pressure. Allocator policy is otherwise unchanged.

## Diagnostics and tests

HashError distinguishes dynamic-descriptor errors, mapping setup, source-tag-address-qualified
HashFailure (translation, overflow, work limit, zero/invalid counts, index errors, cycles and missing
GNU termination), missing symbol descriptor, conflicting evidence and derived symbol range failure.
The nested translation error preserves BSS/crossing/ambiguity/source distinctions. M8 introduces no
linker error category. u32 array counts widened to u64 cannot overflow their fixed-width products;
maximum-count tests prove rejection at actual source bounds instead of manufacturing u32 wrap.

[Generated tests](../../crates/astero-loader/tests/hash_extents.rs) cover SysV/GNU/both/none, partial
GNU evidence, conflicts, invalid indices, terminators, work/name limits, null/duplicate names,
source identity, full-symbol bounds and deterministic iteration. Sweeps cover 64 SysV count pairs,
32 GNU termination/source-boundary cases and symbol exact-end/one-byte-short boundaries.
See [validation](validation.md) for executed counts and index freshness.

No import/export derivation, dependency lookup, symbol binding, NIDs, relocations, PLT/GOT, HLE,
SELF, memory application, PS5-specific linking, PS5Rust migration or execution is introduced.
Admission guards and immutable load planning remain unchanged. Recommended M9: bounded relocation
record observation using established symbol extents as index evidence, without application or linking.
