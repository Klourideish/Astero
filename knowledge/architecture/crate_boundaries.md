# Crate boundaries

| Crate | Owns | Must not own |
|---|---|---|
| astero-abi | Guest ABI layouts, identifiers and constants. | Host-side application types or service mechanisms. |
| astero-memory | Guest mappings, protections, allocation and checked access. | Process scheduling or library exports. |
| astero-loader | Binary parsing, load plans, relocations and import metadata. | Executing guest code or owning session lifecycle. |
| astero-kernel | Processes, threads, synchronization, clocks and execution. | Guest library export contracts or application composition. |
| astero-hle | Provider interfaces, registration, resolution and dispatch. | Depending on libs or implementing guest library families. |
| astero-libs | Guest library exports and delegation to owning services. | Owning mutex machinery, scheduling or memory mapping mechanisms. |
| astero-shader | Shader decoding, IR, transformations and host generation. | GPU submission or presentation. |
| astero-gpu | Commands, registers, resources, submission and GPU backend. | Display timing or application lifecycle. |
| astero-video | Display/output state, presentation and presentation timing. | Command decoding or GPU resource ownership. |
| astero-audio | Audio processing, mixing and host output. | Guest library contracts or session composition. |
| astero-debug | Inspection, execution-control interfaces and diagnostics. | Owning emulator state or becoming a runtime dependency. |
| astero-core | Session composition, subsystem wiring and lifecycle. | Low-level mechanisms or guest ABI dumping ground. |
| astero-cli | Command-line frontend consuming application interfaces. | Owning emulator state. |
| astero-gui | GUI frontend consuming application interfaces in this workspace. | Owning emulator state or silently selecting a GUI framework. |
| astero-gpu-smoke | Isolated GPU/shader validation executable. | Session composition or reporting unperformed GPU validation. |

Core coordinates services; runtime/service crates never depend on core or debug.
Debug is an application observer, not a runtime service: it may depend on core observation interfaces.
Core MUST NOT depend on debug. CLI and GUI consume core and debug. No cycle is permitted.
GUI toolkit/renderer dependencies stay inside GUI; its M1 Vulkan context is independent of astero-gpu.
Vulkan is the chosen host graphics API, not a statement about the PS5 guest API.
Debug consumes deliberate observation and controlled inspection interfaces exposed by owners.
Frontends consume application interfaces; core owns future session state.
Libs depends on HLE, never the reverse. Pthread exports live in libs; mutex machinery lives in kernel.
ABI contains guest-defined representations only. Do not add shared host-side types there for convenience.

Native guest execution starts at `astero-kernel/src/execution/`; future host backends nest beneath it.
Loading, memory, synchronization, library families, NIDs, registers, PM4, shaders and debugging
start as nested modules in their owning crates. Add purposeful modules as interfaces emerge;
do not add crates to bypass interface design. Keep lib.rs/mod.rs focused on wiring and exports.
Approximately 1,000â€“1,500 lines triggers review, not a target or blanket size allowance.

The machine-readable allowlist covers normal, dev, build and target-specific dependencies.
Allowed edges do not mandate manifest dependencies. Every actual edge must be necessary and acyclic.

See [session ownership](session_observation.md) and [GUI/host graphics decisions](gui_framework.md).
