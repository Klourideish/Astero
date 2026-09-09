use astero_core::input::{
    load_plan::*,
    ps5_identity::{IdentityOutcome, NameEvidence},
};
use std::fmt::Write;
pub fn render(p: &GuestLoadPlan) -> String {
    let mut out = format!(
        "PLAN ONLY\nArtifact: {:?}; bytes={}\nAcquisition succeeded; explicit load/link planning requested\nImage bias: {:#x}; entry: {:?}\nLimits: {:?}\n",
        p.input().linkage().source().provenance(),
        p.input().linkage().source().len(),
        p.image_bias().0,
        p.entry(),
        p.limits()
    );
    let mut nids = 0;
    if let IdentityOutcome::Complete(e) = p.input().outcome() {
        nids = e
            .symbols()
            .iter()
            .filter(|s| matches!(s.evidence, NameEvidence::Encoded { .. }))
            .count();
        for d in e.descriptors().iter().filter(|d| d.name.is_some()) {
            let bytes = p
                .input()
                .linkage()
                .source()
                .read(&d.name.unwrap())
                .expect("source-bound metadata");
            writeln!(
                out,
                "Identity {:?}: bytes={bytes:02X?}; id={:?}; version={:?}; experimental",
                d.kind, d.id, d.version_bits
            )
            .unwrap();
        }
    }
    writeln!(
        out,
        "Segments: {}; dependencies: {}; external references: {}; encoded NIDs: {}",
        p.segments().len(),
        p.dependencies().len(),
        p.references().len(),
        nids
    )
    .unwrap();
    for s in p.segments() {
        writeln!(
            out,
            "Segment[{}] {:?}; file bytes={}; zero-fill={:?}; permissions={:?}",
            s.program_index,
            s.mapping.range,
            s.raw.file_size,
            s.mapping.zero_fill,
            s.mapping.permissions
        )
        .unwrap();
    }
    for (i, d) in p.dependencies().iter().enumerate() {
        writeln!(
            out,
            "Dependency[{i}] bytes={:02X?}; explicit providers={:?}",
            p.input()
                .linkage()
                .source()
                .read(&d.name)
                .expect("source proof"),
            d.supplied
        )
        .unwrap();
    }
    let selected = p
        .references()
        .iter()
        .filter(|r| matches!(r.resolution, PlannedResolution::Selected(_)))
        .count();
    writeln!(
        out,
        "Provider candidates: {}; planned resolutions: {}; unresolved/ambiguous: {}",
        p.candidates().len(),
        selected,
        p.references().len() - selected
    )
    .unwrap();
    for r in p.references().iter().take(16) {
        if let Some(c) = r.nid.and_then(catalogue_correlation) {
            writeln!(
                out,
                "Catalogue: {}; Astero registered={}; {}",
                c.name, c.astero_registered, c.evidence
            )
            .unwrap();
        }
        writeln!(
            out,
            "Reference symbol[{}] NID={:?}: {:?}",
            r.symbol,
            r.nid.map(|x| format!("0x{x:016X}")),
            r.resolution
        )
        .unwrap();
    }
    let supported = p
        .relocations()
        .iter()
        .filter(|r| !matches!(r.action, Action::Unsupported(_)))
        .count();
    let concrete = p
        .relocations()
        .iter()
        .filter(|r| r.value.is_some() || r.action == Action::None)
        .count();
    writeln!(
        out,
        "Relocations: {}; supported action categories: {}; concrete values/no-ops: {}",
        p.relocations().len(),
        supported,
        concrete
    )
    .unwrap();
    for (i, r) in p.relocations().iter().take(16).enumerate() {
        writeln!(out,"Relocation[{i}] {:?} symbol={} type={} place={:?} addend={} value={:?} provenance={:?}",r.action,r.raw.record.symbol_index(),r.raw.record.relocation_type(),r.place,r.raw.record.addend,r.value,r.raw.table_kind).unwrap();
    }
    writeln!(
        out,
        "Readiness: {:?}; blockers: {}",
        p.readiness(),
        p.blockers().len()
    )
    .unwrap();
    for b in p.blockers().iter().take(16) {
        writeln!(out, "Blocker: {b:?}").unwrap();
    }
    writeln!(out,"Display limit: 16 per reference/relocation/blocker list; omitted={}/{}/{}. Plan retains all bounded records.",p.references().len().saturating_sub(16),p.relocations().len().saturating_sub(16),p.blockers().len().saturating_sub(16)).unwrap();
    if !matches!(p.input().outcome(), IdentityOutcome::Complete(_)) {
        writeln!(
            out,
            "Identity prerequisite: {:?}; linkage prerequisite: {:?}; symbols: {:?}",
            p.input().outcome(),
            p.input().linkage().outcome(),
            p.input().linkage().symbols().input().outcome()
        )
        .unwrap();
    }
    out.push_str("NO GUEST MEMORY MUTATED\nNO RELOCATIONS APPLIED\nNO GUEST EXECUTION\nNo HLE registration or runtime binding.\n");
    out
}
