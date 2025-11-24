// Disassembly Panel

use super::DebugUI;
use crate::bus::Bus;
use crate::debug::Debugger;

/// Show the disassembly panel
pub(super) fn show(
    ui_state: &mut DebugUI,
    ctx: &egui::Context,
    _debugger: &Debugger,
    bus: &mut Bus,
) {
    egui::Window::new("Disassembly")
        .open(&mut ui_state.show_disassembly_panel)
        .default_width(500.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(&mut ui_state.disasm_address);

                ui.label("Instructions:");
                ui.add(egui::DragValue::new(&mut ui_state.disasm_count).range(1..=100));
            });

            ui.separator();

            if let Ok(addr) = u16::from_str_radix(&ui_state.disasm_address, 16) {
                let instructions =
                    crate::debug::disassemble_count(addr, ui_state.disasm_count, bus);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for instr in instructions {
                        ui.monospace(instr.to_string());
                    }
                });
            } else {
                ui.label("Invalid address");
            }
        });
}
