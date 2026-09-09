//! Display only; all classification, counts and completeness come from the loader report.
use astero_debug::snapshots::linkage::{Completeness, LinkageSnapshot};
use std::fmt::Write;
pub fn render(snapshot: &LinkageSnapshot, details: bool) -> String {
    let Some(report) = snapshot.report() else {
        return "Linkage evidence: unavailable (no report supplied)\nNo candidate totals available.\n".into();
    };
    let mut text = format!(
        "Linkage evidence | source {:?} | module {:?} (inspection context)\n",
        report.source(),
        report.module()
    );
    let qualifier = if matches!(report.completeness(), Completeness::Complete) {
        ""
    } else {
        " observed"
    };
    let status = match report.completeness() {
        Completeness::Complete => "complete",
        Completeness::Partial { .. } => "partial",
        Completeness::Unavailable(_) => "unavailable",
        Completeness::Failed(_) => "failed",
    };
    writeln!(text, "Enumeration: {status} ({:?})", report.completeness()).expect("String write");
    let c = report.counts();
    writeln!(
        text,
        "Import candidates{qualifier}: {}\nExport candidates{qualifier}: {}",
        c.imports, c.exports
    )
    .expect("String write");
    writeln!(
        text,
        "Symbol observations: {} | internal: {} | unclassified: {} | null: {}",
        c.observed, c.internal, c.unclassified, c.null
    )
    .expect("String write");
    writeln!(
        text,
        "Names observed: unnamed {} | empty {} | non-UTF-8 {}",
        c.unnamed, c.empty_names, c.non_utf8_names
    )
    .expect("String write");
    writeln!(
        text,
        "Unknown attributes observed: binding {} | type {} | visibility {}",
        c.unknown_binding, c.unknown_type, c.unknown_visibility
    )
    .expect("String write");
    writeln!(
        text,
        "Relocation references observed: unique {} | ordinary {} | PLT/JMPREL {}",
        c.relocations.unique, c.relocations.ordinary, c.relocations.plt
    )
    .expect("String write");
    writeln!(text, "Trusted extent: {:?}", report.extent()).expect("String write");
    if details {
        for row in report.details() {
            // Escaping bytes prevents control sequences and avoids lossy UTF-8 identities.
            let name = row.name.as_ref().map(|b| {
                b.iter()
                    .flat_map(|&b| std::ascii::escape_default(b))
                    .map(char::from)
                    .collect::<String>()
            });
            writeln!(text,"Symbol {}: {:?} | name {:?} {:?} | binding {:?} type {:?} visibility {:?} section {:?} | raw info {:#x} other {:#x} value {:#x} size {} | source {:?} name source {:?} | references {:?}",row.index,row.classification,row.name_state,name,row.binding,row.symbol_type,row.visibility,row.section,row.info,row.other,row.value,row.size,row.source,row.name_source,row.relocations).expect("String write");
        }
    }
    text
}
