# Running Astero

Run these commands in PowerShell from the repository root (for example, C:\Astero).
Cargo builds the selected tool before launching it. None of these modes loads or executes a guest.

## Prerequisites

- Rust/Cargo with Rust 2024 edition support; no minimum Rust version is promised.
- On the validated Windows MSVC setup: a working MSVC linker/Windows SDK.
  Building the GUI also requires the C++ toolchain for ImGui.
- For the GUI: a Vulkan loader/driver and a graphics/presentation-capable GPU.
  There is no alternative-renderer fallback.
- Python is only needed for repository policy/index tooling, not the launch commands below.

## CLI

Create an unloaded host session and print its state and debugger capabilities:

~~~powershell
cargo run -p astero-cli
~~~

There is no dedicated --help mode. Invalid arguments print usage and return a nonzero exit status.

**Real filesystem input, acquisition only:** this copy/pasteable example reads the repository README:

~~~powershell
cargo run -p astero-cli -- acquire --path ".\README.md" --max-bytes 1048576 --max-read-calls 64
~~~

Replace the quoted path with your selected file. Both limits are required: the example allows at
most 1 MiB and 64 read attempts (including retries and the EOF probe); these are example values,
not defaults. Output shows the chosen limits, source identity and acquired byte length, or a
structured failure. Acquisition does not parse, load or produce linkage evidence.

**Synthetic demo, no file input:** print generated linkage evidence, optionally with retained details:

~~~powershell
cargo run -p astero-cli -- --linkage --synthetic
cargo run -p astero-cli -- --linkage --synthetic --details
~~~

Inspect linkage availability without supplying synthetic evidence:

~~~powershell
cargo run -p astero-cli -- --linkage
cargo run -p astero-cli -- --linkage --details
~~~

These last two commands report unavailable evidence for an empty session. Acquisition and linkage
are separate modes; running acquisition does not supply evidence to a later command.

### Explicit inspection (opt-in)

The separate inspect command acquires bytes, then observes ELF64 file/program headers only.
It requires the same two acquisition limits plus --max-program-headers (all header types).
Zero permits a header-only file; exceeding the budget fails without a partial report.
Successful inspection does not admit, load, execute or produce linkage.

For a repeatable synthetic input, create this small fixture once (existing files are not overwritten):

~~~powershell
cargo run -p astero-loader --example inspection_fixture -- .\target\m16-header.elf
cargo run -p astero-cli -- inspect --path ".\target\m16-header.elf" --max-bytes 1024 --max-read-calls 4 --max-program-headers 1
~~~

Expect Acquired (272 bytes), then Inspection: Complete and one observed program header.
On subsequent runs, reuse the fixture or choose a new output path for the generator.
For real input, replace the quoted path and explicitly choose suitable limits.

These two inspection commands deliberately fail with exit code 1: a header-budget refusal and
invalid ELF magic respectively. Acquisition itself still succeeds.

~~~powershell
cargo run -p astero-cli -- inspect --path ".\target\m16-header.elf" --max-bytes 1024 --max-read-calls 4 --max-program-headers 0
cargo run -p astero-cli -- inspect --path ".\README.md" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 8
~~~

### Explicit raw dynamic observation (opt-in)

The separate dynamic command requires both acquisition limits, --max-program-headers and
--max-dynamic-entries. There are no defaults. This observes raw tags/values only; it does not
interpret strings, symbols, hashes, relocations or linkage, or load/execute a guest.

Generate the small synthetic fixture once (existing output is not overwritten), then observe it:

~~~powershell
cargo run -p astero-loader --example dynamic_fixture -- .\target\m17-dynamic.elf
cargo run -p astero-cli -- dynamic --path ".\target\m17-dynamic.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 3
~~~

Expect Acquired (1536 bytes), Complete (raw table scope only), three entries including retained
DT_NULL, and Unknown(-42) with its numeric raw value. The synthetic STRTAB pointer is deliberately
unusable: this command reports the value without following it. Reuse the fixture on later runs.

DT_NULL consumes one entry of budget. This deliberately fails with EntryLimit and exit 1:

~~~powershell
cargo run -p astero-cli -- dynamic --path ".\target\m17-dynamic.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 2
~~~

Using the earlier m16-header fixture reports Unavailable (no PT_DYNAMIC), exit 0:

~~~powershell
cargo run -p astero-cli -- dynamic --path ".\target\m16-header.elf" --max-bytes 1024 --max-read-calls 4 --max-program-headers 1 --max-dynamic-entries 0
~~~

No incomplete prefix is presented as Complete. Malformed input fails with structured diagnostics.
Neither acquire nor inspect automatically runs dynamic observation; this is a separate request.

### Explicit selected descriptor observation (opt-in)

The separate descriptors command reports STRTAB/STRSZ and SYMTAB/SYMENT metadata only.
It requires all acquisition/dynamic limits plus --max-descriptors: the number of supported
families attempted (at most two currently). No defaults. Zero permits only absence of supported
families; insufficient budget fails without a partial successful list.

Generate these small synthetic fixtures once; the writer refuses existing output paths:

~~~powershell
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-valid.elf valid
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-none.elf none
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-conflict.elf conflict
~~~

Successful observation (exit 0) shows two families, original dynamic values and translated source
ranges. The symbol range proves only its first entry, not a symbol count:

~~~powershell
cargo run -p astero-cli -- descriptors --path ".\target\m18-valid.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 2
~~~

