# M38 AudioOut and media startup

AudioOut mechanisms live in astero-audio/output/service; guest adapters in libs/audio/exports;
core runtime owns the service and shares M25 scheduler. Audio -> timing is a new permitted
edge; libs/core depend on audio. No process-global service, extra scheduler or physical device.

## Migration evidence

PS5Rust catalogue:00_START_HERE/README.md and catalogue_map.md;
11_AUDIO_MEDIA/INDEX.md, audio_port_limits.md,
source/ps5-libs__src__audio_out/INDEX.md and001.md; read-only counterpart
crates/ps5-libs/src/audio_out.rs (complete port lifecycle/format/buffer/volume/state contract).
13_NETWORK_PLATFORM/INDEX.md, source/ps5-libs__src__platform/INDEX.md,
021.md AudioContextState/period/queue; focused counterpart platform.rs constants and
AudioOut2 registration range1280-1788 and tests4150-4183. Catalogue025 etc were not broadly read.
05_KERNEL_HLE/INDEX.md and source/ps5-libs__src__kernel__threading__mutex/INDEX.md,
with counterpart kernel/threading/mutex.rs: posix_mutex and init_guest_mutex use token identity.
Astero M25/M33/M34/M37 architecture, old run and current owner implementations were inspected.
No new external emulator source or firmware binary was consulted; reference claims in prototype
comments remain inherited corroboration, not new independent firmware proof.

## Ownership and semantics

Process-owned AudioService retains distinct Legacy/Context/Port/User kinds. Handles increase
without reuse;16 live legacy ports,32 per other kind,4096 cumulative creations; publication output
preflight and rollback. Context destroy refuses live child ports. Runtime closes/cancels everything
before joining workers; no host handle escapes. Diagnostics retain bounded retired records.

Legacy PCM formats0-7: signed16 or float32,mono/stereo/eight channels,48000Hz only;
1-65536 frames, supported prototype port types0-4,10,126,127; unknown high format bits/rates refuse.
Volume retains selected8 lanes,initial127. GetPortState writes32 deterministic bytes.
Batch descriptors are16 bytes (u32 handle at0,pointer at8),1-25 entries. Every referenced port
and source is checked/copied before any admission. No partial-success batch. PCM is owned by
queues until modeled completion or cancellation. Legacy output returns configured frame count
on admission; a full port blocks for capacity. Null output is a no-op after handle validation.

AudioOut2 reset writes64 bytes with inherited defaults at0/4/12/16/20; queue depth offset12,
grain offset16,48000Hz default. Query reports inherited1MiB work area; Create validates it and
publishes an owned handle. Context work area stays guest-owned, no asynchronous pointer retained.
Port config rate offset8 is checked. Unknown nonempty context/port attributes explicitly stop;
the prototype's ignored-attribute success is not migrated. Mastering accepts only neutral flags0.
System state is the inherited neutral64-byte structure, not physical-device capabilities.
Speaker memory query bounds count1-32 without clamping invalid requests into success.
User/context/port lifecycle,queue level and state supported; no audio event callbacks installed.
No guest audio mixing/encoded decoding or host playback is claimed.

## Timing and resource policy

TimedNullSink uses M25 tickets, one per queued unit, ceiling(frames*1e9/rate) nanoseconds.
This is experimental modeled device time, not actual speaker output or exact PS5 wake precision.
Queues retire fired tickets when observed. Context queue depth1-32,nonblocking full ->NOT_READY,
blocking waits outside the central lock on oldest ticket. Close/shutdown cancels tickets and
returns interruption, never fake completion. Runtime deadline caps waits; a fired deadline ticket
is not an audio completion if its real modeled period is still pending. No sleep polling or new
Windows timer-resolution policy. Batch ticket installation failure cancels earlier reservations.
32MiB per batch plus object/queue capacities bound retained data; payloads are never logged.
Mutex-protected process state and owned copies avoid guest aliasing and async guest-pointer access.

## Mutex migration correction

M37 exact calls:MutexattrInit0x17c6d41f0006dbce; Settype0x88ca7c42913e5cee(type2),
MutexInit0x726a3544862f6bda. Attribute slot0x200800880,output slot0x200800878.
First helper init succeeded; synth/synthClientBatch/effects reused that output slot and each
returned0x80020016. Kernel create rejected an already-recorded publication address.
Prototype identity is the allocated opaque token, not the output slot. M38 permits new mutexes
from reused publication slots and valid token copies; kind/nonzero/stale/nonreused ID checks and
ownership/waiter semantics remain. Address is diagnostic provenance. Other synchronization kinds
retain their previous address contract pending relevant evidence. Regression reproduces four
recursive mutex creations from one output slot, copies to distinct locations,lock/unlock/destroy,
and stale rejection. No overwrite-destroy or blanket success shortcut.

## Real workload and remaining boundary

One primary_real_elf run used the exact M38 USAGE command, 250 ms execution and 15000 ms
containment bounds. Exit 0/Clean, 16489 us overall / 15283 us native. Six AudioOut2 calls,
one each, returned zero: Initialize, ContextResetParam, ContextQueryMemory, ContextCreate,
UserCreate, PortCreate. Context 1, user 2 and port 3 (parent 1) were created and retired.
Context grain512, depth4, modeled48000Hz/float-stereo; port inherits this configuration.
The user record's config is a placeholder, not an additional audio port or hardware observation.
Context work area0x210042be0, size0x100000. No PCM/output/queue push, audio waits/completions,
volume changes or events occurred. Null-sink timing is synthetically validated only.

