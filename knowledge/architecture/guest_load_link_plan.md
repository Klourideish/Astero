# M26: immutable guest load/link planning

A planned provider is not a runtime binding. A relocation value is not a performed write.

## Ownership and boundary

`load_plan::link::plan(Arc<Ps5IdentityEvidenceReport>, Vec<ProviderInput>, image_bias, limits)`
produces a private immutable GuestLoadPlan or structured resource/source failure. Loader owns all
rules. Core input/load_plan delegates; CLI load-plan acquires only explicitly named inputs and calls
existing M23/M24 machinery. No session, timing worker, host pointer, memory backend, HLE registry or
filesystem provider search is involved. Existing M16-M24 commands remain independent.

M2 synthetic admission remains unchanged: it still refuses dynamic requirements. This capability
reuses its MappingIntent, CopyIntent, AddressRange, VirtualAddress and source tokens, but does not
claim an M2 ValidatedTarget. A plan is diagnostic/planning admission, not guest execution admission.
Bounded M16 headers are reused for segment evidence; M21-M24 reports remain shared and immutable.
No ELF parser, source identity or address type is duplicated.

## Focused evidence

Catalogue-relative references, entered through each README and 00_START_HERE/README.md/catalogue_map.md:

PS5Rust:
- 03_LOADER_LINKER/INDEX.md, provider_authority.md, module_queries_unload.md,
  elf_self_protections.md, relocation_subset.md.
- 03_LOADER_LINKER/source/ps5-core__src__loader__relocations/INDEX.md and focused
  001.md (bounds), 004.md (numeric subset, authority and function/object/direct distinctions),
  005.md checked_relative_value/checked_absolute_value/checked_pc32_value excerpts,
  007.md resolver/TLS and diagnostic stub/scratch comments.
- 05_KERNEL_HLE/source/ps5-hle__src__module/INDEX.md and 001.md: numeric Nid registries,
  separate function/object providers, authority and registration provenance; names are not identities.
- 03_LOADER_LINKER/source/ps5-libs__src__kernel__module_loader/INDEX.md and 004.md resolve_import:
  authoritative system HLE, direct guest, replaceable stand-in, then unresolved fallback tiers.
  The source comment attributes a Unity blocker to main-image and module resolution policy drift.
  That is legacy documentary workload evidence, not a reproduced Astero execution result.
- 07_NIDS/INDEX.md, MASTER_NID_INDEX.md, nid_resolution_mechanism.md; focused numeric hits:
  by_sysmodule/libSceLibcInternal.sprx/012.md (__cxa_finalize),
  by_sysmodule/SOURCE_OWNER_kernel/003.md (LIBC_NEED_FLAG_NID, function name unestablished).
- 15_FIRMWARE_KNOWLEDGE/document_guides/403_MODULES_SYMBOLS.md.md: approved module/symbol
  inventory and unwind export evidence. This does not prove complete provider ABI or bootstrap rules.
  The __cxa_finalize record joins approved firmware veneer_records.jsonl ~4315 to the numeric NID.

Decrypted catalogue:
- 01_IMAGES/PS5Util.elf/README.md, mapping.md, dynamic.md and
  import_relocations/0001.md: separate VA/file offsets, zero fill, RELRO/TLS/module-param headers,
  dependencies, and actual 64/GLOB_DAT/JUMP_SLOT import slots.
- That 68,080-byte catalogue artifact is a different build from both selected local modules.
  Its counts are corroboration of structure, not expected equality with current inputs.

No external emulator repository was needed. No bulk NID implementation/registry was imported.
Legacy fallback scratch, fake bindings, first-loaded duplicate selection, HLE dominance and runtime
patching are deliberately not copied. Library/module numeric IDs are artifact-local, as M24 proved.

## Provider and dependency policy

Providers must be explicitly supplied complete identity reports. Candidate definitions require:
nonzero trusted symbol index; ordinary section designation; global/weak binding; function/object/
notype; default/protected visibility; unique mapped value/size coverage (function entry file-backed
and executable); canonical NID; unambiguous ExportLibrary and own Module declarations.
Invalid provider source ranges or unavailable evidence remain blockers, not fallback providers.

