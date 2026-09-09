# Bounded linkage evidence reporting (M11)

> Candidate counts are exact only when report completeness is Complete.
>
> Frontend presentation must not strengthen loader evidence.

## Ownership and APIs

Loader owns `elf::dynamic::candidates::report::{collect, LinkageEvidenceReport}`. It consumes
one immutable `ElfInspection`, M8 `SymbolTable` and M9 `RelocationTables`, reusing M10 candidate
enumeration and classification. This is an observation report, not an admitted target, resolved
import, registered export or runtime module. No M6-M10 classification or evidence API was weakened.

The report contains source identity, optional caller-supplied module ID, observed artifact role,
trusted extent/hash provenance, counts and detailed symbol records. The module ID is inspection
context, not a decoded or assigned runtime identity. Names are exact owned bytes with original
source tokens. Raw attributes, section state, value/size and per-symbol relocation-use counts
remain available alongside classification. Duplicate names retain separate indices. Ordinary and
PLT counts can overlap for a canonical alias; unique counts count each record once.

Reports expose getters and immutable slices. Owned bounded name copies permit reports to outlive
parser borrows without retaining the complete source. Tokens describe provenance, not a file reopen
or runtime access mechanism. Extent evidence is a small owned clone; no target-label string copy or
whole-file copy is needed. The existing ELF-local report boundary is sufficient: no new crate,
serializer, global report registry or core state bucket is introduced.

## Completeness and budgets

`Completeness` distinguishes:

- `Complete`: every trusted symbol was collected with its associations and retained detail.
- `Partial`: a caller limit stopped collection; observed count, remaining symbol count where known,
  and the precise budget reason survive. Remaining **candidate categories** are unknown even when
  the remaining symbol count is exact. Counts describe the retained successful prefix only.
- `Unavailable(TrustedExtentMissing)`: no trusted enumeration authority; zero collected records
  does not establish zero imports/exports.
- `Failed(ReportError)`: a source mismatch or structured underlying evidence failure. Any retained
  prefix remains visible, but its counts are not whole-table totals.

Debugger report absence (`None`) is separate from an available report with unavailable extent.
A report must not be treated as complete simply because it contains no errors in its details.

`ReportBudget` bounds symbol observations, detailed records, retained name bytes, per-name/total
scan work and M9 relocation inventory work. All zero budgets are valid requests. Collection stops
before collecting another symbol at the symbol/detail limit. The name-retention limit is checked
before copying/counting that symbol; any scan already performed remains subject to scan budgets.
Counts and details therefore describe the same prefix. The tool does not collect all candidates
and discard the excess. No hidden truncation or guessed allocation ceiling is used.

M10 validates its entire canonical relocation inventory before emitting candidates. M11 preserves
that safety rule: relocation budget exhaustion produces `Partial` with **zero** collected symbols
and the nested relocation error. It does not claim a successfully observed relocation prefix or
silently drop associations. Association memory remains proportional to the caller's explicit
relocation budget. Symbol and relocation name scans have independent budgets and charge repeated
references. A future streaming association design would require a new evidence contract, not a
silent relaxation here.

Malformed names remain `Failed`; scan limits become `Partial` while retaining the original
structured errors. Source IDs are checked before collection. An unexpected iterator end is a
structured report failure, not fabricated completion. Count arithmetic is bounded by trusted
symbol count and the canonical relocation inventory; a record is added once to one symbol.

## Consistency and consumers

Counts and details come from one M10 enumeration over immutable source-bound observations. There
is no mutable second pass, runtime query, reclassification, name normalization or NID lookup.
The report owns its results. This establishes coherence **within this report**, not consistency
between independently sampled future runtime services and a session snapshot.

`astero-debug::snapshots::linkage::inspect_linkage` pins an optional `Arc<LinkageEvidenceReport>`.
It returns the same report without translation or classification. The new `LinkageEvidence`
capability describes offline report inspection, not guest module/NID attribution. All guest
control and resolution operations remain unsupported.

`astero-cli::linkage::render(snapshot, details)` renders the shared report. Partial/failed/unavailable
counts are labelled **observed**, and status/error context is printed. Optional detailed output
escapes name bytes, preserving non-UTF-8 values and preventing source control characters from
becoming terminal commands. No machine-readable mode or serialization dependency existed, so none
was added solely for convenience.

`cargo run -p astero-cli -- --linkage [--details]` currently reports **no report supplied** because
there is no target-input/composition adapter. It does not fabricate demo linkage state. Generated
integration fixtures exercise actual nonempty reports through debugger and CLI rendering. Real
file input, runtime loading and session attachment remain outside this milestone.

Future GUI panes can consume the same `LinkageSnapshot`/Arc and read-only details. No GUI pane or
new GUI dependency was added; core and GUI implementation remain unchanged.

## Dependency and navigation changes

Actual edges added: debug -> loader (already allowed) and CLI -> loader **dev-only** for generated
integration fixtures. CLI production still consumes core/debug. Policy explicitly restricts that
fixture edge to Cargo `kind=dev`, including target-specific/renamed declarations. Cargo.lock records
only these internal package edges; no external package/version changes. Loader remains dependency-free.

Dedicated homes: loader `elf/dynamic/candidates/report/`, debug `snapshots/linkage/`, CLI `linkage/`.
Reviewed index links connect report, debugger/CLI consumers, errors and tests; generated source
ranges remain authoritative navigation metadata. NID/ABI inventories remain empty.

## Validation and next boundary

Loader tests cover complete reports, mixed classifications, raw/empty/absent names, duplicates,
counts, ownership, deterministic ordering, budget sweeps, missing extent, mismatches and malformed
names. Debugger tests verify identical Arc-backed evidence and status preservation. CLI tests
verify candidate labels, escaped bytes, optional detail, non-total partial output and the absent
report executable command. See [validation](validation.md) for exact run results.

Recommended M12: a bounded **in-memory** report composition/request interface that can deliver
reports to frontends with explicit missing-input/preparation errors. Decide how an application
supplies inspected evidence before adding filesystem adapters. Keep NID/PS5 interpretation,
resolution, relocation application, HLE registration, runtime modules and execution separate.
