use crate::model::linkage::{LinkageView, display_name};
use imgui::Ui;
pub(crate) fn draw(ui: &Ui, view: &LinkageView<'_>, selected: &mut Option<u64>) {
    ui.separator();
    ui.text("Linkage evidence (read-only)");
    ui.text_colored([1.0, 0.8, 0.3, 1.0], view.provenance());
    ui.text(view.loaded_state());
    ui.text_wrapped(view.status());
    let Some(report) = view.report() else {
        return;
    };
    ui.text_wrapped(format!(
        "Source {:?} | module {:?} (inspection context)",
        report.source(),
        report.module()
    ));
    if let Some(counts) = view.counts() {
        ui.text(format!(
            "Import candidates observed: {} | Export candidates observed: {}",
            counts.imports, counts.exports
        ));
        ui.text(format!(
            "Internal observed: {} | Unclassified observed: {} | Null: {}",
            counts.internal, counts.unclassified, counts.null
        ));
        ui.text(format!(
            "Relocation references observed: unique {} | ordinary {} | PLT/JMPREL {}",
            counts.relocations.unique, counts.relocations.ordinary, counts.relocations.plt
        ));
    }
    if let Some(extent) = report.extent() {
        ui.text_wrapped(format!(
            "Trusted symbol extent: {} symbols, evidence {:?}",
            extent.symbol_count(),
            extent.evidence()
        ));
    }
    ui.separator();
    if let Some(_table) = ui.begin_table("Retained linkage detail", 2) {
        ui.table_next_column();
        ui.text(format!("Retained symbols: {}", view.details().len()));
        ui.child_window("Symbol selection")
            .size([0.0, 240.0])
            .build(|| {
                for row in view.details() {
                    if ui
                        .selectable_config(format!(
                            "Symbol {} | {:?}",
                            row.index, row.classification
                        ))
                        .selected(*selected == Some(row.index))
                        .build()
                    {
                        *selected = Some(row.index);
                    }
                    ui.text_wrapped(display_name(row));
                }
            });
        ui.table_next_column();
        ui.text("Selected evidence");
        if let Some(row) = selected.and_then(|index| view.selected(index)) {
            ui.text(format!("Symbol index: {}", row.index));
            ui.text_wrapped(format!("Classification: {:?}", row.classification));
            ui.text_wrapped(display_name(row));
            ui.text(format!(
                "Binding: {:?} | Type: {:?}",
                row.binding, row.symbol_type
            ));
            ui.text(format!(
                "Visibility: {:?} | Section: {:?}",
                row.visibility, row.section
            ));
            ui.text_wrapped(format!(
                "References: {} unique, {} ordinary, {} PLT/JMPREL",
                row.relocations.unique, row.relocations.ordinary, row.relocations.plt
            ));
            ui.text_wrapped(format!(
                "Source: {:?} | Name source: {:?}",
                row.source, row.name_source
            ));
            ui.text(format!(
                "Raw info: {:#x} | other: {:#x}",
                row.info, row.other
            ));
        } else {
            ui.text_wrapped("Select a retained symbol to inspect its evidence.");
        }
    }
}
