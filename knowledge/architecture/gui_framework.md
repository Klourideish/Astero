# GUI and host graphics decision — M1, 2026-09-08

Locked by user direction: Astero owns the Winit event loop and window/input/repaint lifecycle.
ImGui supplies debugger widgets only. Ash supplies direct Vulkan bindings. Vulkan is Astero's
intended primary **host graphics API**; it is not described as the PS5 guest API.
The eframe/egui/Glow proposal was rejected and removed. No fallback to Iced, Slint, eframe,
wgpu or OpenGL is implemented.

## Bounded compatibility check

| Direct GUI dependency | Exact resolved version | Purpose | Declared licence |
|---|---|---|---|
| winit | 0.30.13 | Event loop, window/input | Apache-2.0 |
| ash | 0.38.0+1.3.281 | Dynamically loaded Vulkan bindings | MIT OR Apache-2.0 |
| ash-window | 0.13.0 | Native handle to Vulkan surface | MIT OR Apache-2.0 |
| raw-window-handle | 0.6.2 | Borrowed native handle traits | MIT OR Apache-2.0 OR Zlib |
| imgui | 0.12.0 | Dear ImGui Rust bindings; tables enabled | MIT OR Apache-2.0 |
| imgui-winit-support | 0.13.0 | Input/DPI/cursor translation | MIT OR Apache-2.0 |
| imgui-rs-vulkan-renderer | 1.16.0 | Draw recording into Astero's command buffer | MIT |

Verified against Cargo metadata and published sources:
[Winit](https://docs.rs/crate/winit/0.30.13),
[Ash](https://docs.rs/crate/ash/0.38.0%2B1.3.281),
[surface bridge](https://docs.rs/ash-window/0.13.0/ash_window/fn.create_surface.html),
[ImGui](https://docs.rs/crate/imgui/0.12.0),
[platform manifest](https://docs.rs/crate/imgui-winit-support/0.13.0/source/Cargo.toml),
[renderer manifest](https://docs.rs/crate/imgui-rs-vulkan-renderer/1.16.0/source/Cargo.toml).

The platform adapter explicitly uses ImGui 0.12 and Winit 0.30; the renderer uses ImGui 0.12
and Ash 0.38. Its Winit 0.29 example dependency is dev-only: its library has no Winit dependency.
The selected normal graph has one version of each required API. The renderer accepts Astero's
instance/device/queue/pool/render pass and records into Astero's command buffer.
[Renderer API](https://docs.rs/imgui-rs-vulkan-renderer/1.16.0/imgui_rs_vulkan_renderer/struct.Renderer.html).

Maintenance caveat: ImGui 0.12 is a 2024 release, and the third-party renderer's examples lag the
chosen platform adapter. The adapter has its own [repository](https://github.com/imgui-rs/imgui-winit-support).
This compatible pinned combination does not promise latest Dear ImGui coverage or long-term maintenance.
Revalidate upgrades. imgui-sys requires a C++ toolchain (MSVC here). Dear ImGui/cimgui have their own
notices. Dependency licences do not select Astero's licence; LICENSE stays untouched. Preserve applicable
dependency notices when distributing artifacts.

## Earlier alternatives considered

These are rejected alternatives, not fallback implementations. Relative weight/fit is an architectural
assessment, not a benchmark or measured comparison.

| Candidate | Rust/Windows/maintenance | Tooling and graphics fit | Weight/licensing |
|---|---|---|---|
| egui/eframe | Rust-native, Windows supported, active releases | Panels, collapsible trees, grids/log scrolling; eframe owns the shell | Window/renderer stack; MIT/Apache-2.0; rejected by user |
| Iced | Rust-native, Windows, ongoing releases; upstream calls it experimental | Tables/pane layouts, reactive messages, scrolling; built-in wgpu/software paths differ from direct Ash | Reactive runtime and renderer stack; MIT; not adopted |
| Slint | Rust runtime plus declarative language, Windows and maintained product | Model-backed lists/table-style widgets and panels; custom graphics requires backend design | UI compiler/runtime/backends; GPL, royalty-free and commercial options differ; no licence route chosen |

Sources: [egui](https://github.com/emilk/egui), [releases](https://github.com/emilk/egui/releases),
[Iced](https://github.com/iced-rs/iced), [Iced releases](https://github.com/iced-rs/iced/releases),
[Slint](https://github.com/slint-ui/slint),
[Slint desktop/licensing discussion](https://www.slint.dev/blog/making-slint-desktop-ready).
All can consume external immutable state; that separation is an Astero contract.

## Implementation boundaries

- windowing.rs: Winit ApplicationHandler, lifecycle, event forwarding and approximately 30 Hz repaint
  scheduling. Zero-size/occluded windows wait instead of drawing.
- toolkit.rs: ImGui context, platform input adapter, inspection panes and collapsible diagnostics.
- view_model.rs: toolkit-independent SessionObserver adapter returning the common debugger report.
- vulkan/: independent GUI context, swapchain and submissions. No astero-gpu dependency.

The Vulkan path uses one graphics/present queue, FIFO swapchain and one frame in flight. Resize and
out-of-date recreate size-dependent resources; suspend/resume recreates the context. Acquire has a finite
timeout. A fence protects command/renderer reuse; present semaphores are per swapchain image.
Device idle precedes resize/destruction. Failures return errors, without graphics API fallback.

Unsafe remains forbidden in other crates. GUI denies it except in the private vulkan module because Ash
requires unsafe FFI. Window lifetime exceeds surface/context lifetime; renderer/swapchain/sync objects
are destroyed before pool/device/instance. Partial initialization retains ownership guards for cleanup.
Runtime smoke validation is not Vulkan conformance or a driver-loss test.

Cargo.lock has 219 packages including 15 workspace members and platform-specific packages.
Windows resolve has 64 nodes (15 local, 49 external). No rejected framework/renderer packages remain.
No external dependency was added to core, debug or CLI.

Future question: should GUI and emulated-GPU rendering share a Vulkan instance/device abstraction?
Do not create it before resource ownership, synchronization, failure isolation and presentation requirements
justify it. M1 GUI starts without a guest GPU; no emulator GPU implementation exists.
