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

## Recommended M2: synthetic target admission and load-plan contract

Design a small read-only loader metadata/load-plan interface using synthetic fixtures only.
Core owns target admission and exposes truthful metadata/diagnostics through the same inspection path.
Test rejected/malformed inputs; admission must not imply execution or successful mapping.
Establish evidence and ownership first; no prototype migration or decrypted binary execution.
Decide whether host Ready remains distinct from target-admitted readiness before extending lifecycle.
M2 is a recommendation only and has not started.
