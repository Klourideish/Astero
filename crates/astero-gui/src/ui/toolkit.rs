//! ImGui widgets and platform input adapter only. No Vulkan or session mutation.
use crate::model::SessionView;
use imgui::{Condition, Context};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::window::Window;

pub(crate) struct Toolkit {
    pub context: Context,
    pub platform: WinitPlatform,
}
impl Toolkit {
    pub fn new(window: &Window) -> Self {
        let mut context = Context::create();
        context.set_ini_filename(None);
        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: 18.0,
                    ..Default::default()
                }),
            }]);
        Self { context, platform }
    }

    pub fn prepare(&mut self, window: &Window, view: &SessionView) -> Result<(), String> {
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .map_err(|e| e.to_string())?;
        let ui = self.context.frame();
        let size = ui.io().display_size;
        ui.window("Astero Session Inspector")
            .position([0.0, 0.0], Condition::Always)
            .size(size, Condition::Always)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .build(|| {
                ui.text("Astero | Session Foundation");
                ui.text("Host session only. No guest execution.");
                ui.separator();
                match view.inspect() {
                    Err(e) => ui.text(format!("Inspection unavailable: {e:?}")),
                    Ok(report) => {
                        let s = &report.session;
                        ui.text(format!("Identity: {}", s.id));
                        ui.text(format!("Lifecycle: {:?}", s.lifecycle));
                        ui.text(format!(
                            "Loaded target: {}",
                            s.loaded_target
                                .as_ref()
                                .map_or("No guest loaded", |t| t.display_name.as_str())
                        ));
                        ui.text(format!(
                            "Host lifecycle changes: {}",
                            s.statistics.lifecycle_changes
                        ));
                        ui.separator();
                        if let Some(_table) = ui.begin_table("Inspector panes", 2) {
                            ui.table_next_column();
                            ui.text("Subsystem availability");
                            for sub in &s.subsystems {
                                ui.text(format!("{}: {:?}", sub.name, sub.availability));
                            }
                            ui.spacing();
                            if ui.collapsing_header(
                                "Diagnostics",
                                imgui::TreeNodeFlags::DEFAULT_OPEN,
                            ) {
                                for d in &s.diagnostics {
                                    ui.text(format!("{d:?}"));
                                }
                            }
                            ui.table_next_column();
                            ui.text("Debugger capabilities");
                            for (capability, support) in report.capabilities {
                                ui.text(format!("{capability:?}: {support:?}"));
                            }
                        }
                    }
                }
            });
        self.platform.prepare_render(ui, window);
        Ok(())
    }
}
