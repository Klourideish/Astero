# M47 filesystem startup and VM wrapper continuation

Implementation complete; final validation in progress. No M48 scope is planned.

## Evidence and adaptation

PS5Rust catalogue routes consulted: `00_START_HERE/README.md`,
`12_FILESYSTEM_IO/INDEX.md`, `12_FILESYSTEM_IO/stdio_writes.md`,
`12_FILESYSTEM_IO/source/ps5-core__src__filesystem/001.md`, and
`12_FILESYSTEM_IO/source/ps5-libs__src__kernel__file/INDEX.md`.
Current read-only prototype `crates/ps5-libs/src/kernel/file.rs` supplies exact
NIDs, corrected 120-byte stat encoding, descriptor/stdio contracts, 64 KiB writes,
EOF/error separation, and App0Mount/detect_app0_mount/resolve_in policy.
`crates/ps5-libs/src/kernel/system.rs` supplies corrected sceKernelMunmap identity,
whole-view refusal tests, and two explicit Mprotect identities. M32/M34 runtime,
M44 observability, M45 resource and M46 request/stats owners remain authoritative.
No external emulator implementation was consulted afresh; prototype stat comments
attribute prior corroboration to Kyty/SharpEmu, which is inherited evidence only.

## Ownership and bounds

Kernel filesystem owns synchronized descriptor records and mount resolution; libs
owns exact guest adapters. Core injects mounts before execution. CLI accepts
optional --title-root and forwards the source path; it owns no filesystem state.
No new dependency or unsafe code. Descriptors monotonically increase, never expose
host handles and are not reused within a runtime. Operations lease a descriptor
record; close/shutdown mark it closed and release the file under its lock.
The 1024-entry limit includes three standard descriptors. Stdin is an empty source;
stdout/stderr append to the existing bounded process output, never the terminal.

FILE values are 16-byte guest heap tokens, not host FILE pointers or firmware FILE
layouts. Stream identity is checked against the owner table. Tokens are retained
until heap teardown to avoid reuse ambiguity; close releases host file ownership.
This costs bounded guest heap storage over repeated opens. No host FILE layout is
claimed. Basic stdio is unbuffered; prototype read-ahead performance optimization
is not yet adapted. EOF and error indicators remain separate. Positioned I/O
holds the descriptor lock and restores shared offset; positioned append writes
are explicitly unsupported. Full guest ranges are preflighted; copies use 64 KiB
scratch with the existing 64 MiB operation budget. OS failure after progress can
leave partial I/O; no transaction/rollback or atomic-write claim is made.

## Mount and metadata policy

Explicit title root wins. Otherwise executable parent is code root; only when
it and its immediate parent have sce_sys/sce_module structure does the parent
become data root. Code precedes data. Relative guest paths resolve under app0.
App0 is read-only. Data writes require an explicitly configured writable root;
CLI currently configures only app0. Unconfigured system/savedata roots refuse.
Drive prefixes, alternate streams, backslashes, reserved DOS devices, trailing
Windows-normalized dots/spaces and escaping parents refuse. Canonical containment
checks links before open. This is not a hostile filesystem sandbox: external
concurrent reparse-point changes are not isolated. Windows case behavior remains
a limitation; exact PS5 case sensitivity is not claimed.

Stat emits exactly 120 bytes with explicit fields: mode8/u16, nlink10/u16,
size72/u64, blocks80/u64, blocksize88/u32. Other bytes zero; regular-file512-byte block accounting and prototype directory
128-block/65536-byte block-size values are compatibility policy, not hardware/filesystem truth.
Directory metadata is available; directory enumeration and AIO remain unsupported.
No synchronous operation is called asynchronous AIO.

## VM

sceKernelMunmap reuses M45 whole-view unmap. Invalid/partial views fail without
mutation; removing a VA view does not release its direct backing. Two Mprotect
identities reuse the same protection owner. No second VM implementation or fake
partial success. Existing maps/query/reservation wrappers remain unchanged.

## Observability

Additive schema-one filesystem fields retain opens/closes/current streams/peak
entries/read/write bytes/seeks/stats/failures and bounded last guest path. Paths
reported to users are guest paths, not private host paths. Existing output capture
feeds trace/log details. Final resources are closed after guest workers join.

## Real runs

Exact local commands: `& ./target/m47-primary-1.ps1` and
`& ./target/m47-second-1.ps1`. Both retain M46 corpus roles, placement/acquisition
limits, 250 ms wall bound, 15000 ms containment and 65536 HLE-call budget.
No PC sampling or interactive terminal revalidation was performed. Both used
plain output, JSON fingerprints and captured diagnostic logs.

Primary: sceKernelMunmap returned0 for the 6 MiB view; subsequent direct backing
release returned0. One allocation/map/unmap, no remaining backing/views at stop.
No filesystem activity. New unresolved libc/libc NID0x5CA45E82C1691299, ordinal35,
arguments [0x81000103,0x81000103,0,host-value,0xd0,0xe0]. Inherited comparative
record `docs/targeted information/registry/comparative.json` identifies
`src/libs/libC.cpp:814`, `LibC::catchReturnFromMain`, encoded XKRegsFpEpk.
This is a comparative label, not independent symbol/semantic confirmation.
Boundary RIP0x7ff6e1db74d6, RSP0x200800f88; separate guest continuation0x1000000b8.
Overall207.991ms, supervised205.738ms;11015 HLE calls,64 unique providers,
eight workers joined. No fault or supervisor redirection.