Budget refusal, no supported descriptors, and conflicting duplicate tags respectively:

~~~powershell
cargo run -p astero-cli -- descriptors --path ".\target\m18-valid.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1
cargo run -p astero-cli -- descriptors --path ".\target\m18-none.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 0
cargo run -p astero-cli -- descriptors --path ".\target\m18-conflict.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 1
~~~

Expect Failed Budget (exit 1), Unavailable NoSupportedDescriptors (exit 0), and Failed DuplicateTag
(exit 1). Fixtures deliberately contain unsuitable payload bytes: this command does not read strings,
enumerate symbols, walk hashes, decode relocations, derive linkage, load a guest or execute code.
Earlier acquire, inspect and dynamic commands never trigger this separate operation.

### Explicit string references (opt-in)

string-references observes DT_NEEDED bytes only. It does not enumerate the string table or create
resolved dependencies. All descriptor/acquisition limits plus three lookup limits are mandatory:
--max-string-references, --max-scan-bytes-per-reference and --max-total-scan-bytes.
NUL counts toward both scan limits; duplicates and empty strings consume reference budget normally.

Create small synthetic fixtures once (existing outputs are refused):

~~~powershell
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-valid.elf valid
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-raw.elf raw
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-none.elf none
~~~

Successful UTF-8 and non-UTF-8 observations (exit 0), showing "libdemo.so" and <non-UTF8: FF>:

~~~powershell
cargo run -p astero-cli -- string-references --path ".\target\m19-valid.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 11 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path ".\target\m19-raw.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 2 --max-total-scan-bytes 2
~~~

Insufficient scan bytes, insufficient reference count, and no supported references:

~~~powershell
cargo run -p astero-cli -- string-references --path ".\target\m19-valid.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 10 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path ".\target\m19-valid.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 0 --max-scan-bytes-per-reference 11 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path ".\target\m19-none.elf" --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 1 --max-string-references 0 --max-scan-bytes-per-reference 0 --max-total-scan-bytes 0
~~~

Expect ScanLimit (exit 1), ReferenceBudget (exit 1), and Unavailable (exit 0). No partial string is
returned. Empty referenced strings display <empty>; absence is Unavailable. No dependency resolution,
linkage, guest loading or execution occurs. Earlier commands never trigger this operation.

### Explicit hash metadata (opt-in)

hash-metadata observes SysV/GNU hash structures and establishes trusted symbol counts where proven.
It does not enumerate symbols, resolve names, derive linkage or load/execute a guest. All acquisition,
header and dynamic limits are mandatory, plus --max-hash-words: shared 32-bit word work across both
hash families (including repeated visits and bloom extent). No descriptor budget is needed for the
fixed selected hash/symbol-descriptor set. Exact budget succeeds; exhaustion fails without a count.

Create the small synthetic fixtures once (existing outputs are refused), then run these commands:

~~~powershell
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-sysv.elf sysv
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-both.elf both
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-conflict.elf conflict
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-none.elf none
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-lower.elf lower
cargo run -p astero-cli -- hash-metadata --path .\target\m20-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 9
cargo run -p astero-cli -- hash-metadata --path .\target\m20-both.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 18
cargo run -p astero-cli -- hash-metadata --path .\target\m20-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 8
cargo run -p astero-cli -- hash-metadata --path .\target\m20-none.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 0
cargo run -p astero-cli -- hash-metadata --path .\target\m20-conflict.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 18
cargo run -p astero-cli -- hash-metadata --path .\target\m20-lower.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 7
~~~

The five writers succeed with 1536 bytes each. The six observations respectively produce:
trusted count 3 (SysV, exit 0); corroborated count 3 (both, exit 0); WorkLimit (exit 1);
Unavailable/no hash (exit 0); ConflictingEvidence (exit 1); Complete GNU metadata with exact count
Unavailable/lower-bound-only (exit 0). Reuse files on later runs or choose fresh fixture paths.
For real input, replace the path and deliberately choose every limit. Count evidence is not a
runnable/loaded guest, symbol validation or linkage. Earlier commands never invoke this request.

### Explicit symbols (opt-in)

symbols requires exact M20 count evidence and observes every proven member in table-index order.
All acquisition/hash limits remain mandatory, plus --max-descriptors, --max-symbols,
--max-name-lookups, --max-name-scan-bytes and --max-total-name-scan-bytes. Names charge NUL and
repeated lookups; unnamed entries skip lookup. Insufficient budgets fail without a successful prefix.

Create these synthetic fixtures once (existing files are refused), then run the commands:

~~~powershell
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-sysv.elf sysv
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-raw.elf raw
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-lower.elf lower
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 2 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 5 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-lower.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-raw.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 8
~~~

The three writers create 1536 bytes each. Observations respectively show Complete with indices
0,1,2 and duplicate "alpha" names (exit 0); EntryBudget failure (exit 1); ScanLimit failure (exit 1);
Unavailable because GNU gives only a lower bound (exit 0); and Complete with <non-UTF8: FF> (exit 0).
Index zero displays <unnamed>; a referenced empty string displays <empty>. Reuse fixtures on later
runs or select new output paths. For real input, replace the path and choose limits deliberately.
Attributes such as SHN_UNDEF are structural facts, not import/export classification. No linkage,
NID/dependency resolution, guest loading or execution occurs. Earlier operations remain independent.

## GUI

Open the Winit/Vulkan/ImGui session inspector without evidence:

