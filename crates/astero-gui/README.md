# astero-gui

Real Winit 0.30 / ImGui / Ash 0.38 Vulkan shell. main retains the core Session owner; presentation receives only SessionObserver. GUI rendering does not depend on astero-gpu.

## Module ownership

main: composition; windowing: lifecycle/input/repaint; toolkit: widgets; view_model: observation adapter; vulkan: private instance/device/swapchain/submission boundary.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).
