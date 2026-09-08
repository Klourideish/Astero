# Final structural gap audit before M2

Audited 2026-09-08 against clean baseline d60f7d5 (following 304e600), all crate READMEs,
module inventory and dependency rules. Classifications describe physical ownership, not capability.
M2 has not started. No prototype implementation is adopted.

## Evidence and limits

Read-only reference: [PS5Rust catalogue](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/README.md).
Its [extraction limits](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/00_START_HERE/extraction_status.md)
describe a source snapshot and bounded lexical/semantic records, not executed tests or current correctness.
Historical families justify homes; they do not establish PS5 semantics or Astero APIs.

- [Kernel ownership records](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/05_KERNEL_HLE/source/ps5-libs__src__kernel__ownership/001.md)
  identify file/directory handles, generations, mounts and asynchronous request witnesses.
- [Filesystem index](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/12_FILESYSTEM_IO/INDEX.md)
  includes file operations, AIO, APR and save data; the old core filesystem trait alone is not a VFS design.
- [Audio/media index](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/11_AUDIO_MEDIA/INDEX.md)
  contains a substantial ATRAC9 family plus audio decoder, player and VideoDec2 contracts.
  [Media limits](C:/Users/Kyle/Desktop/PS5Rust_Knowledge_Catalogue/11_AUDIO_MEDIA/media_scope.md)
  explicitly distinguish bounded player state from unsupported codec/control behavior.
- The catalogue's 02_CORE_EXECUTION, 03_LOADER_LINKER, 05_KERNEL_HLE, 08_GPU, 09_SHADER,
  13_NETWORK_PLATFORM, 15_FIRMWARE_KNOWLEDGE and 16_DIAGNOSTICS indexes corroborate the
  existing execution, loader, synchronization, graphics, shader, platform and diagnostic families.

## Coverage and disposition

Paths below are relative to the named crate's src/. C = CLEAR HOME EXISTS;
A = HOME EXISTS BUT OWNERSHIP IS AMBIGUOUS; M = STRUCTURAL HOME MISSING;
D = DELIBERATELY DEFERRED. Arrows show this audit's resolution.

| Area | Class | Home / disposition |
|---|---|---|
| Native guest execution and dispatch | C | kernel/execution/host; HLE call dispatch stays hle/dispatch |
| Process, thread, context, TLS | C | kernel/process and threading/{thread,context,tls} |
| Synchronization primitives | C | kernel/synchronization and primitive children; additional primitives fit here |
| Handles / kernel objects | M -> C | Added kernel/objects; identity/lifetime only, object mechanisms remain with their owners |
| Timing / clocks | C | kernel/timing; audio/video timing remains domain-owned |
| Filesystem / VFS / host I/O | M -> C | Added kernel/filesystem; libs/filesystem remains guest contracts, loader remains parsing |
| Libc / language runtime | A -> C | libs/libc and libs/runtime; runtime restricted to language initialization/finalization and unwind/exception ABI |
| Sysmodules | C | libs/sysmodule contracts, loader/modules and dependencies for load mechanisms |
| Platform services / input | A | Named guest families can live in libs; host service/input ownership requires a focused design before implementation |
| NID registration / lookup / resolution | C | hle/registration,resolution,nids; libs/nids identifies exports; loader/imports retains metadata |
| Loader / ELF / SELF / modules | C | loader/artifact,admission,load_plan,elf,self_format,modules,metadata |
| Imports / exports / relocations | C | loader/imports,exports,relocations,dependencies |
| Firmware-derived knowledge interfaces | D | knowledge/firmware and owning subsystem records; no runtime catalogue loader until a concrete use case |
| Diagnostics / tracing / structured events | C | owner observations, core/session/observation, debug/tracing,snapshots; services never depend on debug |
| Faults / exceptions | C | kernel/execution/host captures host boundary faults; kernel/signals owns guest delivery; debug/faults inspects |
| Debugger inspection / control | C | debug/session plus threads,context,memory,breakpoints,watchpoints; mechanisms remain service-owned |
| PC sampling / address attribution | C | debug/sampling,modules,symbols,nids; actual PC acquisition belongs to kernel execution interfaces |
| Networking | A | libs/network guest contracts exist; socket/host transport service placement is unresolved, not permission to put mechanisms in libs |
| Audio | C | audio/devices,buffers,mixing,timing,output; libs/audio contracts |
| Media / codecs | M -> C; D | Added libs/media guest contracts and audio/codecs mechanisms; video decoding and cross-media playback service design deferred |
| VideoOut / VBlank / presentation | C | video/videoout children,presentation,vblank; libs/videoout guest exports |
| AGC guest API | C | libs/agc/{api,resources,submission,registration} |
| AGC GPU mechanisms | C | gpu/agc/{commands,state,resources,submission}; keep separate from exports |
| PM4 packet families | C | gpu/pm4/{packets,decode,dispatch,draw,write_data,wait_reg_mem,synchronization} |
| GPU registers | C | gpu/registers/{defaults,context,shader,uconfig,decode} |
| GPU resources / descriptors | C | gpu/resources/{buffers,images,descriptors} |
| Command buffers | C | gpu/command_buffers |
| Queues / submission / synchronization | C | gpu/submission,synchronization; backend-specific queues under backend/vulkan |
| Host Vulkan | C | gpu/backend/vulkan; GUI's independent renderer/vulkan remains separate |
| Shader decode | C | shader/rdna2/{decode,instructions,operands} |
| IR / SSA / analysis / lowering | C | shader/ir,ssa,analysis,lowering |
| SPIR-V generation | C | shader/spirv |
| Shader / pipeline caching | D | Translation artifacts belong with shader; device pipelines/cache with gpu/backend/vulkan. Cache keys, persistence and invalidation need concrete contracts first |
| GPU smoke scenarios / replay / validation | C | gpu-smoke/scenarios,fixtures,capture,replay,validation |
| GUI debugger panes / logs / diagnostics | C | gui/ui named panes; gui/model remains toolkit-independent |
| Configuration | D | Session composition inputs belong to core/session; UI preferences to gui/app; subsystem options to their owners. No global configuration bucket |
| Compatibility / workarounds | D | Evidence-backed exceptions must stay with the affected mechanism and focused knowledge record, with scope and removal conditions; no global title-patch registry |

## Minimal additions and bounded existing homes

Four documentation-only mod.rs roots close the missing homes above. Kernel objects and filesystem
must not inherit prototype global state or export-layer mechanism ownership. No universal handle API,
VFS trait, codec interface, dependency, or extra crate is invented. Audio codecs have demonstrated
growth; video codecs and cross-media playback need a separate ownership decision before work starts.

libs/runtime is strictly guest language-runtime contracts, not host execution or a service container.
libs/families stays navigation-only: implementations go into named family roots. libs/kernel means
kernel export contracts, not kernel machinery. No services/platform/misc/common/utils catch-all is added.
For later network/platform work, decide named service ownership first; do not hide it under runtime,
core, HLE or a generic platform folder. These later-domain decisions do not require premature crates
or block the pre-M2 structural boundary.

Existing parents already accommodate pipeline caching, queues, events and additional PM4/ISA families;
extra empty directories would not settle their contracts. Shared GUI/GPU Vulkan device ownership and
an observation crate remain deferred as previously recorded. No M1 code was moved or changed.

Validation is recorded in [validation.md](validation.md). The inventory now has 152 module homes;
the dependency graph remains six internal edges. The completed active item was removed with user approval; durable findings remain here.
