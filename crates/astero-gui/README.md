# astero-gui

Real Winit 0.30 / ImGui / Ash 0.38 Vulkan shell. main retains the core Session owner; debug UI receives only SessionObserver. GUI rendering does not depend on astero-gpu.

## Module ownership

[app/](src/app/mod.rs), [model/](src/model/mod.rs), [renderer/](src/renderer/mod.rs), [ui/](src/ui/mod.rs), [window/](src/window/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).

M13 [model/linkage](src/model/linkage/mod.rs) borrows session evidence; ui/modules renders a read-only selectable detail pane. Launch with --synthetic-linkage for shared generated evidence, or no arguments for absent evidence. See [GUI evidence](../../knowledge/architecture/gui_linkage_evidence.md).

M48 adds an opt-in host adapter under window/presentation and renderer/vulkan. It consumes
astero-video frames through the same endpoint as headless tests, without creating ImGui.
The debug UI and output window are separate application modes; neither owns emulator video
state. A Session closes its attached endpoint. No astero-video -> GUI or GUI -> GPU edge.
See [host presentation](../../knowledge/architecture/host_presentation.md) for limits and smoke.