~~~powershell
cargo run -p astero-gui
~~~

Open it with **synthetic** linkage evidence:

~~~powershell
cargo run -p astero-gui -- --synthetic-linkage
~~~

Expect session identity, Ready host lifecycle, No guest loaded, subsystem/capability information
and a read-only linkage pane. Expand/collapse the capability section; scroll the retained symbol
list and click a row for details. The synthetic pane labels its origin and completeness, displays
observed counts and preserves raw byte names. Default launch reports no supplied linkage evidence.
Resize/maximize normally and close the window to exit. These are the only supported GUI modes;
there is no file picker or guest-loading control.

## Other executable and useful checks

~~~powershell
cargo run -p astero-gpu-smoke
~~~

This prints a scaffold notice only; it performs no GPU validation.

Build all tools without launching, or run the CLI integration tests:

~~~powershell
cargo build --workspace --all-targets
cargo test -p astero-cli
~~~

See [validation records](knowledge/architecture/validation.md) for recorded test and GUI runtime
coverage. Synthetic examples are demonstrations, not evidence that a PS5 executable runs.

## Structural symbol candidates (synthetic example)

This separate command explicitly requests symbol observations, then classifies that immutable
report. Acquisition and the older observation commands never run classification automatically.
All acquisition/hash/symbol/name limits below are required, plus `--max-classifications` in records.
Every symbol, including index zero and special indices, consumes one unit. Zero refuses nonempty
input; insufficient budget fails before any successful classification prefix. Duplicate names
remain distinct. Names and unknown numeric attributes are preserved.

Run from the repository root. Fixture writers refuse overwrites; reuse existing fixtures or
choose new output names when repeating.

```powershell
cargo run -p astero-loader --example classification_fixture -- .\target\m22-ordinary.elf ordinary
cargo run -p astero-loader --example classification_fixture -- .\target\m22-special.elf special
cargo run -p astero-cli -- classify-symbols --path .\target\m22-ordinary.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 3
cargo run -p astero-cli -- classify-symbols --path .\target\m22-ordinary.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 2
cargo run -p astero-cli -- classify-symbols --path .\target\m22-special.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 3
```

Expected: writers create 1536 synthetic bytes (exit 0). Ordinary/3 reports Complete with NullSymbol,
UndefinedCandidate and DefinitionCandidate (exit 0). Ordinary/2 reports Budget { count: 3,
maximum: 2 } (exit 1). Special/3 retains reserved section and unknown attributes as SpecialCandidate
(exit 0). The two ordinary named symbols are both GLOBAL/FUNC; only section state changes the role.
Undefined does not mean resolved import; defined/global does not mean export. No dependency
resolution, linkage, NID resolution or guest loading/execution occurs.

## Real ELF linkage evidence

`linkage-evidence` is one explicit capability request over acquired bytes: trusted symbols,
structural roles, RELA references and DT_NEEDED names. Prior commands never invoke it automatically.
It is separate from the existing synthetic session `--linkage` demonstration. No providers are
resolved, no NIDs interpreted, no relocations applied, and no guest is loaded or executed.

The ignored local configuration selects real input without embedding a machine path. Run from the
repository root; the following limits were validated on the configured small PS5Util sample:

```powershell
$corpus = Get-Content -LiteralPath .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$sample = $corpus.artifacts.linkage_sample.path
cargo run -p astero-cli -- linkage-evidence --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 1024
cargo run -p astero-cli -- linkage-evidence --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 10
```

The selected build reports 14 symbols, 11 relocations, 9 non-null symbol-associated records,
8 external-reference candidates, 5 definition candidates, and libkernel.prx/libc.prx declarations
(exit 0). The second command refuses 11 records against budget 10 (exit 1). Other builds may differ.
Definition candidates are not exports; external references are not resolved imports. Raw SCE tags
and NID-looking names remain uninterpreted. All relocation application semantics remain unsupported.

All limits are mandatory. `--max-relocations` bounds canonical records and one association per
record; aliases are counted once. Symbol limits bound candidate/use records. Name lookup and scan
budgets cover symbol names followed by dependency names, including terminators/repeated names.
Failures do not present a successful linkage prefix. Unavailable trusted symbols are explicit.

For a portable generated input (no local corpus required):

```powershell
cargo run -p astero-loader --example linkage_fixture -- .\target\m23-synthetic.elf
cargo run -p astero-cli -- linkage-evidence --path .\target\m23-synthetic.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 16 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 4 --max-name-scan-bytes 6 --max-total-name-scan-bytes 24 --max-relocations 4
```

The writer creates 1536 bytes and refuses overwrites. Reuse its output or choose a fresh filename.
The generated report has 3 symbols, 4 relocations, 1 external-reference candidate, 1 definition and
two duplicate alpha dependency declarations; both commands exit 0. This is synthetic evidence.

## PS5 identity evidence

`ps5-identity` explicitly composes real linkage evidence with PS5 descriptor/name correlation.
It reports canonical numeric NIDs, experimental artifact-local library/module IDs and raw version
bits. Missing/conflicting context stays explicit. It does not resolve providers, bind HLE, load
dependencies, apply relocations, or load/execute a guest. It is separate from `--linkage --synthetic`.

All M23 limits are required. `--max-identity-records` additionally counts every SCE metadata entry
(including unsupported/duplicate entries) plus every symbol (including null/plain names). Insufficient
budget fails before a successful identity prefix; exact count succeeds. Existing name limits now
cover metadata names after symbols and DT_NEEDED names. No plain-name hashing is performed.

