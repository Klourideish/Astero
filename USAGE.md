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
