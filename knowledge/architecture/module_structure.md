# Known high-growth module homes

Known high-growth subsystem roots are created before implementation. New functionality must go into
the narrowest existing owner. Creating a loose top-level Rust file for an already-known growth area
requires explicit architectural justification. lib.rs and mod.rs stay focused on declarations,
exports and small wiring. Approximately 1,000-1,500 lines remains a review trigger, not a target.

The original structural pass declared 148 directory modules across 13 crates: the requested baseline plus the existing
libs families and nids navigation homes. ABI and CLI already have small, suitable M1 structures;
neither needs speculative directories. One mod.rs per home provides ownership documentation and
child declarations. At that stage only relocated M1 code used additional implementation files. M2 now adds focused
loader contract files under existing roots; see [loader pipeline](loader_pipeline.md).

The [final gap audit](structural_gap_audit.md) adds four documentation-only homes: kernel/objects,
kernel/filesystem, libs/media and audio/codecs. M4 adds five focused ELF child homes; M5 adds six dynamic/translation homes; M6 adds two dynamic string/dependency homes; M7 adds one symbol-table home; the M8 adds six hash homes; the M9 adds seven relocation homes; the current inventory totals 179 modules.

## Inventory and ownership

The [exact machine-readable inventory](module_structure.json) lists every nested path. Crate README
links navigate each root. These names do not imply implemented capabilities.

| Crate | Directory module homes | Deliberate nested areas |
|---|---:|---|
| [astero-core](../../crates/astero-core/README.md) | 4 | session/lifecycle, observation, statistics |
| [astero-loader](../../crates/astero-loader/README.md) | 38 | artifact, admission, load_plan, ELF/SELF and metadata/import/export/relocation/dependency stages |
| [astero-memory](../../crates/astero-memory/README.md) | 6 | address, mapping, protection, allocation, access, regions |
| [astero-kernel](../../crates/astero-kernel/README.md) | 18 | objects, filesystem, execution/host, threading/thread/tls/context, synchronization/mutex/condvar/rwlock/semaphore/event_flag, process/timing/signals/errno |
| [astero-hle](../../crates/astero-hle/README.md) | 6 | dispatch, providers, registration, resolution, calls, nids |
| [astero-libs](../../crates/astero-libs/README.md) | 24 | media contracts; named guest library families; pthread export families; agc/api/resources/submission/registration |
| [astero-gpu](../../crates/astero-gpu/README.md) | 28 | AGC mechanisms; PM4 packet/decode/dispatch/draw/write/wait/synchronization; register classes; resource types; Vulkan backend |
| [astero-shader](../../crates/astero-shader/README.md) | 9 | rdna2/decode/instructions/operands, IR, SSA, analysis, lowering, SPIR-V |
| [astero-video](../../crates/astero-video/README.md) | 7 | videoout/buffers/flip/status/registration, presentation, vblank |
| [astero-audio](../../crates/astero-audio/README.md) | 6 | codecs, devices, buffers, mixing, timing, output |
| [astero-debug](../../crates/astero-debug/README.md) | 14 | session, threads/context/memory, attribution, tracing/sampling, breakpoints/watchpoints/faults/GPU/snapshots |
| [astero-gui](../../crates/astero-gui/README.md) | 14 | app, window, renderer/vulkan, model, ui/session/debugger/threads/memory/modules/gpu/logs/diagnostics |
| [astero-gpu-smoke](../../crates/astero-gpu-smoke/README.md) | 5 | scenarios, fixtures, capture, replay, validation |

## AGC boundary

Guest â†’ astero-libs::agc â†’ future GPU service/interface â†’ astero-gpu::agc â†’ PM4/registers/resources/submission.

- libs/agc owns exported functions, HLE registrations, guest ABI translation and guest-visible
  resource/submission calls. It delegates mechanisms and never owns the GPU backend.
- gpu/agc owns graphics command/state handling, graphics resources, submission state and translation
  into GPU mechanisms. It never owns guest export/ABI contracts.

No service interface or AGC behavior is invented in this pass. Likewise, libs/pthread owns guest
contracts while kernel/synchronization owns machinery. HLE NID handling and libs export identities
remain distinct homes. GUI host Vulkan is independent of guest GPU mechanisms.

## Existing code relocated