Validated real runs, from the root with the ignored local corpus configuration:

```powershell
$corpus = Get-Content -LiteralPath .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$sample = $corpus.artifacts.linkage_sample.path
cargo run -p astero-cli -- ps5-identity --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 1024 --max-identity-records 1024
$sample = $corpus.artifacts.utility_build_comparison.path
cargo run -p astero-cli -- ps5-identity --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 1024 --max-identity-records 27
```

The first reports Complete, 13 canonical numeric NIDs, zero unconfirmed names and PS5Util/libkernel/
libc module declarations (exit 0). Library IDs differ across builds; they are not global identities.
The second refuses 28 records against budget 27 (exit 1); use 1024 to obtain the comparison report
(exit 0, also 13 NIDs). Seven unsupported SCE entries remain raw in each report.

Portable synthetic input (writer refuses overwrites; reuse the file or choose a fresh name):

```powershell
cargo run -p astero-loader --example identity_fixture -- .\target\m24-identity.elf
cargo run -p astero-cli -- ps5-identity --path .\target\m24-identity.elf --max-bytes 4096 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 32 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 16 --max-name-scan-bytes 64 --max-total-name-scan-bytes 256 --max-relocations 4 --max-identity-records 6
```

Expected: 2560 generated bytes, Complete, two encoded NIDs, experimental sample/library declarations
and one unsupported metadata entry (both exit 0). This is synthetic evidence, not a loaded module.

## Timing infrastructure demonstration

```powershell
cargo run -p astero-timing --example deadlines
```

This manual-clock example starts a real asynchronous worker, registers three events, cancels one,
advances logical time by 5 ms, observes the other two in registration order, and joins the worker.
Expected output (exit 0):

```text
Cancellation: Cancelled
event=1 dispatch=1 at=5000000ns lateness=0ns
event=2 dispatch=2 at=5000000ns lateness=0ns
pending=0 fired=2 cancelled=1
Worker joined. Manual host-domain demonstration; no guest execution.
```

It does not sleep for a guest or configure Windows timer resolution. Default CLI/GUI sessions and
offline artifact commands do not create a timing worker.


## Offline guest load/link plan