Referenced undefined symbols are mandatory planning requests (R_NONE is excluded). A match requires
canonical numeric NID, exact byte library AND module names, exact raw version bits, eligible consumer
binding/type/visibility and compatible types (NOTYPE is unspecified). Exact version equality is a
conservative experimental policy, not Sony version compatibility. Unknown/conflicting context never
selects a provider. One match selects; none is unresolved; multiple remain ambiguous. Weak unresolved
references are not silently set to zero. Neither names nor catalogue recognition alone resolve.

Context packing and cross-artifact equivalence retain M24's experimental status. IDs are never
compared across artifacts. Candidates retain source-bound names and explicit provider/symbol index;
original reports expose all attributes and provenance. Ordering is input/provider/symbol order.

DT_NEEDED bytes match only a caller's explicit dependency_alias, supplied with --provider-alias.
No extension stripping or filesystem basename inference occurs. Aliases are caller declarations,
not platform identity proof. Duplicate declarations/matches remain visible. No recursive acquisition.
A selected provider or dependency still requires its own placement/dependency/bootstrap load plan:
ProviderRequiresOwnLoadPlan prevents calling an isolated provider declaration a closed load set.
CLI providers have no guessed bias; library addresses are not assigned from host pointers.

HleProviderDeclaration reserves documentary identity (NID, module/library bytes, evidence label and
opaque declaration ID). It is not accepted as a registered implementation in M26. The future HLE
adapter must prove authority/registration and produce address/trampoline intentions explicitly;
loader never calls handlers. Catalogue correlations are separate and always astero_registered=false.

## Segments, entry and actions

Consumer image_bias is explicit, in bytes; mapped intent is bias + p_vaddr, checked. It is not a
host allocation result. Every PT_LOAD retains raw header and source token, file/memory sizes,
alignment and exact R/W/X bits. File size must fit memory, source range must exist, memory extent
must not overflow. ELF offset/VA alignment congruence and bias alignment are checked. Empty memory,
unknown permission bits and overlapping virtual intervals block. File tails are explicit zero-fill.
No protection is actually changed. Host page-union/staging policy remains the future memory adapter's
responsibility; unresolved RELRO is explicitly blocking rather than silently retaining writable pages.

Nonzero ELF entry is checked in file-backed executable bytes. Missing entry stays absent/blocking,
not a guessed bootstrap. Nonzero DT_INIT/DT_FINI/array/preinit pointers remain raw Bootstrap blockers.
Nonempty TLS, RELRO, SCE process/module parameters and unsupported program kinds remain explicit blockers;
all raw headers are retained. Empty TLS (both sizes zero) needs no storage and is not a blocker. PT_NULL/DYNAMIC/NOTE/PHDR/EH_FRAME/STACK/DYNLIBDATA do not themselves
establish execution support. PS5/platform observations keep the plan experimental even without blockers.

Relocations reuse M9/M23 records and source provenance. Supported action categories:
- NONE (0): no write.
- 64 (1): checked S + A, 8 bytes.
- PC32 (2): S + A - P in i128, require signed i32; retain its 32-bit representation.
- GLOB_DAT (6), JUMP_SLOT (7): S, 8 bytes; addend retained raw but not added.
- RELATIVE (8): checked B + signed A, 8 bytes, require null symbol index.

B is explicit bias, S is local defined/absolute value or selected provider value plus explicitly
supplied provider bias, P is checked relocation place. The arithmetic helpers are legacy corroborated;
GLOB_DAT/JUMP_SLOT S-only behavior uses ordinary x86-64 ELF interpretation, not new PS5 ABI proof.
Prospective stores are little-endian; width and bit pattern are retained, never written.
TLS symbol values are not treated as ordinary image addresses. No TLS module ID is invented: DTPMOD64 and all other types remain numeric Unsupported.
Non-relative null-symbol actions conservatively lack symbol-address evidence. Local symbol extent
must lie in an intended segment. Every write-sized target must fit a segment; overlapping writes
block. Concrete values are optional: unresolved references retain action/provenance without values.
A relocation links to a reference-plan index, avoiding duplicated ambiguity vectors.

## Readiness, resources and diagnostics

ReadyForLoad means the supported planning subset has no blockers and no experimental platform
assumptions. It does not certify host allocation, guest ABI, executable safety or runtime admission.
Experimental means no blockers but interpretation assumptions remain. Blocked lists every retained
missing dependency/reference, unsupported semantic, bad segment/entry or relocation condition.
Missing lower evidence preserves segment/header evidence and shared failed prerequisites.
Budget/allocation/source failures return no successful prefix. No mutable getter exists.

