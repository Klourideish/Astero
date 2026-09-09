use astero_core::input::{
    classification::{ClassificationOutcome, SymbolClassificationReport},
    symbols::SymbolOutcome,
};
use std::fmt::Write;
pub fn render(report: &SymbolClassificationReport) -> String {
    let input = report.input();
    let mut text = format!(
        "Symbol observation prerequisite:\n{}\nStructural classification explicitly requested\nLimit: max-classifications={}\n",
        crate::symbols::render(input),
        report.limits().max_classifications
    );
    match report.outcome() {
        ClassificationOutcome::Unavailable => {
            text.push_str("Classification: Unavailable (symbol observations unavailable)\n")
        }
        ClassificationOutcome::Failed(e) => {
            writeln!(
                text,
                "Classification: Failed\n{e}\nNo successful prefix retained."
            )
            .expect("String write");
            if let SymbolOutcome::Failed(e) = input.outcome() {
                writeln!(text, "Prerequisite failure: {e}").expect("String write");
            }
        }
        ClassificationOutcome::Complete(records) => {
            writeln!(
                text,
                "Classification: Complete\nClassifications: {}",
                records.len()
            )
            .expect("String write");
            for (r, s) in report.entries() {
                let f = s.fields();
                writeln!(text,"[{}] {:?}: st_shndx=0x{:04X} binding={:?} type={:?} visibility={:?} section={:?} name={}",r.symbol_index(),r.role(),f.shndx,f.binding,f.symbol_type,f.visibility,f.section,crate::symbols::display_name(s.name_bytes())).expect("String write");
            }
        }
    }
    text.push_str("Structural roles only: undefined is not an import; defined/global is not an export. No resolution, linkage, NID lookup, or guest loading/execution.\n");
    text
}