Use an explicit consumer and optional provider from your ignored local corpus configuration:

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
cargo run -p astero-cli -- load-plan --path $corpus.artifacts.linkage_sample.path --provider $corpus.artifacts.utility_build_comparison.path @planLimits
cargo run -p astero-cli -- load-plan --path $corpus.artifacts.utility_build_comparison.path @planLimits
cargo run -p astero-cli -- load-plan --path $corpus.artifacts.primary_real_elf.path @planLimits
```

These real-input examples produce useful **Blocked** plans (exit 1), not loaded guests. Both utility
builds report five segments, two dependencies, eight external references and eleven relocation
records. The alternate utility supplies five candidates but satisfies none of the libc/kernel
references. The executable reports five segments and 30,898 relocation records. Missing providers
and bootstrap/RELRO requirements remain explicit blockers. The bias is an explicit guest-address
intent, not a host allocation. Real provider paths have no guessed placement.

`--provider` can repeat up to `--max-providers`. Optional `--provider-alias <exact-name>` immediately
after a provider explicitly declares a DT_NEEDED match; no filename/suffix inference or dependency
search occurs. All existing acquisition/observation limits apply per artifact. `--max-plan-records`
bounds retained planning records and match/blocker details; insufficient capacity refuses the whole
plan. To exercise refusal, replace the final value of `$planLimits` with `'1'` and rerun the first
command (exit 1, Budget; no successful prefix).

Output retains exact totals and shows at most 16 reference/relocation/blocker details per list,
with explicit omitted counts. Full bounded evidence remains in the API plan. Catalogue-known names
are not Astero registrations. **PLAN ONLY: no guest memory mutated, no relocations applied, no guest
execution.** Existing synthetic linkage and independent observation commands are unchanged.

## Explicit guest-image staging (no execution)

Use `$corpus` and `$planLimits` from the load-plan block above, then:

```powershell
cargo build -p astero-cli
& .\target\debug\astero-cli.exe stage-image --path $corpus.artifacts.linkage_sample.path @planLimits --max-mapped-bytes 67108864
& .\target\debug\astero-cli.exe stage-image --path $corpus.artifacts.utility_build_comparison.path @planLimits --max-mapped-bytes 67108864
& .\target\debug\astero-cli.exe stage-image --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864
```

These are real input paths, not synthetic linkage demos. All three validated examples exit 0 with
`StagedWithPendingWork`, `MetadataOnly` protections, `Ready for execution: false`, and
`Teardown: 0 active mappings`. The utility builds each apply 2 relocations and retain 9 pending;
the executable applies 29,791 and retains 1,107 pending. Staging copies bytes into owned logical guest
regions, zero-fills tails and applies already-proven independent values. It does not resolve or load
providers. Pending target bytes remain unchanged. No guest code, constructors or HLE calls execute.

`--max-mapped-bytes` is required and bounds the sum of region memory sizes, excluding address holes.
Replace it with `1` to see explicit Budget refusal (exit 1, zero active mappings). Other arguments
are identical to load-plan, including optional explicit providers. Provider plans do not make those
providers resident. The byte backend has no executable host mapping; native execution requires a
future backend with proven placement/protection. The CLI releases all image storage before exiting.

## Windows native VM realization (no execution)

On x86-64 Windows, reuse `$corpus` and `$planLimits` from the load-plan block:

```powershell
cargo build -p astero-cli
& .\target\debug\astero-cli.exe native-map --path $corpus.artifacts.linkage_sample.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864
& .\target\debug\astero-cli.exe native-map --path $corpus.artifacts.utility_build_comparison.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864
& .\target\debug\astero-cli.exe native-map --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864
```

Validated on the current host: exact guest/host identity at bias 0x100000000, enforced OS page
protections, NativeBackedWithPendingWork, exit 0 and zero native reservations/byte mappings after
teardown. Utilities commit 16,384 bytes each; executable commits 17,432,576 bytes. Pending relocation
counts remain 9/9/1,107. No guest code executes. RELRO and execution readiness remain deferred.

`--max-native-bytes` is required and bounds both the reserved envelope (including holes) and committed
pages, independently of the logical M27 byte budget. Replace it with `1` for Budget refusal (exit 1,
zero native reservations). Exact placement collision also fails without choosing another address.
Output distinguishes logical segment permissions from effective shared-page protections and marks
widening. No writable-executable page is permitted. Other host architectures refuse explicitly.
This is separate from the unchanged byte-backed stage-image command and synthetic linkage demo.

## Native entry preparation (no execution)

Use the existing `$corpus` and `$planLimits` from the load-plan block on x86-64 Windows:

```powershell
cargo build -p astero-cli
& .\target\debug\astero-cli.exe entry-readiness --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216
```

This real-input run prepares an 8-MiB RW stack above a NOACCESS guard and a separate experimental
TLS/TCB block. Addresses are explicit placement policy, not file identities. `--max-runtime-bytes`
bounds their combined page-rounded bytes, including the guard. Limits and placements are required;
collisions refuse without choosing another address. Replace its value with `1` for budget refusal
(exit 1). The unchanged acquire/inspect/native-map commands do not start runtime preparation.

Expected validated result: raw entry `0x70`, planned RIP `0x100000070`, RSP `0x200800fb8`,
`PreparedBlocked`, `EntryReady: false`, exit 0. Exit 0 means a preparation report was produced, not that
entry is safe. Output identifies 1,107 pending relocations, unresolved providers, deferred RELRO,
experimental TLS and missing native recovery/entry adapters. No timing worker is requested.
Image, stack, TLS and byte-storage ownership counters return to zero on teardown.
**NO GUEST CODE EXECUTED.** Constructors and synthetic linkage behavior remain unchanged.

## Experimental native entry closure (preparation only)

Reuse `$corpus` and `$planLimits` above on x86-64 Windows with FSGSBASE support:

```powershell
cargo build -p astero-cli
& .\target\debug\astero-cli.exe entry-readiness --close-entry --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216
```

The explicit flag selects and prints `ExperimentalEntryOwnedInit`. This selected real workload
reports `EntryReady: true`: 836 function landings, 271 writes to 17 guarded unresolved object
identities, two startup registrations and no untreated pre-entry writes. This is permission for a
future controlled attempt, not proof the guest works. Unknown functions and object accesses stop
for diagnosis; the TCB/environment and initializer policy remain experimental. RELRO's committed
pages are read-only; its 12 KiB unmapped gap remains inaccessible. Exit 0 produces the report and
drops the ready token. **NO REAL GUEST ARTIFACT CODE EXECUTED.** No constructors or callbacks run.

For M30-specific resource refusal, repeat the command with `--max-runtime-bytes 8396800`:
M29 preparation fits, but closure's additional mappings refuse with `Budget` and exit 1.
The combined cap covers stack/TLS, RX landings and NOACCESS object traps. Without `--close-entry`,
the previous preparation command remains `PreparedBlocked`.

To exercise only Astero-owned synthetic assembly (no input file):

```powershell
cargo run -p astero-kernel --example bridge_smoke
```

This validates register/stack and FS/GS preservation, return/import landings, and controlled
illegal-instruction/access-violation recovery. It prints probe results and exits 0 on success.
It is distinct from real-artifact preparation and from the synthetic linkage demonstration.


## First real native entry (trusted experimental workload only)

This executes real guest instructions. It is **not a security sandbox**; use only the configured
`primary_real_elf`, not arbitrary binaries. Reuse `$corpus` and `$planLimits` from the load-plan
block. Validate the synthetic nonreturning-loop supervisor first:

```powershell
cargo run -p astero-kernel --example supervisor_smoke
cargo build -p astero-cli
& .\target\debug\astero-cli.exe first-entry --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000
```

The first command executes only Astero-owned loop assembly, interrupts it, restores host state,
joins and prints `SYNTHETIC LOOP STOPPED` (exit 0). The real command announces
`REAL GUEST CODE WILL EXECUTE`, constructs readiness on a dedicated thread in an isolated child,
and permits the migrated startup/libc cluster (at most 4096 calls). It stops at an unknown import, object trap,
fault, return or deadline. It never invokes callbacks/fini or automatically retries.

`--wall-ms` is required (1..500): the deadline requests native interruption, subject to host scheduling
latency, not an instruction count. `--containment-ms` is required (1000..30000) and includes preparation;
it kills/waits a stuck child as **containment failure**, not clean guest recovery. Host/HLE PCs are
never forcibly unwound as guest PCs. Ordinary `entry-readiness` remains nonexecuting.

M32's validated result advances beyond the old guarded-object stop: the owned __stack_chk_guard
is readable, both atexit registrations succeed, and 15 guest heap allocations, 17 memcpy calls,
three memcmp calls and 14 __cxa_atexit registrations complete. The next boundary is unresolved
scePthreadRwlockInit (0xe942c06b47eae230), with return address 0x1001c25b6 and RSP 0x200800e58.
The captured RIP is an Astero import landing, not that guest return address. Exit 0 means clean
controlled stop/teardown, not game boot. All native/runtime reservations were released; source
SHA256 was unchanged. Callback records are retained but never invoked in this experiment.

Named startup policy: 4 MiB guest heap/4096 live allocations, 1 MiB per primitive operation,
128 environment entries, 8192 bytes per environment string, 256 callbacks and 4096 provider calls.
Heap and owned data pages count against --max-runtime-bytes. Allocation exhaustion, invalid guest
access and policy refusal remain explicit. Throwing operator new refuses on exhaustion; no C++
unwinding is implemented. The environment starts empty and stores values in guest memory.

`Returned` is normal controlled return; `UnresolvedFunction` stops at a missing provider;
`ProviderStopped` is supported termination; `ProviderRefused` retains a provider/access refusal;
`StartupAllowanceExhausted` means the bounded call policy was exhausted. AV/illegal instruction/
guarded object stops immediately; `SupervisorExpired` captures an actual suspended PC.
`WorkerFailure`/`TimeoutKilled` mean failed containment/recovery, not success.


M33 extends the same `first-entry` command with pthread synchronization (exact libkernel providers).
The documented 250 ms / 15000 ms command successfully initializes rwlock, condition and mutex objects
then stops at unregistered `scePthreadAttrInit` (0x9ec628351cb0c0d8). It does not create guest threads.
The synchronization snapshot lists opaque guest slot/ID/kind/owner and lifecycle/wait counters.
This run observed three live objects, zero waits/wakes/timeouts, clean joined teardown and unchanged
source hash. Object snapshot metadata is not a live resource. Timed waits use Astero timing;
execution-deadline cancellation stops the bridge. Guest clock conversion is a documented initial
compatibility policy, not complete realtime/paused guest clock behavior. No additional command needed.


## M34 pthread lifecycle migration

The same explicit `first-entry` command now permits the bounded lifecycle cluster. It executes real
trusted corpus code; this is not a security sandbox. Preparation commands still never execute it.
Validated once on x86-64 Windows (run from the repository root):

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000
```

