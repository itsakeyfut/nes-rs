// Memory Viewer Panel

use super::DebugUI;
use crate::bus::Bus;
use crate::debug::Debugger;

/// Show the memory viewer panel
pub(super) fn show(
    ui_state: &mut DebugUI,
    ctx: &egui::Context,
    debugger: &Debugger,
    bus: &mut Bus,
) {
    egui::Window::new("Memory Viewer")
        .open(&mut ui_state.show_memory_panel)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(&mut ui_state.memory_address);

                ui.label("Bytes:");
                ui.add(egui::DragValue::new(&mut ui_state.memory_bytes).range(1..=4096));
            });

            ui.separator();

            if let Ok(addr) = u16::from_str_radix(&ui_state.memory_address, 16) {
                let dump = debugger
                    .memory
                    .dump_cpu_memory(bus, addr, ui_state.memory_bytes);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace(dump);
                });
            } else {
                ui.label("Invalid address");
            }

            ui.separator();

            // Quick access buttons
            ui.horizontal(|ui| {
                if ui.button("Zero Page").clicked() {
                    ui_state.memory_address = String::from("0000");
                    ui_state.memory_bytes = 256;
                }
                if ui.button("Stack").clicked() {
                    ui_state.memory_address = String::from("0100");
                    ui_state.memory_bytes = 256;
                }
                if ui.button("ROM").clicked() {
                    ui_state.memory_address = String::from("8000");
                    ui_state.memory_bytes = 512;
                }
            });
        });
}
