# astero-video

Display/output state, presentation and presentation timing.

## Interfaces and status

M48 implements host presentation: validated CPU RGBA8/BGRA8 frames, an owned-resource
extension seam, a bounded thread-safe endpoint and a deterministic headless consumer.
No dependency on GUI, ImGui or Vulkan. Guest VideoOut APIs, flips, vblank and guest GPU
integration remain absent. See [host presentation](../../knowledge/architecture/host_presentation.md).

Run `cargo test -p astero-video --test presentation` for headless proof.
The optional Windows adapter runs with `cargo run -p astero-gui -- --presentation-smoke`.
This is synthetic host output only.

## Module ownership

[presentation/](src/presentation/mod.rs), [vblank/](src/vblank/mod.rs), [videoout/](src/videoout/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Command decoding or GPU resource ownership.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