Expected: pass the previous AttrInit boundary, record bounded lifecycle calls and stop at the next
unsupported operation with clean teardown. Actual: exit 0, eight real native workers using 16 KiB
stacks and independent TLS; seven entered condition waits. Nondefault scheduling priority returned
ENOTSUP explicitly. Main stopped at `ProviderRefused / Limit` for libc `memset`
(NID 0xf334c5bc120020df): 1,352,000 requested bytes exceed the existing 1 MiB operation bound.
No failed-operation bytes were written. Elapsed 10,121 us; workers were interrupted and joined,
all native/runtime reservations=0, no release errors, source SHA256 unchanged.

Output includes guest thread ID, host identity, start/stack/TLS, state, typed completion and provider
call disposition. A stopped wait's return lane is not a successful wake. Real run: seven waits,
zero signalled wakes/timeouts. M25 tickets participate; native execution retains M31 supervision.
Capacity: 32 worker creation records, 256 live attr objects, 4096 shared provider attempts. No orphan
host thread survives runtime teardown. No full pthread scheduler or TLS destructor support is claimed.
See [lifecycle evidence](knowledge/architecture/pthread_lifecycle.md) for exact calls, addresses,
NID statuses, preserved uncertainties and the outside-cluster boundary. The synthetic commands are
`cargo test -p astero-kernel --test lifecycle`, `cargo test -p astero-kernel --test bridge`,
`cargo test -p astero-core --lib workers::tests` and
`cargo test -p astero-core --test thread_contracts`; these do not execute corpus bytes.


## M35 large libc memory and runtime continuation

Use only the trusted configured `primary_real_elf` on x86-64 Windows. Real native code executes;
this supervisor is not a security sandbox. From the repository root, the exact validated command is:

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000
```

Expected: the valid 1,352,000-byte memset passes and libc startup continues to a structured next
boundary. Actual final run (third within this migration): exit0/Clean, 13,256 us; memset returned
0x100906e68, puts recorded `RezVR Start !!!`, strcpy_s and strstr returned. Next stop:
UnresolvedFunction `sceUserServiceInitialize`, NID0x8f760cbb531534da, exact
libSceUserService/libSceUserService, ordinal400. No UserService provider was installed.

The output includes checked byte-work policy/counters and owned guest diagnostic output. Current
policy: 64 MiB logical operation, 512 MiB total attempted work, 64 KiB copy scratch, 4096 shared
provider attempts. Actual charged work1,380,184 bytes, one operation above1MiB, zero budget refusals.
Heap peak7104 bytes in the existing4MiB arena (no growth). Eight workers entered condition waits;
all were interrupted/joined at shutdown, with no signalled wake or timeout claim. All mappings
released, no release errors, source SHA256 unchanged. Full addresses/calls and all three run results
are in [M35 evidence](knowledge/architecture/libc_large_memory.md).

Synthetic checks (no corpus execution):

```powershell
cargo test -p astero-libs --test large_memory
cargo test -p astero-memory --test native_access --test heap
cargo test -p astero-core --lib supervised_native_workers_perform_large_checked_memset_on_shared_heap
```

Copies validate complete mapped/permission coverage first. Gaps, guards, overflow and read-only
writes refuse. memcpy overlap refuses; memmove handles either direction. Checked `_s` functions
retain their documented constraint-error clearing; later OS copy failure is not transactional rollback.
Preparation commands remain non-executing. No new runtime CLI flags or implicit execution were added.

## M36 UserService startup continuation

Same trusted primary_real_elf and native supervisor; this is not a security sandbox.
Exact validated command (REAL GUEST CODE WILL EXECUTE):

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $corpus.artifacts.primary_real_elf.path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000
```