All four helper-created recursive mutexes returned zero. The three synth/synthClientBatch/effects
failure messages disappeared; output was only `RezVR Start !!!`. This supports the token-identity
correction. All eight workers were interrupted out of condition waits and joined; no successful
condition wake/timeout is claimed. Audio live objects/pending tickets zero, opened3/closed3.
Heap peak/live1055728 bytes, 88 live allocations before teardown, one4MiB arena.
Host FS restored and GS preserved; no supervisor redirection, zero suspends/resumes;
zero native/runtime reservations and release errors. Retained42 callbacks were not invoked.

Next stop is UnresolvedFunction ordinal586, AudioOut2 GetSpeakerInfo:
NID0x0C89B3D85B7D1368, exact libSceAudioOut2/libSceAudioOut identity.
Captured landing RIP0x7ff6ffa80396 is HOST landing code, not actual guest PC.
RSP0x2008006b8; guest return address0x1003db993 (not claimed as executing RIP).
Arguments [0x200800790,0,0,0xdae67f5050,0xa0,0xa0]. No fault or guarded object.
Initial RIP0x100000070, RSP0x200800fb8, FS0x210000000; execution thread20440.
Source SHA256 before and after:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.

Focused follow-up routed through 07_NIDS/INDEX.md, MASTER_NID_INDEX.md,
by_sysmodule/libSceAudioOut.sprx/INDEX.md and001.md (lines21-27), then
15_FIRMWARE_KNOWLEDGE/INDEX.md, modules/libSceAudioOut.sprx/INDEX.md and009.md
(lines94-169). Firmware4.03 record originates in docs/firmware/403/generated/veneer_records.jsonl
line1102: VA0x203f0, file0x243f0, bounded626 bytes. This corroborates the name and some constants;
it does NOT establish complete argument/output layout. PS5Rust implementation is absent;
catalogue marks implementation Unknown and wiring NOT_ESTABLISHED. No invented speaker
structure/success response was added. This is the explicitly allowed unknown/deep audio ABI
boundary, not an established helper withheld from migration. No external emulator consulted.

Recommended next work: establish GetSpeakerInfo and surrounding speaker-query structure contract
from focused firmware evidence, then continue the media startup wave. M39 has not begun.
The inherited neutral state structures, period policy and unknown attributes remain limitations;
physical playback, event delivery and uncommon formats are unsupported. No second title run.

## Validation

27 added executable tests:16 service,10 guest adapter,one mutex factory regression; existing copied
mutex test refined for token copying with stale/wrong-kind rejection. Coverage includes manual-time
completion, cancellation/close/shutdown, execution deadline versus completion, batch preflight,
capacity and rollback, config forgery, volume, concurrent service use and publication failures.
No raw audio payload logged. Real evidence validates startup/lifetime only, not playback quality.
Full validation results are recorded in validation.md. Local raw logs remain ignored under target.

## Exact migrated identities

All below are implemented and registered; Runtime means called successfully in the single run.
Others are unexercised in real execution, covered only by synthetic tests. No new ABI record is
claimed for incomplete/inherited AudioOut structures. Existing three ABI records are unchanged.

| Function | NID | Runtime |
|---|---|---|
| sceAudioOut2Initialize | 0x836B558852288471 | yes |
| sceAudioOutInit | 0x25F10F5D5C6116A0 | no |
| sceAudioOutOpen | 0x7A436FB13DB6AEC6 | no |
| sceAudioOutClose | 0xB35FFFB84F66045C | no |
| sceAudioOutOutput | 0x40E42D6DE0EAB13E | no |
| sceAudioOutOutputs | 0xC373DD6924D2C061 | no |
| sceAudioOutSetVolume | 0x6FEB8057CF489711 | no |
| sceAudioOutGetPortState | 0x1AB43DB3822B35A4 | no |
| sceAudioOut2ContextResetParam | 0xB7962B8B3B9FA507 | yes |
| sceAudioOut2ContextQueryMemory | 0xA439A67BB0609BA1 | yes |
| sceAudioOut2ContextCreate | 0xD31EA8D555406126 | yes |
| sceAudioOut2ContextDestroy | 0xA27E991FB01BA35D | no |
| sceAudioOut2ContextAdvance | 0x3C4DB31CCA8B487B | no |
| sceAudioOut2ContextPush | 0x68823D8799E58BD5 | no |
| sceAudioOut2UserCreate | 0xC72C1871107B9DB4 | yes |
| sceAudioOut2UserDestroy | 0x21A65727D33BF6EA | no |
| sceAudioOut2PortCreate | 0x24ADB06A664FCF03 | yes |
| sceAudioOut2PortSetAttributes | 0xF174C0AD23F25879 | no |
| sceAudioOut2PortDestroy | 0x71DF91B70F83D71F | no |
| sceAudioOut2PortGetState | 0x81AB4450A1BE11AE | no |
| sceAudioOut2ContextGetQueueLevel | 0x47B774175836AAC5 | no |
| sceAudioOut2GetSpeakerArrayMemorySize | 0x1B560E2832585F66 | no |
| sceAudioOut2GetSystemState | 0x6E404DF8230BC117 | no |
| sceAudioOut2MasteringInit | 0x5C7977F193649DBB | no |
| sceAudioOut2ContextSetAttributes | 0xE1DAB6ADB956960D | no |
