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

## Recommended bounded M3

Bind synthetic metadata to immutable in-memory source bytes with explicit identity/size guarantees,
and exercise the same admission/planning contract with adversarial byte-backed fixtures.
No real ELF/SELF parsing, runtime application or prototype migration is implicitly authorized.
The separate host Ready versus target-admitted lifecycle decision remains deferred until integration.
