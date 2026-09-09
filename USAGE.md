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
