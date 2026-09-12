# M48 host presentation spine

## Ownership and scope

astero-video/presentation owns Frame, format/backing metadata, the PresentationSink
producer contract, Endpoint mailbox, weak Observation and Headless consumer. Core Session
optionally owns an endpoint and closes it on replacement, stop, detach or drop. Frontend adapter
lifetime ends before Session drop. No process-global presentation state exists.

The existing astero-gui private Vulkan Context/Swapchain/Graphics implements the host
adapter, and window/presentation owns its Winit loop. This application mode creates no
ImGui context or renderer. It reuses the M1 device, command, surface, swapchain, fence and
per-image semaphore lifetime code rather than cloning an unsafe graphics stack. Adding
a crate was unnecessary. Core -> video and GUI -> video are the two new dependency edges;
video remains dependency-free. Debug widgets are optional and are not presentation owners.
The separate synthetic window mode can become a standalone frontend without changing video.

No guest VideoOut exports, PS5 formats, flips, vblank, guest GPU, PM4, shaders or real guest
frames exist here. The host Vulkan raster adapter is not an emulator GPU implementation.
No NIDs or guest ABI records change. No guest artifacts are accessed or executed.

## Frame and queue contract

Frames carry a monotonically increasing u64 ID (per endpoint), dimensions, row stride,
host-test RGBA8/BGRA8 format, optional producer timestamp in nanoseconds and backing.
Timestamp is informational in the producer's domain, not guest vblank or a clock promise.
CPU backing is immutable Arc<[u8]>; validated extent is exactly stride * height, including
padding. Dimensions are 1..4096 by 1..2160; stride is at least width * 4 and divisible by 4;
backing is limited to 64 MiB. Checked u64 size arithmetic precedes all pixel access.

Backing::Host retains Arc<dyn HostResource> with owner-scoped identity. It is a lifetime
seam for a future backend-specific image lease, not a raw Vulkan handle or guest address.
Both current consumers explicitly reject it. Import, device compatibility, synchronization,
zero-copy and resource capability negotiation remain unimplemented; future backend resource
contracts can extend this seam without redefining frame publication or session ownership.

Endpoint is a latest-frame-wins mailbox with one pending and one outstanding consumer
receipt. New pending frames replace older pending frames, incrementing dropped. take cannot
issue a second receipt until completed; duplicate/late completion cannot increment presented.
The window retains one last frame for expose/resize redraw, and redraws do not count as new
frames. At most pending + current + briefly replaced backing are retained by this path.
Producer clones may cross threads; only Winit's application thread manipulates host graphics.
No guest worker can obtain window/device objects through the contract.

Submission rejects invalid dimensions, stride, backing, non-increasing IDs and closed endpoints.
Backend unsupported/host initialization/device errors end the smoke with diagnostics and Failed
state before deterministic close. Pending close is a drop; backend rejection of an outstanding
receipt increments refusal. Backend errors propagate to main; no success is fabricated.
Closed is terminal and remaining resources are released. Resource observations are weak.
Checksumming occurs outside the short mailbox lock; counters and metadata update together.
No queue worker, guest scheduler or process-global thread is introduced.

## Consumers and host limits

Headless drain consumes one CPU frame, records dimensions, byte count, frame ID and FNV-1a
checksum, and replaces the prior metadata. It retains no image after drain. Resource-backed
frames are refused, never treated as zero-filled pixels. Drop closes the endpoint.

The window adapter consumes the actual CPU pixel values. It coalesces horizontal identical
pixels into bounded Vulkan color-clear rectangles inside the existing render pass, scaling
the rectangles to the current surface extent. It does not substitute a hardcoded clear color
or draw the pattern through ImGui. This deliberately small host-test backend supports at most
16,384 horizontal runs per frame; higher-complexity images fail explicitly. It is suitable
for the checkerboard proof, not a general high-throughput image uploader. Texture upload and
GPU-resource import are deferred. Color-management/physical scanout fidelity is not claimed.

FIFO present, one frame in flight, acquired semaphore and per-swapchain-image completion
semaphores reuse M1. Zero extent skips draw. Resize/out-of-date waits for device idle, destroys
old size-dependent resources and recreates before draw. Context outlives submissions; Graphics
is destroyed before Window. A successful Vulkan present request is counted as presented;
that counter alone does not prove display scanout. Manual observation supplies visible proof.
Device loss and Vulkan validation-layer cleanliness have not been experimentally validated.

## Composition and observability

