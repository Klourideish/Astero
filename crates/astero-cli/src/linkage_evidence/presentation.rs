use astero_core::input::linkage_evidence::{LinkageOutcome, LinkageReport};
use std::fmt::Write;
pub fn render(report: &LinkageReport) -> String {
    let mut text = format!(
        "Explicit ELF linkage evidence (observations only)\nSource: {:?}; bytes: {}; provenance: {:?}\nLimits: {:?}\n",
        report.source().identity(),
        report.source().len(),
        report.source().provenance(),
        report.limits()
    );
    match report.outcome() {
        LinkageOutcome::Unavailable => {
            text.push_str("Linkage evidence: Unavailable (trusted symbol membership unavailable)\n")
        }
        LinkageOutcome::Failed(e) => {
            writeln!(text,"Linkage evidence: Failed\n{e}\nPrerequisite: {:?}\nSymbol prerequisite: {:?}\nNo successful linkage prefix retained.",report.symbols().outcome(),report.symbols().input().outcome()).expect("String write");
        }
        LinkageOutcome::Complete(e) => {
            writeln!(text, "Linkage evidence: Complete\nCounts: {:?}", e.counts())
                .expect("String write");
            writeln!(
                text,
                "Trusted proof: {:?}",
                report.symbols().input().proof().outcome()
            )
            .expect("String write");
            for ((role, symbol), usage) in report.symbols().entries().zip(e.symbol_uses()) {
                let f = symbol.fields();
                writeln!(text,"symbol[{}] {:?} {:?}; binding={:?} type={:?} visibility={:?} section={:?} raw-info=0x{:02X} other=0x{:02X}; name={}; refs={} ordinary={} plt={}; source={:?}",f.index,role.role(),usage.role,f.binding,f.symbol_type,f.visibility,f.section,f.info,f.other,crate::symbols::display_name(symbol.name_bytes()),usage.references,usage.ordinary,usage.plt,f.source).expect("String write");
            }
            for r in e.relocations() {
                writeln!(text,"relocation {:?}[{}] source={:?} offset=0x{:X} info=0x{:X} type={} symbol={} addend={} ordinary={:?} plt={:?}",r.table_kind,r.index,r.source,r.record.offset,r.record.info,r.record.relocation_type(),r.record.symbol_index(),r.record.addend,r.dynamic_index,r.plt_index).expect("String write");
            }
            for n in e.needed() {
                let bytes = report
                    .source()
                    .read(&n.source)
                    .expect("private validated same-source name token");
                writeln!(
                    text,
                    "DT_NEEDED entry={} offset={} name={} source={:?} (declaration only)",
                    n.dynamic_index,
                    n.offset,
                    crate::symbols::display_name(Some(bytes)),
                    n.source
                )
                .expect("String write");
            }
            if let Some(raw) = report.symbols().input().proof().raw() {
                for entry in raw.entries() {
                    writeln!(
                        text,
                        "dynamic[{}] {:?} raw-value=0x{:X}",
                        entry.index, entry.tag, entry.value
                    )
                    .expect("String write");
                }
            }
        }
    }
    text.push_str("All relocation types remain numerically observed; application semantics unsupported. All providers unresolved. No export claim, provider/NID resolution, relocation application, guest loading or execution.\n");
    text
}
