# M39 GetSpeakerInfo and AudioOut continuation

This query lacked a PS5Rust implementation. M38 registered none and retained the exact unresolved
identity. M39 establishes the binary write contract from firmware, then uses a deliberately logical
stereo null-sink topology corroborated by comparative implementations. No physical device claim.

## Evidence and signature

PS5Rust catalogue routing:00_START_HERE/README.md and catalogue_map.md;
15_FIRMWARE_KNOWLEDGE/INDEX.md, modules/libSceAudioOut.sprx/INDEX.md and009.md;
source record docs/firmware/403/generated/veneer_records.jsonl line1102.
The old payload path was stale. Located the same named module under the sibling ps5_prx_dump_403
corpus directory, read-only. SHA25669d7a304883235c57ffd4231d3a6f8edd79c13fbd1dcd6b0cb96f145d3f3815c.
Disassembled only the626-byte export at VA0x203f0/file0x243f0 plus helper0x4940 and focused
speaker-state references in that module. No firmware execution or payload copying into Astero.

Established signature: signed32 result GetSpeakerInfo(output_pointer RDI, selector u32 ESI).
There is NO context/user/port handle argument. The anticipated per-port signature was rejected.
0x2040e checks initialization;0x2043e returns0x80268006 if absent.
0x2041a checks null output;0x20448 returns0x8026800c. Invalid selector returns0x80268001.
Firmware version gate0x14fffff restricts selector0 on older versions,allows0/1 later;selector1
also depends on device state. Astero implements selector0;selector1 explicitly unsupported.
0x20461 supplies80 bytes to helper0x4940,which zeroes via memset. Copies at0x20566/574/582
cover output0..80,using unaligned stores; no stronger alignment requirement is established.
Success0 at0x20587. Device-specific alternate stores show dword offset4,8,16 and word20.

Decrypted catalogue routing:00_START_HERE/README.md,catalogue_map.md,01_IMAGES/INDEX.md,
05_NIDS/INDEX.md. Selected executable/firmware are outside its focused IL2CPP image inventory;
no fabricated cross-artifact match. Used current primary_real_elf via LOCAL_TEST_CORPUS instead.
SHA256A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Verified PT_LOAD translation; disassembled call0x3db98e,not arbitrary bytes executed for research.
Caller puts output at RSP+0xd0,selector0 inESI; return0x3db993 reads output byte0,requires<=2,
then reads bit0 atoffset8. It does not test EAX from this query. The next port-state call is
0x3db9d8 followed by a1000-unit delay call. Stack reserves a separate next output at+0x120,
corroborating exactly80 bytes. Initial misaligned decode prefixes were discarded,not treated as evidence.

PS5Rust surrounding baseline:13_NETWORK_PLATFORM/source/ps5-libs__src__platform/INDEX.md,
021.md and read-only platform.rs AudioOut2 handlers1280-1788; M38 service already migrates
these known functions. No matching GetSpeakerInfo implementation exists there.

Comparative evidence was needed for field semantics,not signature/size:
KytyPS5-main/src/libs/audio.h and libAudio2.cpp AudioOut2GetSpeakerInfo;
prosper-master/prosper/src/hle/audio/hle_audio.cpp audio2_get_speaker_info;
sharpemu-main/src/SharpEmu.Libs/Audio/AudioOut2Exports.cs AudioOut2GetSpeakerInfo.
Kyty/Prosper agree on type0,mask3,flags0,front angles-30/+30. SharpEmu instead writes2/rate48000
and has a skip-stack success workaround. That conflicting layout and skipped-write success were rejected.
These local reference snapshots are comparative evidence,not architectural authority.

## Layout, ownership and confidence

ABI owns explicit80-byte encoding,never host Rust padding. Typeu8 at0; reserved1..4;
masku32 at4; flagsu32 at8; reserved12..16;16 pairs of signed16 azimuth/elevation at16..80.
Size/output ordering/error conditions firmware-supported; mask/angle meanings comparative-correlated.
Unknown bits and reserved fields zero. Logical stereo: type0,mask3,flags0,angles(-30,0),(30,0),
otherszero. This is explicit null-sink policy,not firmware security/hardware topology truth.
Global speaker configuration is independent of mono/stereo/8-channel input ports; tests prove that
opening/closing such ports cannot change this global query. No fabricated handle validation.
No new dependency or scheduler. Existing AudioService owns initialization,query count and shutdown;
libs checks all80 writable bytes before write. Invalid tails/RO/overflow refuse; no successful prefix.
Null output preserves firmware error; shutdown returns controlled stop. No atomic rollback claim for
an OS write failure after preflight. Query count records successful writes only.