Ordinary Session construction has no endpoint and no window. Explicit attach/detach owns
publication shutdown. The host backend owns window/surface resources and tears them down
before detaching. The core execution Observer can weakly observe an optional endpoint;
its shared JSON Snapshot.host_presentation feeds CLI dashboard text and fingerprints.
Unavailable/Ready/Active/Failed/Closed are host states, with submitted/presented IDs,
resolution/format/bytes and receive/present/drop/refusal counters. Active requires completion.
Guest VideoOut and GPU/AGC remain NotReached. No first-entry CLI window flag was introduced.
Existing headless commands continue to report host presentation Unavailable.

## Synthetic and manual evidence

Commands: `cargo test -p astero-video --test presentation`,
`cargo test -p astero-core --test presentation`, and
`cargo run -p astero-gui -- --presentation-smoke`.
The latter emits SYNTHETIC HOST PRESENTATION, creates 640x360 red/cyan checkerboards,
alternates phase every 500 ms, and exits after 60 seconds or window close. Frontend Instant
controls this host smoke only; it implements no guest timing. No corpus configuration is used.

Windows manual observation on the current NVIDIA host confirmed both successive color phases
in one run, with screenshots 550 ms apart. A separate run maximized the window and showed
correctly scaled full-output checkerboards without a crash. Bounded runs exited 0 after
119 and 118 completed frames, respectively, with no drops/refusals. Explicit Alt+F4 produced
CloseRequested and exit 0 after 57/57 frames, with Closed state, no pending backing and no
reported Vulkan error. These counts refer to the manual smoke runs, not a fixed test oracle.
The existing no-argument Session Inspector also rendered Ready/No guest loaded and closed
with exit 0. Capture tooling initially targeted a stale/undefined window selection; that was
corrected using the enumerated exact window identity. It was not a renderer failure.

Automated tests cover frame bounds, ordering, replacement/checksum, drop policy, in-flight
completion, resource lifetime/refusal, threaded publication, session teardown, weak live
observation, guest-state separation and scaled raster coverage. Full workspace validation
results are recorded in [validation](validation.md). No physical-display test is added to CI.

Final ownership/telemetry build was also visually inspected and explicitly closed: exit 0,
49/49 frames, zero drops/refusals, matching core HostPresentation Closed snapshot. Suspension
abandons an outstanding receipt before releasing the host window so a resumed consumer is
not stuck behind it; this receipt behavior is tested, but OS suspend/resume was not manually
exercised. Policy tests admit only the video exception for GUI while still rejecting GPU.

## M48 file inventory

- `Cargo.lock`
- `README.md`
- `USAGE.md`
- `crates/astero-cli/src/entry/dashboard.rs`
- `crates/astero-core/Cargo.toml`
- `crates/astero-core/README.md`
- `crates/astero-core/src/input/entry/observability/mod.rs`
- `crates/astero-core/src/input/entry/observability/model.rs`
- `crates/astero-core/src/session/owner.rs`
- `crates/astero-gui/Cargo.toml`
- `crates/astero-gui/README.md`
- `crates/astero-gui/src/lib.rs`
- `crates/astero-gui/src/main.rs`
- `crates/astero-gui/src/renderer/vulkan/mod.rs`
- `crates/astero-gui/src/renderer/vulkan/rendering.rs`
- `crates/astero-gui/src/window/mod.rs`
- `crates/astero-video/README.md`
- `crates/astero-video/src/lib.rs`
- `crates/astero-video/src/presentation/mod.rs`
- `knowledge/architecture/README.md`
- `knowledge/architecture/crate_boundaries.md`
- `knowledge/architecture/dependency_policy.json`
- `knowledge/architecture/gui_framework.md`
- `knowledge/architecture/validation.md`
- `knowledge/indexes/DIAGNOSTIC_INDEX.md`
- `knowledge/indexes/IMPLEMENTATION_INDEX.md`
- `knowledge/indexes/MODULE_INDEX.md`
- `knowledge/indexes/SUBSYSTEM_INDEX.md`
- `knowledge/indexes/TEST_INDEX.md`
- `tools/check_policy.py`
- `tools/test_policy.py`
- `crates/astero-core/tests/presentation.rs`
- `crates/astero-gui/src/renderer/vulkan/pixels.rs`
- `crates/astero-gui/src/window/presentation.rs`
- `crates/astero-video/src/presentation/endpoint.rs`
- `crates/astero-video/src/presentation/frame.rs`
- `crates/astero-video/tests/presentation.rs`
- `knowledge/architecture/host_presentation.md`

Ignored PROJECT_STATE.json retains M48 ready_for_cleanup; generated JSON remains local-only.
