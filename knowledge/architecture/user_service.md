# M36 UserService startup foundation

## Evidence and ownership

PS5Rust catalogue: 00_START_HERE/README.md and catalogue_map.md ->
13_NETWORK_PLATFORM/INDEX.md -> source/ps5-libs__src__platform/INDEX.md, 015.md, 016.md,
021.md. Focused original crates/ps5-libs/src/platform.rs lines 174-191, 2397-2679 and
3962-4075 establish 15 identities, single user, defaults, initial login/logout queue and tests.
Firmware route: 15_FIRMWARE_KNOWLEDGE/INDEX.md -> modules/libSceUserService.sprx/INDEX.md;
001.md GetEvent (8-byte store, 0x80960002/05/07), 002.md Initialize (0x80960005, 700),
022.md GetUserName (0x400 forwarding constant). These bounded veneers corroborate identity and
selected constants, not a complete UserService ABI or a real account backend.
Decrypted route: 00_START_HERE/{README,catalogue_map}.md -> 01_IMAGES/INDEX.md ->
Il2cppUserAssemblies.elf/README.md and PSNCommon.prx/README.md; 05_NIDS/INDEX.md ->
right.elf/0001.md and focused Il2cppUserAssemblies.elf, PS5Util.elf, PSN.prx index searches.
Those selections did not independently identify this workload's UserService import. Do not
claim cross-artifact equivalence. Current M35 exact real import is authoritative workload evidence.
No external emulator or raw firmware was consulted or changed.

New libs/user_service directory is the narrow guest family owner; families is retained navigation,
not a catch-all platform implementation. Kernel/process/users owns the bounded mechanism. Core
Foundation retains Arc<Mutex<Users>>, shared by main/worker registries. CLI prints an immutable
snapshot. No new crate edge, timers, global registries, filesystem or account services.

## Migrated policy and deliberate refinements

All 15 PS5Rust UserService exports in user_service/exports::EXPORTS are registered with exact
libSceUserService/libSceUserService keys, not NID fallback. Experimental user ID 0x10000000,
name Player, initial login, default settings are prototype policy, not firmware security/account facts.
Initialize is idempotent, queues one login only on first initialization. Terminate is idempotent,
clears login/queue; reinitialize begins another generation. Initial user remains stable after logout;
login enumeration then returns four -1 IDs. No host account data is read.

Kernel fixed two-event storage cannot grow: a generation admits one login and at most one logout;
repeated logout adds nothing. Full queue refuses before state change. Poll copies first, consumes
only after success while locked. Polling old login never reactivates a logged-out user (prototype
bug deliberately rejected). No guest callbacks are installed or invoked; prototype poll API suffices.

Little-endian outputs: user/settings i32; login list four i32s; event {i32 type, i32 user}; type0 login,
type1 logout. Name is seven bytes including NUL. Caller capacity <7 refuses as AccessError::Limit,
not truncated success; zero is not guessed as capacity17. Game presets require readable u64 capacity
>=40, write exactly40 bytes, prefix40 and zero default payload. Prototype short-prefix overwrite and
unreadable-size fallback are rejected. These layouts are migrated prototype contracts, not new
firmware-proven ABI records. Profile/account/parental control operations are not implemented.

Settings retain age18, vibration/trigger1, transcription/hold-delay/zoom/follow-focus0 as simplified
prototype defaults. Init's optional pointer is checked as a four-byte priority, accepted0..1023
(experimental bounded adapter policy); null selects default. Priority has no scheduling effect.
No options extensions are claimed. Firmware 700 constant motivates retaining the ordinary priority
path but does not prove complete range semantics. Name veneer 0x400 does not prove optional-capacity
semantics; current explicit third-argument contract is the bounded prototype adaptation.

All outputs use M35 charge/full validation/copy; gaps, RO, overflow and bad tails refuse. Later OS
failure remains structured/nontransactional; queued event is retained for retry. Mutex covers only
small <=40-byte service operations, never waits/native guest callback execution. Runtime drop releases
state; tests verify independent runtimes and final Arc reclamation. No new shutdown worker exists.

Errors: firmware-correlated 0x80960002 not-initialized interpretation, 0x80960005 invalid-argument
(including invalid user as prototype), 0x80960007 no event. Buffer/OS/policy failures stay typed HLE
AccessFailure instead of inventing an unproven SCE buffer code. No errno mutation. Unknown calls stop.
Repeated initialize success is deliberate prototype policy, not claimed firmware idempotence proof.

