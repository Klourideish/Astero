# M24: PS5 identity evidence

An encoded NID is identity evidence, not a resolved function. A declared library is not a provider.

## Boundary and ownership

`identity::observe(Arc<LinkageReport>, IdentityLimits)` explicitly consumes a complete M23 report.
Loader `elf/dynamic/identity` owns codec, descriptor hypotheses, correlation and immutable results.
Core `input/ps5_identity` delegates; CLI `ps5-identity` explicitly composes acquisition, M23 and M24.
No prior command calls M24. No GUI, dependency edge or external dependency changes.
The report shares its complete input, original symbol/name/range tokens, dependencies and relocation
associations. It never re-enumerates symbols/relocations, mutates input, opens dependencies, constructs
a session, resolves providers, binds HLE, applies relocations or executes a guest.

## Focused evidence consulted

Paths below are relative to the catalogues routed by local AGENTS.md, not tracked machine paths.
Both were entered through README and 00_START_HERE/README.md.

PS5Rust catalogue:
- `07_NIDS/INDEX.md` and `07_NIDS/nid_resolution_mechanism.md`.
- `03_LOADER_LINKER/source/ps5-core__src__loader__nid/INDEX.md` and `001.md`:
  custom alphabet, 11-character numeric decoding and distinct salted SHA-1 name hashing.
- `03_LOADER_LINKER/provider_authority.md`, `relocation_subset.md` and focused loader/elf records:
  legacy stand-ins/unresolved binding are not source identity or provider proof.
- `15_FIRMWARE_KNOWLEDGE/INDEX.md` and
  `document_guides/403_MODULES_SYMBOLS.md.md`: approved documentary firmware example
  `RpQJJVKTiFM` = `0x4694092552938853` (associated there with sceKernelGetModuleInfoForUnwind).
  This is corroboration of the codec vector, not firmware re-execution or an Astero ABI contract.

Decrypted catalogue:
- `01_IMAGES/INDEX.md`, `01_IMAGES/PS5Util.elf/README.md`, `mapping.md`, `dynamic.md`,
  `imports/INDEX.md`, `imports/0001.md`, `import_relocations/INDEX.md` and `import_relocations/0001.md`.
- Eight local undefined-name prefixes match catalogue numeric examples. The catalogue's 68,080-byte
  image has 18 symbols/13 relocations and is not either local build. Its dynamic record explicitly
  does not establish packed version/ID semantics.

No external emulator source, raw firmware payload or bulk NID database was needed/copied.

## Codec and confidence

The alphabet is `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-`.
An 11-byte token contains 66 bits; two low padding bits must be zero. Decode shifts those bits away;
encode is its fixed-width inverse. Noncanonical padding is a candidate/error, unlike legacy's alias
acceptance. Known decrypted and approved firmware vectors plus bounded roundtrips test this choice.
This is NOT plain-name hashing. Legacy separately hashes name bytes plus its suffix using SHA-1
and interprets the first eight digest bytes little-endian. M24 neither needs nor implements that path.

Only `canonical-NID#canonical-library-ID#canonical-module-ID` is an encoded symbol correlation.
IDs use a shortest 1-3-character base-64 integer bounded to u16 under the suffix hypothesis.
Bare 11-character strings remain candidates; arbitrary strings/non-UTF-8 remain raw/plain evidence.
Malformed names, suffixes and padding retain original bytes without claiming a numeric identity.
`Encoded` proves canonical numeric decoding under the documented format, not the function's meaning.
Context is independently Absent/Missing/Hypothesis/Conflict. No name-spelling heuristics exist.

## Descriptor hypothesis and conflict rules

Supported experimental interpretations:

| Tag | Hypothesized role | Fields |
|---|---|---|
| 0x61000043 | own Module | high16 ID, middle16 version bits, low32 string offset |
| 0x61000045 | NeededModule | same |
| 0x61000047 | ExportLibrary declaration | same |
| 0x61000049 | ImportLibrary declaration | same |

`MetadataEvidence::ExperimentalPacking` is retained in every interpreted record. Version bits stay
raw; no major/minor ABI interpretation is claimed. Name offsets use existing translated DT_STRTAB
and M6 bounded NUL lookup. Whole table/source bounds remain authoritative. A malformed name fails
with its dynamic index and structured string error; there is no invented fallback name.
Other tags in 0x61000000..0x6100ffff retain index/tag/raw value as Uninterpreted. Other unknown tags
remain accessible in the shared M23 raw evidence.