| Before (relative to the crate src/) | After | Reason |
|---|---|---|
| core/session.rs | session/owner.rs; Lifecycle extracted to session/lifecycle/state.rs | Separate owner from lifecycle vocabulary without changing transitions |
| core/observation.rs | session/observation/snapshot.rs; Statistics to session/statistics/counters.rs | Keep the existing contracts in their narrow session owners |
| debug/inspection.rs, capabilities.rs, control.rs | session/ with the same filenames | Existing host-session debugger contract belongs together |
| gui/windowing.rs | window/events.rs | Winit lifecycle/input scheduling owner |
| gui/toolkit.rs | ui/toolkit.rs | Existing small M1 inspector/frame composition; future panes go in ui children |
| gui/view_model.rs | model/session_view.rs | Toolkit-independent observation adapter |
| gui/vulkan/* | renderer/vulkan/* | Preserve all Vulkan implementations and the exact unsafe boundary |
| hle/providers.rs | providers/identity.rs | Preserve the existing Provider trait; mod.rs only re-exports it |
| loader/loading.rs, relocations.rs | load_plan/mod.rs, relocations/mod.rs | M0 documentation-only roots; no parsing added |
| memory/mappings.rs, access.rs | mapping/mod.rs, access/mod.rs | Mapping and checked-access homes |
| kernel/synchronization.rs | synchronization/mod.rs | Child mechanism families |
| libs/families.rs, nids.rs | families/mod.rs, nids/mod.rs | Retain existing documentation/namespaces; new exports use explicit family roots |
| gpu/pm4.rs, registers.rs, resources.rs | corresponding directory mod.rs | Parent homes for the requested children |
| shader/decoding.rs, ir.rs | rdna2/decode/mod.rs, ir/mod.rs | ISA-specific decoding and independent IR |
| video/presentation.rs; audio/mixing.rs, output.rs; hle/resolution.rs | corresponding directory mod.rs | Preserve M0 comments and establish ownership |

Public compatibility re-exports retain core::observation, core::session types,
debug::{inspection,capabilities,control}, gui::view_model, gui::run, hle::providers::Provider,
loader::loading, memory::mappings and shader::decoding. There is one physical implementation per
contract; no duplicate models. GUI app/mod.rs wires the existing run entry, with main unchanged.
Small existing M1 UI composition stays together; the new pane homes add no expanded debugger features.

## Structural policy

check_policy.py now also runs check_structure.py. It checks required mod.rs files, immediate parent
module declarations, collisions with same-name loose files, and a bounded top-level growth filename
list. Generic misc/common/utils/helpers filenames require a narrow reason and a local architecture
evidence file in loose_file_exceptions. There are no exceptions now. Nested legitimate leaf names,
such as renderer/vulkan/context.rs, remain allowed. This is a check of the explicit inventory and
ordinary out-of-line declarations, not a general Rust parser or an arbitrary source-file size limit.

Run all Python checks with `python -m unittest discover -s tools -p "test_*.py" -v`.
See [validation](validation.md) for exact results. The structural pass preceded M2 and added no guest
APIs, parsing, execution or emulator behavior. M2 uses the same 152 module homes and six actual internal
dependency edges; its loader admission policy is documented separately.

M4 nests identification, header, program_headers, error and inspect beneath loader/elf.
Private decoding.rs contains only bounded source reads and little-endian scalar decoding.
See [ELF scope](elf_inspection.md); no unrelated crate is restructured.

M5 adds loader/elf/address_translation and dynamic/{entries,tags,observation,error}.
Translation owns only declared-image virtual ranges and checked source backing. Dynamic observation
owns tag decoding and descriptor consistency, not future string/symbol/relocation interpretation.
See [dynamic observations](dynamic_elf_observation.md).

M6 nests dynamic/string_table and dynamic/dependencies beneath loader/elf. Generic byte-name and
dependency declaration types remain under loader/dependencies. See [dynamic strings](dynamic_strings.md).

M10 nests candidate classification, evidence, imports, exports and errors under
`elf/dynamic/candidates/`; enumeration wiring consumes existing symbol/relocation owners.

M11 report ownership nests under loader elf/dynamic/candidates/report, debugger snapshots/linkage
and CLI linkage. Counts and classification must never move into frontend modules.

M12 adds core session/inputs and CLI linkage/synthetic. Report/parser mechanics stay in loader;
no neutral/shared-types crate or attachment registry is introduced.

M14 adds loader artifact/filesystem for host acquisition, with separate acquisition orchestration, bounded reads and structured errors. It does not overlap guest kernel/filesystem ownership.

M15 core/input/acquisition owns the application acquisition entry point, independent of core/session. CLI/acquisition separates argument syntax, selection state and presentation. Neither owns filesystem mechanisms.

M16 adds loader elf/inspect/bounded and elf/inspect/synthetic (shared generated fixtures), core input/inspection, and CLI inspection. Existing raw decoders remain their sole mechanism owners.

M17 adds loader elf/dynamic/bounded and shared synthetic fixtures, core input/dynamic, CLI dynamic. Shared raw traversal stays under elf/dynamic/observation; descriptor interpretation remains separate.

M18 adds loader elf/dynamic/descriptors, core input/descriptors and CLI descriptors. Shared M5 pairing stays under observation/descriptors; payload owners remain separate.

M19 adds loader elf/dynamic/string_references, core input/string_references and CLI string_references. M6 string_table remains the sole byte lookup owner.

M20 hash/bounded owns independent hash/count reports; hash/synthetic owns small generated fixtures. Core input/hash_metadata and CLI hash_metadata delegate/present only. Existing hash decoders remain authoritative.