Second: fopen('/app0/Media/boot.config') succeeded from the single-root code/data
mount. One fread read420bytes, two fseek calls, one ftell (three seek mechanism
operations), one fclose. Zero writes/stat calls/failures, peak4 descriptors
including stdio, zero remaining file/stream resources. No host paths are embedded.
Next unresolved __getpctype, NID0xB143F58416A8B8EC, libc/libc, ordinal1339;
identity from `crates/ps5-libs/src/kernel/libc/nids.rs:658`.
Boundary RIP0x7ff6e1db74d6, RSP0x2008009a8; guest continuation0x100be1148.
Overall40.581ms, supervised39.891ms;1569 HLE calls,39 unique providers, no workers.
Nine semaphores and one1MiB direct allocation/view were retained at stop and
released on teardown. No real semaphore wait/signal was observed.

Boundary RIPs are host RuntimeBridge landings, not sampled executing guest PCs.
Both stopped structurally at unresolved functions, not game boots. Threads joined,
FS restored, GS preserved, zero native reservations/release errors/waiters/tickets,
clean child containment. Source SHA256 before/after unchanged:
- primary: A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A
- second: 3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327

The final additive unmap counter was synthetically verified after these runs;
the saved real trace proves successful unmap but predates that JSON field.

## Findings and limitations

Promoted: whole-view wrapper progression; real checked fopen/read/seek/close;
source integrity and cleanup. Overlay precedence/explicit-root behavior has
synthetic proof; both real inputs used single roots, so no real split-overlay
claim. Case sensitivity, incomplete stat fields, read buffering, writable mounts,
partial unmap and AIO remain explicitly limited. Synchronous regular-file I/O
uses bounded chunks but has no independent in-process disk-stall cancellation;
outer containment remains the emergency boundary, not clean recovery from an
arbitrarily stalled host filesystem. No network/device files are intentionally
admitted. Historical Windows487 remains unresolved/unreproduced.
No M48 implementation scope is proposed.

## Registered surface

| Export | NID | Exact library/module |
| --- | --- | --- |
| sceKernelOpen | `0xD46DE51751A0D64F` | libkernel/libkernel |
| sceKernelClose | `0x50AD939760D6527B` | libkernel/libkernel |
| sceKernelRead | `0x0A0E2CAD9E9329B5` | libkernel/libkernel |
| sceKernelWrite | `0xE304B37BDD8184B2` | libkernel/libkernel |
| sceKernelPread | `0xFABDEB305C08B55E` | libkernel/libkernel |
| sceKernelPwrite | `0x9CA5A2FCDD87055E` | libkernel/libkernel |
| sceKernelLseek | `0xA226FBE85FF5D9F9` | libkernel/libkernel |
| sceKernelStat | `0x795F70003DAB8880` | libkernel/libkernel |
| sceKernelFstat | `0x901C023EC617FE6E` | libkernel/libkernel |
| sceKernelFtruncate | `0x556DD355988CE3F1` | libkernel/libkernel |
| open | `0xC2E0ABA081A3B768` | libkernel/libkernel |
| close | `0x6D8FCF3BA261CE14` | libkernel/libkernel |
| read | `0x02A062A02DAF1772` | libkernel/libkernel |
| write | `0x14DE2068F9AE155F` | libkernel/libkernel |
| pread | `0x7B3BFF45204D2AA2` | libkernel/libkernel |
| pwrite | `0xB6909FDBC92E6B3` | libkernel/libkernel |
| lseek | `0x3B2E88A7082D60E9` | libkernel/libkernel |
| ftruncate | `0x8A1E020FDFE08213` | libkernel/libkernel |
| fopen | `0xC5E60EE2EEEEC89D` | libc/libc |
| fclose | `0xBA874B632522A76D` | libc/libc |
| fread | `0x95B07E52566A546D` | libc/libc |
| fwrite | `0x329C61321F1016BA` | libc/libc |
| fseek | `0xAD0155057A7F0B18` | libc/libc |
| ftell | `0x41ACF2F0B9974EFC` | libc/libc |
| rewind | `0xDD020F221FC60E3C` | libc/libc |
| feof | `0x2F170453E202BBC5` | libc/libc |
| ferror | `0x007C7284DF7A772E` | libc/libc |
| fflush | `0x3148C2E256C7ACAE` | libc/libc |
| fileno | `0x166FDD9B2CB01FD4` | libc/libc |
| fstat | `0x9AA40C875CCF3D3F` | libc/libc |
| stat | `0x13A6A8DF8C0FC3E5` | libc/libc |
| fgetc | `0x004B85DC5D9FF130` | libc/libc |
| fputc | `0x6992BC94D7A2FD0C` | libc/libc |
| fgets | `0x29D3FF9D42E9B86C` | libc/libc |
| fputs | `0x42B659749F17B17D` | libc/libc |

VM additions: sceKernelMunmap0x71091EF54B8140E9 and sceKernelMprotect0x03A66B3A6F21CB46/0xBD23009B77316136, all libkernel/libkernel.
Only fopen/fread/fseek/ftell/fclose and Munmap were exercised successfully in M47.
Other registrations remain synthetically validated, not runtime-confirmed.
Total409 registered/4 observation-only NIDs;8 ABI records. No FILE-layout ABI
record is asserted for an emulator-owned opaque token.