Runtime registration capacity is explicitly 256+15=271: first full regression exposed failure with
256; worker wait test then passed with the full cluster admitted. HLE attempts remain4096, timing and
supervisor unchanged. Unrelated priority ENOTSUP remains. No extra service migrated preemptively.

## Validation and runtime

Focused 15 tests cover lifecycle/identity/outputs/events/concurrency and exact dispatch. Full preflight passed before the single real run. M37 is not started.


## Real workload observation

One primary_real_elf run, exact unchanged M35 first-entry command reproduced in USAGE M36;
wall250ms/containment15000ms. Exit0, Clean containment. Elapsed17,851us; native supervision16,206us,
thread10872, no redirection, suspends0/resumes0. Initialize called once at option pointer0x200800c88,
returned0. Snapshot initialized=true, logged_in=true, user0x10000000, pending_events1,generation1.
No initial-user/list/name/settings/event/terminate calls were observed. No event was polled or guest
callback invoked. Only Initialize is runtime-confirmed; other14 exports are synthetically tested.
294 main provider calls and16 worker calls; eight workers joined after eight condition waits were
interrupted, zero signal wakes/timeouts. Synchronization created12/destroyed1 (11 remaining objects
before owner teardown), no waiters afterward. Heap peak7104, 87 live allocations before teardown.
Host FS restored/GS preserved for main/workers; reservations0/releaseerrors[].

Final stop UnresolvedFunction, NID0x43657E8AABE3802D (vsnprintf, prototype
crates/ps5-libs/src/kernel/libc/nids.rs VSNPRINTF_NID), libc/libc, ordinal328. Boundary RIP0x7ff604303326
is host import landing, NOT executing guest-image PC. Separate guest return address0x10033a037;
RSP0x200800778. Arguments [0x100fc7940,0x200,0x10061e2db,0x200800830,0xa0,0xa0]. No exception/fault.
Formatting/va_list is a different libc contract, not a user-state helper; no implementation added.
Recommended M37: coherent bounded libc formatting/varargs migration, informed by this actual call.

Source SHA256 before/after unchanged:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Ignored raw evidence target/m36-real-1.log. No repeat run chasing the outside-cluster boundary.
Supported: actual UserService initialization and worker coexistence. Refined: registration capacity
needed to include all15 keys (271 bound); rejected prototype event loss/relogin and output truncation.
Unknown: option priority policy, fuller name ABI, multi-user/account semantics and callbacks remain
explicit. Native entry did not prove those uncalled contracts. ABI remains2; no new full-ABI claim.

## Exact migrated identity list

All below registered under libSceUserService/libSceUserService; only Initialize observed in M36.

| Export | Numeric NID |
|---|---|
| sceUserServiceInitialize | 0x8F760CBB531534DA |
| sceUserServiceGetInitialUser | 0x09D5A9D281D61ABD |
| sceUserServiceGetAgeLevel | 0xC28369BBEE3944B9 |
| sceUserServiceGetGamePresets | 0xFEC0F4DA6143061E |
| sceUserServiceGetAccessibilityChatTranscription | 0xAE71211EA1BFE31A |
| sceUserServiceGetAccessibilityPressAndHoldDelay | 0x64A26DC5D82FCF08 |
| sceUserServiceGetAccessibilityVibration | 0xA96607385C2A0B16 |
| sceUserServiceGetAccessibilityTriggerEffect | 0xFF763918EFBF8BBF |
| sceUserServiceGetAccessibilityZoomEnabled | 0x843FC7F3510DF558 |
| sceUserServiceGetAccessibilityZoomFollowFocus | 0x3BA216D7F0F09BFC |
| sceUserServiceGetLoginUserIdList | 0x7CF87298A36F2BF0 |
| sceUserServiceGetUserName | 0xD71C5C3221AED9FA |
| sceUserServiceGetEvent | 0xC87D7B43A356B558 |
| sceUserServiceLogout | 0xDD3F72E710DC7CE9 |
| sceUserServiceTerminate | 0x6F01634BE6D7F660 |

Final validation: 489 executable Rust tests,23 doctests,53 Python tests passed; format/check/build/
warnings-denied Clippy and policy/state/structure passed. Index records3419: implementation1343,
subsystems138,modules564,sources549,tests567,diagnostics40,NIDs216,ABI2. Registered213, unregistered3.
All17 index files regenerate byte-identically; whitespace/freshness checks passed. See validation.md.
