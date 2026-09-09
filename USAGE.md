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
