# Astero

**[Run Astero: CLI and GUI usage](USAGE.md)**

Astero is a Rust PS5 emulator workspace informed by lessons from PS5Rust.
M1 implements a real host session â†’ read-only observation â†’ debugger â†’ CLI/GUI slice.
No emulator implementation has been migrated. M27 stages real guest images into owned byte memory; guest execution and emulated GPU remain absent.

- `cargo run -p astero-cli`: create an unloaded host session and print state/capabilities.
- `cargo run -p astero-gui`: real Winit/ImGui window using an independent Ash/Vulkan context.
  Requires a Vulkan loader/driver and graphics/present-capable device; failures return errors without fallback.
- `cargo run -p astero-gpu-smoke`: unchanged scaffold; no guest GPU validation occurs.

Core owns session state; presentation holds read-only handles. Ready means host initialization,
not guest readiness. Guest debugger features remain explicitly unsupported.

Use [generated navigation](knowledge/indexes/README.md) to locate subsystem owners, source symbols and tests.
Start with [architecture](knowledge/architecture/README.md), [session contract](knowledge/architecture/session_observation.md),
[GUI decisions](knowledge/architecture/gui_framework.md), [agent instructions](AGENTS.md),
[active work](PROJECT_STATE.json) and [validation](knowledge/architecture/validation.md).
There are 16 packages; GUI alone has external dependencies. Rust edition 2024 is used; no MSRV is promised.
ImGui needs a C++ build toolchain. Vulkan is the primary host graphics choice, not the PS5 guest API.

Project licensing remains unresolved: LICENSE is empty and no licence grant is asserted.
[M2](knowledge/architecture/loader_pipeline.md) provides synthetic inspection, admission and immutable
load-plan contracts. [M3](knowledge/architecture/source_binding.md) binds them to immutable in-memory
source bytes and checked ranges; `cargo test -p astero-loader` exercises both. [M4](knowledge/architecture/elf_inspection.md) adds bounded ELF64 header/program-header inspection
using generated fixtures. [M5](knowledge/architecture/dynamic_elf_observation.md) adds bounded dynamic
observations and source-backed address translation. [M6](knowledge/architecture/dynamic_strings.md)
interprets bounded dynamic strings into byte-preserving dependency declarations. Those earlier APIs retain their original boundaries; M26 planning and M27 explicit staging are separate capabilities. [Next bounded milestone](knowledge/architecture/decisions.md).

[M7](knowledge/architecture/dynamic_symbols.md) observes individual dynamic symbol candidates and names;
no symbol count is guessed; the original candidate-only API still refuses enumeration.

[M8](knowledge/architecture/elf_hash_extents.md) adds source-bound hash evidence and gated symbol enumeration.

[M9](knowledge/architecture/elf_relocation_observation.md) adds bounded RELA and same-source symbol-reference observation; no relocation application.

- `cargo run -p astero-cli -- --linkage --synthetic [--details]`: compose generated in-memory
  linkage evidence into a session; no guest is loaded. Omit --synthetic for an empty session.
  See [composition](knowledge/architecture/evidence_composition.md).

M15: astero-cli acquire --path <native-path> --max-bytes <u64> --max-read-calls <u64> acquires bytes only. All limits are required; no parsing/loading occurs. See [frontend acquisition](knowledge/architecture/acquisition_frontend.md).

M25 adds [asynchronous host/manual timing](knowledge/architecture/asynchronous_timing.md), explicitly owned by opted-in sessions. Offline loader workflows start no timing worker.

M27 adds [bounded guest-image staging](knowledge/architecture/guest_image_staging.md). The primary CPU direction is [native x86-64 Windows execution](knowledge/architecture/decisions.md), not an interpreter/JIT. Staging creates no executable host mappings or guest threads.

M28 [native VM realization](knowledge/architecture/windows_native_vm.md) reserves exact guest addresses and enforces Windows page protections. Executable mappings are not permission to execute; all guest entry remains absent.
