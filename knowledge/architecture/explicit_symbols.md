# Explicit trusted dynamic-symbol observation (M21)

> A candidate read is not table membership. Exact M20 evidence is required before M21 visits indices.
> SHN_UNDEF is a structural observation, not an import classification.

## Request, ownership and prerequisites

`symbol_table::bounded::observe(source, Arc<HashMetadataReport>, SymbolObservationLimits)` explicitly
consumes an existing immutable M20 report. It never derives hash evidence itself or calls the M4
artifact adapter/admission. Core delegates. The separate CLI `symbols` command explicitly acquires,
requests M20 evidence, then passes it to this consumer. Earlier commands and GUI remain unchanged.
No dependency changes. No global/session/cache state or implicit constructor enumeration exists.

Source identity must match the report, even if another source contains identical bytes. Failed hash
proof produces Failed with the original report retained; absent/exclusively lower-bound evidence
produces Unavailable without decoding. An exact M20 TrustedSymbolExtent is the only membership ticket.
M8 currently rejects zero nchain and never creates a zero-count exact token; M21 does not fabricate one.
Count N means indices [0,N), including zero, with no attempt at N. Original candidate-only APIs and
M20 remain unchanged. Historical M8 gated enumeration still exists; M21 does not route through its
with_hash constructor or change its behavior.

Before decoding: check caller max_symbols, same-source full token, checked count*24 equality, then
reuse M18's bounded descriptor operation to validate SYMTAB/SYMENT and optional STRTAB/STRSZ. The
symbol descriptor's entry size, source and starting offset must agree with the trusted extent.
The immutable source identity fixes the descriptor bytes as well as the hash evidence. Full extent
source access, arithmetic checks and fallible record reservation precede the first symbol read.

M18 reuses the proof's explicit program-header/dynamic-entry limits and M21's max_descriptors. CLI
composition therefore has two bounded discovery passes (M20 and M18); limits apply per pass, not as
an unspecified cumulative counter. The second pass does not redo hash traversal. M18 retains its
existing strict malformed optional-descriptor policy, including when every st_name would be zero.
This small composition cost avoids a second descriptor parser or widening earlier report ownership.

## Decoder and immutable records

The original M7 fixed-field decoder was extracted into private `symbol_table/decode`. Candidate
reads and M21 call this sole decoder. The helper requires a checked 24-byte range before indexing.
M7 name behavior and null-symbol validation are unchanged. SymbolFields makes raw st_shndx explicit
alongside st_name, st_info, st_other, st_value and st_size; M7's established binding/type/visibility
and Section views remain unchanged and preserve unknown numerics. No import/export eligibility is
computed. A nonzero byte in symbol zero remains InvalidNullSymbol under the existing contract.

SymbolObservationReport owns a shared source and Arc to the original M20 report, limits, outcome
and ordered records. A record owns SymbolFields and an optional immutable name-range token plus a
shared source handle. No name/table payload copies or awkward self-referential lifetimes are needed.
Records preserve index order and duplicates. Identity is source + table range + index, never name.
Raw fields and proof remain inspectable even though no runtime addresses or resolved definitions exist.

## Name and budget semantics

M6 DynamicStringTable::lookup remains the sole scanner. st_name zero is unnamed and performs no
lookup; nonzero st_name may resolve to an empty string, UTF-8, or arbitrary bytes. There is no lossy
conversion, normalization or NID interpretation. CLI displays unnamed as <unnamed>, an empty lookup
as <empty>, UTF-8 quoted/escaped, and other bytes as <non-UTF8: FF ...>. These are presentation only.

All limits are mandatory in CLI; there are no hidden defaults:

| Limit | Semantics |
|---|---|
| acquisition byte/read limits | Existing M14/M15 rules, before evidence requests |
| max-program-headers / max-dynamic-entries | Existing discovery bounds per pass, DT_NULL charged |
| max-hash-words | M20 shared hash work budget, unchanged |
| max-descriptors | M18 supported families attempted, unchanged |
| max-symbols | Whole trusted count including zero; refuse before decoding if insufficient |
| max-name-lookups | Nonzero st_name attempts; duplicates/empty/raw names count individually |
| max-name-scan-bytes | Per-name examined bytes including NUL, existing M6 semantics |
| max-total-name-scan-bytes | Aggregate examined bytes including NUL across successful lookups |

Exact boundaries succeed. Zero symbol budget refuses any current exact count. Zero lookup/scan
budgets permit only unnamed entries. Empty referenced names need one lookup and one scan byte.
A refused lookup-budget attempt is not performed; an actual failed lookup consumes an attempt and
ends the report. Non-UTF-8 bytes charge normally. Per-name allowance is min(per,total remaining).
A successful lookup subtracts examined bytes including NUL, not merely returned bytes. Failure is
terminal: no scan prefix or earlier records become a successful list, and no budget is reused.
Failure's remaining_scan_bytes describes the allowance before that failed lookup; nested M6 error
retains offset/table/scan-limit context. It is not a reusable leftover budget. No whole-string-table
scan occurs and no name is attempted beyond exact symbol membership.

## Status and error policy

Complete means all N entries and requested names succeeded. Unavailable means no exact proof.
Failed means mismatched source, failed proof, descriptor failure, extent mismatch, entry budget,
allocation refusal, null/entry decoding or lookup failure. NameLookupBudget retains index, st_name,
requested attempt and limit; Symbol failures retain index, completed count, attempted lookups,
pre-attempt scan allowance and original SymbolError/M6 error. Source/proof/budgets remain in every
report. No permissive partial name or successful prefix state was added.

## Scope and evidence

Tests exercise SysV/GNU/corroboration, lower-bound/no-count refusal, source mismatch, full extent,
entry/name budgets, source identity, exact order/duplicates, raw fields, empty/unnamed/raw names,
unknown attributes, deterministic observation, earlier CLI independence and native Windows paths.
See [validation](validation.md) and [USAGE](../../USAGE.md) for executed commands and results.
Current Astero M6/M7/M8/M18/M20 contracts suffice; no external catalogue or real binary was used.

No linkage candidate derivation, imports/exports, relocation association, dependency/library/NID
resolution, ABI, admission, guest state, execution or GUI work is introduced. A future separately
requested classification milestone must make that semantic transition explicit; it is not automatic
from complete symbol observation. GNU conservative layout and repeated bounded discovery remain
known costs/limitations rather than reasons to expand M21.
