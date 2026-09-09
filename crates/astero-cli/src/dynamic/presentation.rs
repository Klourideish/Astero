use astero_core::input::dynamic::{DynamicObservationReport, DynamicOutcome};
use std::fmt::Write;
pub fn render(report: &DynamicObservationReport) -> String {
    let mut text = format!(
        "Dynamic observation explicitly requested: raw PT_DYNAMIC entries only\nBudgets: max-program-headers={} max-dynamic-entries={}\nSource: {:?} | bytes: {} | provenance: {:?}\n",
        report.limits().max_program_headers,
        report.limits().max_dynamic_entries,
        report.source().identity(),
        report.source().len(),
        report.source().provenance()
    );
    match report.outcome() {
        DynamicOutcome::Unavailable => {
            text.push_str("Dynamic observation: Unavailable (no PT_DYNAMIC)\n")
        }
        DynamicOutcome::Failed(e) => {
            writeln!(
                text,
                "Dynamic observation: Failed\n{e}\nNo complete entry list retained."
            )
            .expect("String write");
        }
        DynamicOutcome::Complete(table) => {
            writeln!(text, "Dynamic observation: Complete (raw table scope only)\nProgram header: {} | source range: {:?}\nObserved entries: {}\nTermination: DT_NULL (retained; trailing bytes uninterpreted)", table.program_index(), table.source_range(), table.entries().len()).expect("String write");
            for e in table.entries() {
                writeln!(
                    text,
                    "Entry {}: tag={:?} raw-value={:#018x}",
                    e.index, e.tag, e.value
                )
                .expect("String write");
            }
        }
    }
    text.push_str("No guest loaded. No linkage derived. No execution occurred. No strings, symbols, hashes or relocation contents interpreted.\n");
    text
}
