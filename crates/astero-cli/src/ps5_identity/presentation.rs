use astero_core::input::ps5_identity::{IdentityOutcome, NameEvidence, Ps5IdentityEvidenceReport};
use std::fmt::Write;
pub fn render(r: &Ps5IdentityEvidenceReport) -> String {
    let input = r.linkage();
    let mut text = format!(
        "Explicit PS5 identity evidence\nSource: {:?}; bytes: {}; provenance: {:?}\nLimits: {:?}; linkage limits: {:?}\n",
        input.source().identity(),
        input.source().len(),
        input.source().provenance(),
        r.limits(),
        input.limits()
    );
    match r.outcome() {
        IdentityOutcome::Unavailable => text.push_str("Status: Unavailable\n"),
        IdentityOutcome::Failed(e) => {
            writeln!(
                text,
                "Status: Failed\n{e}\nLinkage prerequisite: {:?}\nNo successful identity prefix.",
                input.outcome()
            )
            .expect("String write");
        }
        IdentityOutcome::Complete(e) => {
            let confirmed = e
                .symbols()
                .iter()
                .filter(|s| matches!(s.evidence, NameEvidence::Encoded { .. }))
                .count();
            let candidates = e
                .symbols()
                .iter()
                .filter(|s| matches!(s.evidence, NameEvidence::Candidate(_)))
                .count();
            writeln!(text,"Status: Complete\nEncoded numeric NIDs: {confirmed}; unconfirmed candidates: {candidates}\nDescriptor ID/version/kind and suffix context: experimental hypothesis, not provider proof.").expect("String write");
            for d in e.descriptors() {
                let name = d
                    .name
                    .as_ref()
                    .map(|n| input.source().read(n).expect("same-source token"));
                writeln!(text,"metadata[{}] {:?} tag=0x{:X} raw=0x{:016X} id={:?} version-bits={:?} name={} source={:?}",d.dynamic_index,d.kind,d.tag,d.raw,d.id,d.version_bits,crate::symbols::display_name(name),d.name).expect("String write");
            }
            for s in e.symbols() {
                let name = s
                    .name
                    .as_ref()
                    .map(|n| input.source().read(n).expect("same-source token"));
                writeln!(
                    text,
                    "symbol[{}] name={} evidence={:?}",
                    s.symbol_index,
                    crate::symbols::display_name(name),
                    s.evidence
                )
                .expect("String write");
                if let NameEvidence::Encoded { nid, .. } = s.evidence {
                    writeln!(text,"  numeric NID=0x{nid:016X}; canonical encoded bytes, no function-name claim").expect("String write");
                }
            }
            if let astero_core::input::linkage_evidence::LinkageOutcome::Complete(e) =
                input.outcome()
            {
                writeln!(text, "Linkage prerequisite: {:?}", e.counts()).expect("String write");
                for n in e.needed() {
                    writeln!(
                        text,
                        "Dependency declaration: {}",
                        crate::symbols::display_name(Some(
                            input.source().read(&n.source).expect("same-source token")
                        ))
                    )
                    .expect("String write");
                }
            }
        }
    }
    text.push_str("No providers resolved; no HLE binding, dependency satisfaction, relocation application, guest loading or execution.\n");
    text
}