Expected: UserServiceInitialize succeeds, then stop at the next outside-cluster boundary.
Actual: one run, exit0/Clean,17,851us with250ms wall limit; Initialize returned0, user0x10000000
logged in, one pending login event. Next stop: unresolved libc/libc vsnprintf,
NID0x43657E8AABE3802D, ordinal328. No formatting implementation was added in M36.
Eight workers joined, eight waits interrupted (no signal/timeout claims), reservations0,
releaseerrors[], sourceSHA unchanged. Snapshot now prints UserService lifecycle/user/queue state.

Fifteen UserService exports share process-owned state and checked outputs. Only Initialize was
observed in the real run. Synthetic tests: `cargo test -p astero-libs --test user_service`.
Full identity table, adaptations, ABI uncertainty and raw context are in
[M36 evidence](knowledge/architecture/user_service.md). No host user/account details are used.

## M37 libc formatting continuation

REAL GUEST CODE WILL EXECUTE. Use the configured trusted primary_real_elf only;
this native supervisor is not a security sandbox. Preparation commands remain non-executing.
Exact validated command (same limits as M36):

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$path = $corpus.artifacts.primary_real_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000 *> target/m37-real-1.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Exit=$result Before=$before After=$after"
```

Expected: formatting succeeds, then structured stop at the next outside-cluster boundary.
Actual: one run, exit0/Clean,12,797us; vsnprintf3 and printf3 returned36/47/38 each.
No truncation, no formatting refusal. Console now exposes three `SCREAM: couldn't create
mutex` messages for synth/synthClientBatch/effects. These are observations, not diagnosed fixes.
Next stop: unresolved NID0x836B558852288471, libSceAudioOut2/libSceAudioOut, ordinal138.
All eight workers joined,all reservations released,source SHA unchanged. No audio implemented.

The existing report adds bounded Formatting records: format/destination addresses,capacity,
required/written length,conversion count,truncation,stream and structured failure. Buffers preserve
raw bytes; snprintf returns required length even on valid truncation. Only exact owned stream
tokens work for fprintf/vfprintf; no host FILE or filesystem. Unsupported wide/long-double/hex-float/
positional/percent-n formats explicitly refuse. See [M37 contract](knowledge/architecture/libc_formatting.md).

Validated synthetic commands (no corpus execution):

```powershell
cargo test -p astero-libs --test formatting
cargo test -p astero-core native_worker_formatting -- --nocapture
```

Results:24 formatting tests and one native two-worker test pass; the latter covers mixed XMM/GP
and seven overflow arguments with clean joins/mapping release. Real runtime confirms only the
observed vsnprintf/printf string paths, not every registered conversion/export.

## M38 AudioOut and media startup

REAL GUEST CODE WILL EXECUTE. Trusted configured primary_real_elf only; this is not a security sandbox.
The same bounded command was validated once:

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$path = $corpus.artifacts.primary_real_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000 *> target/m38-real-1.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Exit=$result Before=$before After=$after"
```

Expected: previous AudioOut Initialize succeeds, then stop on a different subsystem or unknown ABI.
Actual: exit0/Clean,16489us overall/15283us native. Initialize, reset/query, context/user/port creation
all succeeded once. Context1/user2/port3; null sink,512-frame context,48kHz modeled rate. No output
buffers or audio waits occurred. All four mutex helper creations succeeded; the three failure
messages disappeared. Next stop: sceAudioOut2GetSpeakerInfo,0x0C89B3D85B7D1368,unregistered.
It has no prototype implementation and its complete output ABI remains unresolved; no fake response.
Eight workers joined; audio objects/tickets and native/runtime reservations released,source SHA unchanged.
Physical playback is not implemented. State/volume/period tests do not establish host audio quality.

Synthetic validation commands (no corpus execution):

```powershell
cargo test -p astero-audio --test output
cargo test -p astero-core --test audio --test synchronization
```

These passed16 audio-service,10 adapter and5 synchronization tests. Exact evidence, NIDs, raw
context and limitations are in [M38 architecture](knowledge/architecture/audio_startup.md).

## M39 GetSpeakerInfo and AudioOut continuation