## Validation and continuation

Six added tests cover fields/reserved bytes,global topology across input formats,initialization/error
priority,unknown selector,full-range preflight,exact/unaligned bounds,repetition and shutdown.
Full results follow. The upper unused register bits are ignored for the established u32 selector.

## Real continuation and outcomes

One run with primary_real_elf, exact M39 USAGE command; wall250ms/containment15000ms.
Exit0/Clean;13195us overall,12154us native; execution thread29972; no redirection or suspension.
GetSpeakerInfo called once: RDI0x200800790,ESI0; returned0. Writes80 bytes through checked
encoder: byte0=0,mask@4=3,flags@8=0,azimuth/elevation pairs(-30,0),(30,0),remaining byteszero.
These values are the deterministic implementation output on a successful write; no independent
post-write memory dump is claimed. Caller advanced to PortGetState,which succeeded once with
port3/output0x2008007e0. This known helper was already migrated in M38,so no duplicate service or
extra phase was needed. All six M38 startup calls still succeeded once. No further AudioOut import
was reached before the different subsystem boundary; nothing established was withheld for M40.

Final UnresolvedFunction ordinal82: sceKernelUsleep0xD637D72D15738AC7,libkernel/libkernel.
First argument0x3e8 (1000),as predicted by the caller's next delay. Catalogue route
05_KERNEL_HLE/INDEX.md,source/ps5-libs__src__kernel__timer/INDEX.md and001.md line77,
read-only kernel/timer.rs constant24 corroborates identity. No kernel sleep implementation added.
Captured RIP0x7ff6c3e00996 is HOST landing code. RSP0x2008006b8; guest return0x1003db9ea
is a return address,not a falsely labelled sampled PC. No exception/fault/object trap.
Initial RIP0x100000070,RSP0x200800fb8,FS0x210000000 unchanged from prepared context.

Audio context1/user2/port3 retired; context512frames/depth4,modeled48kHz float-stereo;
logical speaker topology is separately runtime-owned Stereo. No buffer/output/push or audio waits,
completions,volume changes/events. Three opens/three closes,zero live objects/tickets.
Eight guest workers were interrupted from condition waits and joined;zero signal/timeouts claimed.
The M38 recursive mutex correction remains effective: no synth/synthClientBatch/effects failures.
Heap peak1055728 bytes in4MiB arena. FS restored,GS preserved;zero native/runtime reservations,
releaseerrors[],42 callbacks not invoked. Source before/afterSHA matches the value above.
Raw local report target/m39-real-1.log; no extra title and no repeated depth-chasing runs.

Supported: two-argument global query,80-byte write,selector0 caller progress and lifecycle.
Refined: all-zero no-device proposal replaced BEFORE real execution by logical stereo after
comparative evidence warned that mask0 misrepresents a configured sink. Not a physical playback claim.
Rejected: port-handle signature,rate atoffset4,skip-stack fake success,unrestricted selector1 success.
Unresolved: device-specific flags,type enumeration beyond observed0,selector1/version behavior,
non-stereo physical topology and audio quality. Firmware and comparative confidence stay distinct.

Recommend M40 as a coherent guest clock/sleep adapter migration using canonical astero-timing,
driven by the actual sceKernelUsleep stop. A bounded second-eboot smoke is now reasonable as a
validation target after that clock boundary is handled; none was run or begun in M39.

## Scope and records

ABI adds one representation under existing layouts owner; service owns logical topology/query state;
libs owns checked pointer/selector/error adaptation. No new crate/dependency/unsafe/CLI command.
Six new adapter/representation tests,existing16 service tests and mutex/supervisor/worker regressions.
GetSpeakerInfo registered/runtime-confirmed; PortGetState gains runtime-use evidence; new Usleep
observation remains unregistered. Repository NIDs250:247registered,3unregistered. ABI4.
Native execution invariant,canonical timing and deterministic shutdown unchanged.
