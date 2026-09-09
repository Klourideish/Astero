# Explicit hash metadata and symbol-extent evidence (M20)

> A readable symbol candidate does not prove table membership. Hash count evidence is not symbol enumeration.

## Request and ownership

`elf::dynamic::hash::bounded::observe(SourceArtifact, HashMetadataLimits)` is an independent request.
Core `input/hash_metadata` delegates; CLI `hash-metadata` selects native paths and prints evidence.
No session is created. Acquisition, inspect, dynamic, descriptors and string-references keep their
previous stop boundaries. GUI is unchanged. No dependency edge or external dependency is added.

M17 bounded discovery supplies source-bound raw entries and header evidence. A filtered call to the
existing descriptor collector handles only HASH, GNU_HASH, SYMTAB and SYMENT. M18 still supports only
its original STRTAB/SYMTAB families; M20 does not widen that operation. Other tags, including malformed
NEEDED/RELA/strings, remain raw and are not followed. At most four selected keys exist, so no separate
descriptor budget is necessary: the dynamic-entry budget bounds examination and the fixed set bounds
pairing/retention. All raw entries remain auditable, including duplicates when selected pairing fails.

M8 observation now has a shared private collector, used by both its original API and this request.
SysV/GNU decoding, index checks and extent derivation are unchanged; no parallel hash parser exists.
Only [M8's established ordinary-format rules](elf_hash_extents.md) are used. No external catalogue
or new PS5-specific evidence is needed. Old SymbolTable::new retains CountUnavailable; this request
neither constructs SymbolTable nor invokes read_candidate, with_hash or enumerate.

## Supported evidence and proof

Both existing hash families are supported, in fixed SysV-then-GNU order.

- SysV retains nbucket/nchain and the full source token for header and arrays. Positive counts,
  array backing, every index below nchain and termination of reachable chains are M8 prerequisites.
  Validated nchain is the exact declared number of dynamic-symbol entries, including index zero.
  M20 preserves these stronger existing checks rather than replacing them with a header-only claim.
- GNU retains nbuckets, symoffset, bloom_size, bloom_shift, the validated prefix and CountClaim.
  M8's conservative contiguous bucket-ordered suffix must begin at symoffset; each nonempty bucket
  must start at the next expected index and terminate through a low-bit chain marker. Only after
  all such chains finish is the next index an exact count/exclusive upper bound. Empty buckets
  alone produce LowerBound(symoffset), never an exact count. No EOF/neighbor heuristic is used.
- SysV and exact GNU counts must agree. GNU lower bounds must not exceed an exact SysV count.
  Both provenance records remain present. Malformed or conflicting evidence fails the whole request;
  no source is silently discarded. Lower-bound-only GNU metadata can be Complete while exact count
  remains Unavailable.

Exact evidence additionally requires existing coherent SYMTAB/SYMENT (24 bytes). Checked count*24
and whole-range translation establish the source-backed symbol extent. This does not read symbol
bytes or validate names/entries. A missing symbol descriptor or inadequate symbol backing fails;
M20 does not downgrade that established M8 guarantee or return an unbacked trusted extent.

TrustedSymbolExtent's existing private construction and read-only getters are reused. The report
retains SourceArtifact, caller limits, original descriptor fields, hash raw fields/ranges and proof
records. Count is entries / exclusive upper index, not an inclusive index or file-derived capacity.
The report offers no symbol-entry list or implicit iterator. Future explicit consumers may use the
proof, but no new enumeration operation or mutable attachment/cache is introduced here.

## Budgets and failures

All CLI limits are mandatory; there are no defaults:

| Limit | Unit and enforcement |
|---|---|
| max-bytes / max-read-calls | Unchanged M14/M15 acquisition limits, before observation |
| max-program-headers | All program headers, before collection |
| max-dynamic-entries | Raw entries examined/retained including DT_NULL |
| max-hash-words | Shared M8 work in 32-bit words across SysV then GNU |

Hash work charges header/array reads, repeated SysV bucket/chain visits, GNU bucket/chain reads,
and two words per ELF64 bloom entry. Skipped zero buckets still consume their read; terminating
chain words consume budget. Prefix/extent arithmetic and translation do not themselves consume
word budget; translations are bounded by header count, source extent and charged traversal. A
successful exact boundary is allowed; zero permits no hash word reads. An absent hash descriptor
can be Unavailable with zero hash budget. Source/descriptor structural checks may fail before a
zero work budget is consulted. No unused bloom bits are decoded merely to spend budget.

No vector is allocated from hash counts: arrays remain source-backed. Header/count products widen
u32 to u64; virtual ranges and derived symbol byte sizes are checked. Work exhaustion is Failed,
never an exact count from a partial chain. No partial successful HashObservation is returned.
Raw dynamic evidence remains on hash-stage failure, alongside original source and limits.

HashMetadataOutcome distinguishes Complete (hash metadata validated, exact extent optional),
Unavailable (no supported descriptor / no dynamic table) and Failed. HashMetadataFailure separates
bounded discovery from existing HashError: duplicate/pairing errors, translation/BSS/source failure,
invalid/zero counts, index/cycle/layout/termination errors, WorkLimit, conflicting counts and invalid
symbol extent. Source identity, family/address and nested index/count context remain available;
original tag indices/raw values remain in the report. Identical and conflicting duplicate singleton
tags both fail under existing policy. Failed reports do not authorize count use.

## Validation and limits of claims

Generated fixtures intentionally contain invalid symbol/name bytes and unrelated malformed tags;
M20 succeeds without consuming them. Tests cover prior-command independence, original candidate
count refusal, exact/insufficient budgets, source boundaries, both-family agreement/conflict,
GNU lower-bound-only evidence, native Windows paths and immutable deterministic observations.
See [validation](validation.md) and [USAGE](../../USAGE.md) for actual commands and results.

No name lookup, string enumeration, symbol enumeration/classification, relocation decoding,
linkage, NID/ABI, admission, mapping, runtime or GUI work occurs. Hash index traversal is limited to
M8 proof rules, not runtime lookup. Count evidence is ordinary declared ELF structure, not complete
linker conformance or proof of a runnable PS5 program. GNU unusual layouts outside M8's conservative
rule remain unsupported rather than guessed. A future milestone may expose explicit bounded symbol
observation using trusted evidence; it must remain separately requested.
