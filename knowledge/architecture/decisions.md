# Decisions and next bounded milestone

M1 locks debug â†’ core and forbids core â†’ debug; runtime services depend on neither.
Core owns session/observation contracts. CLI and GUI consume the same debugger report.
A separate observation/interface crate is only a future option if core is outgrown.

Astero owns its Winit 0.30 event loop. ImGui widgets remain isolated in GUI.
Ash 0.38/Vulkan is the primary host graphics choice, not a claim about the PS5 guest API.
GUI owns an independent Vulkan context and runs with no guest GPU. See [GUI decisions](gui_framework.md).
Future GUI/emulated-GPU device sharing is unresolved; no abstraction is added prematurely.

Project licence, MSRV/toolchain pin and supported host matrix remain unresolved.
LICENSE remains empty pending approval. Windows has scoped validation; portability is not claimed.
Git initialization and the baseline commit were authorized after M1 cleanup approval. No push is authorized.

## M2: synthetic target admission and load-plan contracts

[Loader contracts](loader_pipeline.md) now put admission in astero-loader, correcting the earlier
recommendation that core own admission. Core owns future composition; its M1 session is unchanged.
Inspection, admission and planning are distinct; acceptance implies neither mapping nor execution.
M2 remains dependency-free and admits only synthetic descriptions. Real ELF/SELF is deferred.

## M3: immutable source binding

[Source binding](source_binding.md) replaces caller-supplied identity/length with a shared immutable
byte owner and generated process-local object identity. No content hashing or dependencies are needed.
Admission validates source extents against actual bytes; plans retain source handles and checked tokens.
No filesystem adapter, parsing, session integration or application occurs.

## Recommended bounded M4

Inspect ELF headers/program headers using generated in-memory fixtures and primary format evidence,
adapting into the existing source-bound inspection/admission contract. Decide supported ELF admission
explicitly; no SELF, real binaries or runtime application is implicitly authorized. M4 has not begun.
The separate host Ready versus target-admitted lifecycle decision remains deferred until integration.