Correlation compares suffix IDs within the same source and module/library namespace. Identical
raw duplicate declarations retain their order and count; differing raw descriptors/kinds give
Conflict, never a selected provider. The first index is only a representative of identical duplicates.
Absent IDs stay Missing; no cross-artifact ID inference. Relocation/candidate evidence is available
through the shared M23 report but is not strengthened. ExportLibrary is a descriptor hypothesis,
not a promotion of defined symbols to runtime exports. DT_NEEDED names remain independent exact
byte declarations; removing `.prx` or assuming their association would add an unjustified rule.

## Budgets and status

All existing M23 limits remain mandatory. One added `max_identity_records` counts every retained
SCE dynamic record (including duplicates/unsupported tags) plus every symbol (including null/raw).
Count is established from bounded input before collection. Zero refuses nonempty evidence; exact
count succeeds; exhaustion returns Failed without a successful prefix. Fallible vector reservation
reports allocation refusal. Correlation work is bounded by the product of these bounded record counts.
Existing name lookup/per-scan/total-scan budgets are shared across symbol, DT_NEEDED and metadata
names, in that order; repeated names/terminators consume work. No hidden reset for metadata scans.
Existing bounded discovery and M5 descriptor translation are reused to obtain the string-table view;
no parallel ELF parser or new SCE-relative-address adapter is introduced.

Complete means all requested records were retained, including candidates/conflicts/unsupported data.
It does not mean all identities are known. Unavailable propagates unavailable M23 input. Failed
retains the input report and structured prerequisite/budget/discovery/dynamic/string/allocation error.
Sources with no PS5 records can complete with only plain/unnamed symbols; they are not declared PS5.
Source identity and native-path provenance remain inherited from the input, never reconstructed.

## Real experiment and outcome

Both ignored corpus roles were inspected read-only with 1 MiB acquisition, 64 reads/headers,
256 dynamic entries, 4096 hash words, 2 generic descriptors, 1024 symbols/name lookups/relocations,
256 bytes per name, 262144 aggregate name bytes and 1024 identity records. Both exit 0.

| Observation | linkage_sample | utility_build_comparison |
|---|---|---|
| Bytes | 67,668 | 67,628 |
| SHA256 | 2f824c2a233c540d7220a4b1bc3b9c76e1a314267e47fe31740db5b93f582a3f | daa15b69a637b29a34f93d5e9dd28c112e15f82f18ca12a9858b2eab55acef0b |
| Symbols / relocations | 14 / 11 | 14 / 11 |
| Encoded NIDs / unconfirmed | 13 / 0 | 13 / 0 |
| Supported / unsupported SCE records | 7 / 7 | 7 / 7 |
| Own module / ID / version bits | PS5Util / 0 / 0x0101 | same |
| Needed modules | libkernel=1, libc=2 (0x0101) | same |
| Library IDs (version bits=1) | libkernel_unity=0, libkernel=1, libc=2, PS5Util=3 | libkernel=0, libc=1, libkernel_unity=2, PS5Util=3 |
| DT_NEEDED | libkernel.prx, libc.prx | same |

All 13 numeric NIDs overlap; symbol order and eight dependency-side suffixes change. Five PS5Util
names retain `#D#A`. All 26 library/module suffix pairs find a declaration in their own artifact;
there are no missing/conflicting matches. Unsupported records: 0x61000019 (three), 0x61000041,
0x61000017, 0x6100003f, 0x6100003d. Source hashes before/after match exactly. No third input was needed.

Hypothesis outcomes:
- Canonical codec: supported by independent known vectors and all real encoded names.
- Low32 string offsets/high16 local IDs: supported on both inputs, still explicitly experimental
  as a general PS5 contract. Version meaning beyond raw16 remains unresolved.
- Stable cross-build library IDs: rejected. Artifact-local suffix correlation remains consistent.
- Same numeric NIDs across these two builds: supported for these 13 entries, not universal identity.
- Provider/function meaning, attribute tags, alternate layouts and dependency-name correspondence:
  unresolved/deferred. No heuristic silently promotes them.

The NID routing index includes two curated real observed/correlated vectors, with decoder source
ownership and unregistered status; it is not an implemented-HLE inventory. Remaining observed NIDs
are in the report, not bulk catalogue entries. ABI records remain zero.

## Validation and next pressure

Fourteen loader tests and two CLI tests cover codec provenance, aliases, raw names, descriptor
bounds, duplicate/conflicting/missing context, shared scan budgets, fail-before-prefix, repeated
immutable observation and native paths. Existing M16-M23 and synthetic session paths remain intact.
See validation.md for full counts and USAGE.md for repeatable commands. Local detailed logs are
under ignored target/m24-validation; no real artifact or absolute corpus path is tracked.

Recommended M25: a bounded, explicit provider-candidate comparison across deliberately supplied
module evidence, with uncertainty and ambiguous matches preserved, before HLE binding or relocation
application. First corroborate packed version/attribute semantics on a distinct corpus sample;
no global module registry or final runtime binding is established by M24.