Existing acquisition, header, dynamic, hash, symbol/name, relocation and identity limits apply per
artifact. New max_providers bounds supplied artifacts before CLI acquisition. max_plan_records is a
shared record-construction budget (including temporary match indices): segments, candidates, dependencies, references,
relocations and blockers each cost one. Zero refuses the first needed record; exact succeeds.
Vector growth is fallible. Caller-supplied provider/alias storage and CLI argument bytes are already
caller-owned. Work is bounded by input counts, including pairwise matching and overlap checks; this
is not a linear-time guarantee. No hidden filesystem traversal or timing worker is introduced.
CLI prints a named 16-record detail limit for reference/relocation/blocker lists, explicit omissions,
and exact totals; the immutable plan keeps every retained record. Exit 1 indicates Blocked/refusal,
not that no useful evidence was produced. No loader state is mutated.

## Real experiments and outcomes

All inputs selected via ignored LOCAL_TEST_CORPUS.json, read as data only. Acquisition limit 16 MiB,
512 reads; 128 headers, 1024 dynamic entries, 262144 hash words, 2 descriptors, 16384 symbols,
32768 name lookups, 256 bytes/name, 8388608 total name bytes, 131072 relocations, 32768 identity
records. Plan limits: two providers, 524288 records; consumer bias 0x100000000.

| Result | linkage_sample | utility_build_comparison | primary_real_elf |
|---|---:|---:|---:|
| Bytes | 67668 | 67628 | 9225340 |
| Segments | 5 | 5 | 5 |
| Raw entry | 0 | 0 | 0x70 |
| Dependencies | 2 | 2 | 38 |
| Referenced undefined symbols | 8 | 8 | 822 |
| Encoded NIDs | 13 | 13 | 822 |
| Supplied provider candidates | 5 (alternate utility) | 0 | 0 |
| Selected / unresolved | 0 / 8 | 0 / 8 | 0 / 822 |
| Relocations / supported categories | 11 / 11 | 11 / 11 | 30898 / 30898 |
| Concrete values | 2 | 2 | 29791 |
| Unsupported relocation types | none | none | none |
| Readiness / exit | Blocked / 1 | Blocked / 1 | Blocked / 1 |

Both utilities declare libkernel.prx/libc.prx. Their own five eligible definitions do not satisfy
those eight referenced external identities. This rejects the hypothesis that an alternate build is
useful as a libc/kernel provider merely because it shares many NIDs overall. Source identities and
symbol ordering remain distinct. No artifact substitution was required. Executable entry intent is
0x100000070; no jump or thread creation occurs. All original input SHA256 hashes match after runs.
Utility hashes are in M24's identity record; detailed current logs/hashes remain under ignored
 target/m26-validation. No absolute machine path is recorded in tracked files.

Blocked causes in these runs: missing supplied dependencies/providers, RELRO/SCE/bootstrap requirements;
utilities also have no ELF entry. Unknown program values are retained numerically. All observed
relocation types fit the small action subset, but unresolved S prevents most symbolic values.
Catalogue-known __cxa_finalize (0x1F67BCB7949C4067) stays unresolved; the second curated NID
0x3F7DF43F774517AF has only legacy LIBC_NEED_FLAG_NID evidence, not a proven function name.
Both existing NID routing records remain unregistered. No ABI or HLE registration was established.

Synthetic experiments support exact matching, conflict refusal, version distinction, overflow
checks and closed-subset readiness. Cross-artifact name/version matching remains experimental as a
platform policy. Bootstrap/TLS/RELRO and complete provider load closure remain unresolved, not guessed.

## Handoff

M27 should consume only appropriately admitted plans: bounded memory staging/copy/zero-fill and
final protection, with explicit refusal of Blocked plans. Provider closure, TLS, RELRO/bootstrap
and experimental identity policy must be resolved deliberately before running these real artifacts.
The plan contains actionable segment and relocation evidence now; it is not a claim that the three
real inputs can execute next. No memory mapping, relocation application, HLE binding, timing worker
or guest execution was added. See validation.md for actual checks and USAGE.md for repeatable runs.
