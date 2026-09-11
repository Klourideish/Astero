# astero-audio

Audio processing, mixing and host output.

## Interfaces and status

M38 implements runtime-owned AudioOut ports and AudioOut2 context queues with an astero-timing
null sink. No host playback, mixing, codecs or audio event delivery is claimed. Guest layouts and
NID contracts remain in libs; core owns lifetime. See [M38](../../knowledge/architecture/audio_startup.md).

## Module ownership

[codecs/](src/codecs/mod.rs).

[buffers/](src/buffers/mod.rs), [devices/](src/devices/mod.rs), [mixing/](src/mixing/mod.rs), [output/](src/output/mod.rs), [timing/](src/timing/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Guest library contracts or session composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M39 owns logical Stereo speaker-query state independently of input port format; no host-device
topology claim. See [M39](../../knowledge/architecture/audio_speaker_info.md).

M43 codecs/ajm owns bounded AJM lifecycle and ATRAC9 configuration parsing. Batch/decode execution remains unsupported; no codec playback claim. See [M43](../../knowledge/architecture/ajm_c11_startup.md).
