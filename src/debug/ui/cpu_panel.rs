// CPU Debugger Panel

use super::DebugUI;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::debug::Debugger;

/// Show the CPU debugger panel
pub(super) fn show(
    ui_state: &mut DebugUI,
    ctx: &egui::Context,
    debugger: &mut Debugger,
    cpu: &Cpu,
    bus: &mut Bus,
) {
    egui::Window::new("CPU Debugger")
        .open(&mut ui_state.show_cpu_panel)
        .default_width(500.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            let state = debugger.get_cpu_state(cpu, bus);

            // Execution Controls - prominent at the top
            ui.heading("Execution Control");
            ui.separator();

            ui.horizontal(|ui| {
                if debugger.is_paused() {
                    if ui.button("▶ Continue").clicked() {
                        debugger.resume();
                    }
                    if ui.button("⏭ Step").clicked() {
                        debugger.step();
                    }
                } else if ui.button("⏸ Pause").clicked() {
                    debugger.pause();
                }

                // Clear breakpoints (placeholder until full CPU reset is wired)
                if ui.button("🧹 Clear Breakpoints").clicked() {
                    debugger.clear_breakpoints();
                }
            });

            ui.add_space(10.0);

            // Registers
            ui.heading("Registers");
            ui.separator();

            egui::Grid::new("registers_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label("PC:");
                    ui.monospace(format!("${:04X}", state.pc));
                    ui.end_row();

                    ui.label("A:");
                    ui.monospace(format!("${:02X} ({})", state.a, state.a));
                    ui.end_row();

                    ui.label("X:");
                    ui.monospace(format!("${:02X} ({})", state.x, state.x));
                    ui.end_row();

                    ui.label("Y:");
                    ui.monospace(format!("${:02X} ({})", state.y, state.y));
                    ui.end_row();

                    ui.label("SP:");
                    ui.monospace(format!("${:02X}", state.sp));
                    ui.end_row();

                    ui.label("Cycles:");
                    ui.monospace(format!("{}", state.cycles));
                    ui.end_row();
                });

            ui.add_space(10.0);

            // Status Flags with color coding
            ui.heading("Status Flags");
            ui.separator();

            ui.horizontal(|ui| {
                let flags = [
                    ('N', 0x80, "Negative"),
                    ('V', 0x40, "Overflow"),
                    ('-', 0x20, "Unused"),
                    ('B', 0x10, "Break"),
                    ('D', 0x08, "Decimal"),
                    ('I', 0x04, "Interrupt Disable"),
                    ('Z', 0x02, "Zero"),
                    ('C', 0x01, "Carry"),
                ];

                for (flag_char, flag_bit, flag_name) in &flags {
                    let is_set = state.status & flag_bit != 0;
                    let color = if is_set {
                        egui::Color32::from_rgb(0, 200, 0) // Green for set
                    } else {
                        egui::Color32::from_rgb(150, 150, 150) // Gray for clear
                    };

                    ui.colored_label(color, format!("{}", flag_char))
                        .on_hover_text(*flag_name);
                }

                ui.separator();
                ui.monospace(format!("${:02X}", state.status));
            });

            ui.add_space(10.0);

            // Current Instruction
            ui.heading("Current Instruction");
            ui.separator();

            ui.horizontal(|ui| {
                ui.monospace("►");
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 0),
                    format!("${:04X}  {}", state.pc, state.instruction.format_assembly()),
                );
            });

            ui.add_space(10.0);

            // Disassembly view - next instructions
            ui.heading("Disassembly");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    // Get next 10 instructions starting from PC
                    let instructions = crate::debug::disassemble_count(state.pc, 10, bus);
                    let breakpoints = debugger.breakpoints();

                    for (idx, instr) in instructions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Highlight current PC
                            if idx == 0 {
                                ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "►");
                            } else {
                                ui.label("  ");
                            }

                            // Check if there's a breakpoint at this address
                            let has_breakpoint = breakpoints.contains(&instr.address);
                            if has_breakpoint {
                                ui.colored_label(egui::Color32::RED, "●");
                            } else {
                                ui.label(" ");
                            }

                            // Show the instruction
                            let color = if idx == 0 {
                                egui::Color32::from_rgb(255, 255, 255)
                            } else {
                                egui::Color32::from_rgb(200, 200, 200)
                            };

                            ui.colored_label(
                                color,
                                format!(
                                    "${:04X}  {:8}  {}",
                                    instr.address,
                                    instr.format_bytes(),
                                    instr.format_assembly()
                                ),
                            );
                        });
                    }
                });

            ui.add_space(10.0);

            // Breakpoints
            ui.heading("Breakpoints");
            ui.separator();

            // Breakpoint management
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(&mut ui_state.breakpoint_input);

                if ui.button("Add").clicked() {
                    if let Ok(addr) = u16::from_str_radix(&ui_state.breakpoint_input, 16) {
                        debugger.add_breakpoint(addr);
                        ui_state.breakpoint_input.clear();
                    }
                }

                if ui.button("Add at PC").clicked() {
                    debugger.add_breakpoint(state.pc);
                }
            });

            ui.add_space(5.0);

            // List breakpoints in a scrollable area
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    let breakpoints = debugger.breakpoints();
                    if breakpoints.is_empty() {
                        ui.label("No breakpoints set");
                    } else {
                        for addr in &breakpoints {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::RED, "●");
                                ui.monospace(format!("${:04X}", addr));
                                if ui.small_button("✖").clicked() {
                                    debugger.remove_breakpoint(*addr);
                                }
                            });
                        }
                    }
                });

            if !debugger.breakpoints().is_empty() && ui.button("Clear All Breakpoints").clicked() {
                debugger.clear_breakpoints();
            }
        });
}
