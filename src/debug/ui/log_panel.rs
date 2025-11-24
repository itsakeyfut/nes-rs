// Execution Log Panel

use super::DebugUI;
use crate::debug::Debugger;

/// Show the execution log panel
pub(super) fn show(ui_state: &mut DebugUI, ctx: &egui::Context, debugger: &mut Debugger) {
    egui::Window::new("Execution Log")
        .open(&mut ui_state.show_execution_log_panel)
        .default_width(700.0)
        .default_height(300.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Enable CPU Trace").clicked() {
                    debugger.logger.enable_cpu_trace();
                }
                if ui.button("Disable CPU Trace").clicked() {
                    debugger.logger.disable_cpu_trace();
                }
                if ui.button("Clear Log").clicked() {
                    debugger.logger.clear_buffer();
                }
            });

            ui.separator();

            let entries = debugger.logger.last_entries(100);
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in entries {
                        ui.monospace(entry.to_string());
                    }
                });
        });
}