REAL GUEST CODE WILL EXECUTE. Configured trusted primary_real_elf only; not a security sandbox.
Exact validated command, same execution bounds:

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$path = $corpus.artifacts.primary_real_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000 *> target/m39-real-1.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Exit=$result Before=$before After=$after"
```

Expected: checked speaker query succeeds,continue known AudioOut helpers,stop at a different subsystem.
Actual: exit0/Clean,13195us overall/12154us native. GetSpeakerInfo(selector0) succeeded once,
writing80 bytes at0x200800790: type0,logical stereo mask3,flags0,angles-30/+30,unknown/reservedzero.
PortGetState(port3) then succeeded. Next stop: unresolved sceKernelUsleep0xD637D72D15738AC7,
libkernel/libkernel,argument1000. No sleep implementation added; no audio buffers submitted.
Eight workers joined,audio objects/tickets and native/runtime mappings released,sourceSHA unchanged.
No physical speaker/playback claim. Selector1 remains unsupported. Mutex failures did not recur.

```powershell
cargo test -p astero-core --test audio --test synchronization
cargo test -p astero-audio --test output
```

Six new speaker tests cover ABI fields,reserved bytes,global topology independent of port input
channels,range/error/selector behavior and shutdown. Full evidence/confidence and runtime context:
[M39](knowledge/architecture/audio_speaker_info.md). No second eboot was executed.

## M40 kernel clocks and sleep continuation

The exact validated PowerShell command below executes trusted primary_real_elf using the existing
controller. REAL GUEST CODE WILL EXECUTE; this is not a security sandbox.

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$path = $corpus.artifacts.primary_real_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --wall-ms 250 --containment-ms 15000 *> target/m40-real-1.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Exit=$result Before=$before After=$after"
```

The migrated timing family uses M25. One sceKernelUsleep(1000) returned0: 1ms requested,
12.582ms observed (11.582ms lateness). No clock query was observed. The next stop was
libc/libc powf, NID0xD43D07D8A363B211. Total29.017ms; all eight workers joined,
zero pending timing/native resources, source hash unchanged. Guest timing diagnostics distinguish
requested duration, elapsed duration and lateness. No Windows precision guarantee is implied.
CPU-time domains are explicitly unsupported. No second-title run occurred.

## M41 scalar math and second-executable smoke

`first-entry` now accepts optional `--max-hle-calls` (1..65536, default 4096). This is a shared
main/worker provider admission and retention budget, independent of the mandatory wall and
containment limits. It does not disable supervision or enable execution in preparation commands.
The first primary experiment hit 4096 after 3737 sincosf calls; continuation explicitly used 65536.
REAL GUEST CODE WILL EXECUTE for the primary command below. Trusted corpus only; not a sandbox.

Validated primary continuation (no callbacks/fini executed):

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$path = $corpus.artifacts.primary_real_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
$planLimits = @('--max-bytes','16777216','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --max-hle-calls 65536 --wall-ms 250 --containment-ms 15000 *> target/m41-primary-2.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Exit=$result Before=$before After=$after"
```

Primary: powf once and sincosf 5086 times returned; next unresolved __cxa_guard_acquire,
NID 0xDC63E98D0740313C, libc/libc. 96.841 ms overall; eight workers joined, zero native/runtime
reservations, source SHA unchanged. Scalar diagnostics now preserve XMM inputs and return separately
from GP lanes. Full numerical/fenv conformance is not claimed.

Exactly one second executable was attempted using named_title_elf. The 32 MiB acquisition limit
covers its 27,908,888-byte file; execution remains 250 ms/15000 ms. This command is deliberate,
not the default. It prints load plans for both roles before the contained second preparation:

```powershell
$corpus = Get-Content .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$planLimits = @('--max-bytes','33554432','--max-read-calls','512','--max-program-headers','128','--max-dynamic-entries','1024','--max-hash-words','262144','--max-descriptors','2','--max-symbols','16384','--max-name-lookups','32768','--max-name-scan-bytes','256','--max-total-name-scan-bytes','8388608','--max-relocations','131072','--max-identity-records','32768','--image-bias','4294967296','--max-providers','2','--max-plan-records','524288')
foreach ($role in @('primary_real_elf','named_title_elf')) {
  $path = $corpus.artifacts.$role.path
  $before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
  & .\target\debug\astero-cli.exe load-plan --path $path @planLimits *> "target/m41-plan-$role.log"
  "Plan $role exit=$LASTEXITCODE SHA=$before"
}
$path = $corpus.artifacts.named_title_elf.path
$before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
& .\target\debug\astero-cli.exe first-entry --path $path @planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592 --stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216 --max-hle-calls 65536 --wall-ms 250 --containment-ms 15000 *> target/m41-second-1.log
$result = $LASTEXITCODE
$after = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
"Second Exit=$result Before=$before After=$after"
```

Both load-plan commands return exit 1 because their offline plans preserve unresolved provider
blockers; acquisition/planning succeeded. The second first-entry command also returns exit 1:
Windows native exact reservation refused at 0x100000000,size 29134848, error 487. EntryReady was
not issued; NO SECOND-TITLE GUEST CODE EXECUTED. Parent reports WorkerFailure(Some(1)), not
clean guest recovery. Do not fix this by overwriting an occupied mapping or adding a title check.
The exact cause/competing allocation is unmeasured. SourceSHA is unchanged.

Additional validated offline staging diagnostics, using `$corpus` and `$planLimits` above:

```powershell
foreach ($role in @('primary_real_elf','named_title_elf')) {
    & .\target\debug\astero-cli.exe stage-image --path $corpus.artifacts.$role.path @planLimits --max-mapped-bytes 67108864
}
```

Both exit 0, StagedWithPendingWork, zero byte mappings after teardown. These offline diagnostics
do not execute guest code and do not prove native placement. See [M41](knowledge/architecture/libc_scalar_math.md)
for the complete comparison, limitations and recommended next work. No third title was run.
