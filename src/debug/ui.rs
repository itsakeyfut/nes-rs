// Debug UI - egui integration for NES debugger
//
// This module provides an interactive debug UI using egui, with dockable panels
// for CPU state, memory viewing, PPU debugging, disassembly, and execution logs.

use super::Debugger;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;

/// Main debug UI structure
///
/// Provides an egui-based interactive debug interface that can be embedded
/// in the emulator window. Supports multiple dockable panels for different
/// debugging features.
///
/// # Example
///
/// ```no_run
/// use nes_rs::debug::ui::DebugUI;
/// use nes_rs::debug::Debugger;
///
/// let mut debugger = Debugger::new();
/// let mut debug_ui = DebugUI::new();
///
/// // In your egui render loop:
/// // debug_ui.show(ctx, &mut debugger, &cpu, &mut bus, &ppu);
/// ```
pub struct DebugUI {
    /// Whether the debug UI is visible
    visible: bool,

    /// CPU debugger panel visibility
    show_cpu_panel: bool,

    /// Memory viewer panel visibility
    show_memory_panel: bool,

    /// PPU debugger panel visibility
    show_ppu_panel: bool,

    /// Disassembly panel visibility
    show_disassembly_panel: bool,

    /// Execution log panel visibility
    show_execution_log_panel: bool,

    /// Memory viewer address input
    memory_address: String,

    /// Memory viewer byte count
    memory_bytes: usize,

    /// Disassembly address input
    disasm_address: String,

    /// Disassembly instruction count
    disasm_count: usize,

    /// Breakpoint address input
    breakpoint_input: String,
}

impl DebugUI {
    /// Create a new debug UI instance
    ///
    /// # Returns
    ///
    /// A new debug UI with default settings (all panels enabled)
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::debug::ui::DebugUI;
    ///
    /// let debug_ui = DebugUI::new();
    /// ```
    pub fn new() -> Self {
        Self {
            visible: true,
            show_cpu_panel: true,
            show_memory_panel: true,
            show_ppu_panel: true,
            show_disassembly_panel: true,
            show_execution_log_panel: true,
            memory_address: String::from("0000"),
            memory_bytes: 256,
            disasm_address: String::from("8000"),
            disasm_count: 16,
            breakpoint_input: String::new(),
        }
    }

    /// Show or hide the debug UI
    ///
    /// # Arguments
    ///
    /// * `visible` - Whether the debug UI should be visible
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if the debug UI is visible
    ///
    /// # Returns
    ///
    /// `true` if the debug UI is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle the debug UI visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Render the debug UI
    ///
    /// This should be called from the egui render loop to display all debug panels.
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui context
    /// * `debugger` - Reference to the debugger
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Mutable reference to the bus
    /// * `ppu` - Reference to the PPU
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        debugger: &mut Debugger,
        cpu: &Cpu,
        bus: &mut Bus,
        ppu: &Ppu,
    ) {
        if !self.visible {
            return;
        }

        // Main menu bar
        self.show_menu_bar(ctx, debugger);

        // Show enabled panels
        if self.show_cpu_panel {
            self.show_cpu_debugger(ctx, debugger, cpu, bus);
        }

        if self.show_memory_panel {
            self.show_memory_viewer(ctx, debugger, bus);
        }

        if self.show_ppu_panel {
            self.show_ppu_debugger(ctx, debugger, ppu);
        }

        if self.show_disassembly_panel {
            self.show_disassembly(ctx, debugger, bus);
        }

        if self.show_execution_log_panel {
            self.show_execution_log(ctx, debugger);
        }
    }

    /// Show the main menu bar
    fn show_menu_bar(&mut self, ctx: &egui::Context, debugger: &mut Debugger) {
        egui::TopBottomPanel::top("debug_menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Debug", |ui| {
                    if ui.button("Enable Debugger").clicked() {
                        debugger.enable();
                    }
                    if ui.button("Disable Debugger").clicked() {
                        debugger.disable();
                    }
                    ui.separator();

                    if debugger.is_paused() {
                        if ui.button("Resume").clicked() {
                            debugger.resume();
                        }
                        if ui.button("Step").clicked() {
                            debugger.step();
                        }
                    } else if ui.button("Pause").clicked() {
                        debugger.pause();
                    }
                });

                ui.menu_button("Panels", |ui| {
                    ui.checkbox(&mut self.show_cpu_panel, "CPU Debugger");
                    ui.checkbox(&mut self.show_memory_panel, "Memory Viewer");
                    ui.checkbox(&mut self.show_ppu_panel, "PPU Debugger");
                    ui.checkbox(&mut self.show_disassembly_panel, "Disassembly");
                    ui.checkbox(&mut self.show_execution_log_panel, "Execution Log");
                });

                // Status indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if debugger.is_enabled() {
                        ui.colored_label(egui::Color32::GREEN, "● Enabled");
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "○ Disabled");
                    }

                    if debugger.is_paused() {
                        ui.colored_label(egui::Color32::YELLOW, "⏸ Paused");
                    }
                });
            });
        });
    }

    /// Show the CPU debugger panel
    fn show_cpu_debugger(
        &mut self,
        ctx: &egui::Context,
        debugger: &mut Debugger,
        cpu: &Cpu,
        bus: &mut Bus,
    ) {
        egui::Window::new("CPU Debugger")
            .open(&mut self.show_cpu_panel)
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

                    // Reset button (note: actual reset needs emulator access)
                    if ui.button("🔄 Reset").clicked() {
                        // This would need to be handled by the emulator
                        // For now, just clear breakpoints as a placeholder
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

                        for (idx, instr) in instructions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Highlight current PC
                                if idx == 0 {
                                    ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "►");
                                } else {
                                    ui.label("  ");
                                }

                                // Check if there's a breakpoint at this address
                                let has_breakpoint =
                                    debugger.breakpoints().contains(&instr.address);
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
                    ui.text_edit_singleline(&mut self.breakpoint_input);

                    if ui.button("Add").clicked() {
                        if let Ok(addr) = u16::from_str_radix(&self.breakpoint_input, 16) {
                            debugger.add_breakpoint(addr);
                            self.breakpoint_input.clear();
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

                if !debugger.breakpoints().is_empty()
                    && ui.button("Clear All Breakpoints").clicked()
                {
                    debugger.clear_breakpoints();
                }
            });
    }

    /// Show the memory viewer panel
    fn show_memory_viewer(&mut self, ctx: &egui::Context, debugger: &Debugger, bus: &mut Bus) {
        egui::Window::new("Memory Viewer")
            .open(&mut self.show_memory_panel)
            .default_width(600.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Address:");
                    ui.text_edit_singleline(&mut self.memory_address);

                    ui.label("Bytes:");
                    ui.add(egui::DragValue::new(&mut self.memory_bytes).range(1..=4096));
                });

                ui.separator();

                if let Ok(addr) = u16::from_str_radix(&self.memory_address, 16) {
                    let dump = debugger
                        .memory
                        .dump_cpu_memory(bus, addr, self.memory_bytes);
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
                        self.memory_address = String::from("0000");
                        self.memory_bytes = 256;
                    }
                    if ui.button("Stack").clicked() {
                        self.memory_address = String::from("0100");
                        self.memory_bytes = 256;
                    }
                    if ui.button("ROM").clicked() {
                        self.memory_address = String::from("8000");
                        self.memory_bytes = 512;
                    }
                });
            });
    }

    /// Show the PPU debugger panel
    fn show_ppu_debugger(&mut self, ctx: &egui::Context, debugger: &Debugger, ppu: &Ppu) {
        egui::Window::new("PPU Debugger")
            .open(&mut self.show_ppu_panel)
            .default_width(400.0)
            .show(ctx, |ui| {
                let state = debugger.get_ppu_state(ppu);

                ui.heading("PPU State");
                ui.separator();

                ui.monospace(format!("Scanline: {}", state.scanline));
                ui.monospace(format!("Cycle:    {}", state.cycle));
                ui.monospace(format!("Frame:    {}", state.frame));

                ui.separator();
                ui.heading("Registers");
                ui.separator();

                ui.monospace(format!("PPUCTRL:   ${:02X}", state.ppuctrl));
                ui.monospace(format!("PPUMASK:   ${:02X}", state.ppumask));
                ui.monospace(format!("PPUSTATUS: ${:02X}", state.ppustatus));

                ui.separator();
                ui.heading("Scroll & Address");
                ui.separator();

                ui.monospace(format!("V (VRAM addr): ${:04X}", state.v));
                ui.monospace(format!("T (Temp addr): ${:04X}", state.t));
                ui.monospace(format!("Fine X: {}", state.fine_x));

                ui.separator();
                ui.heading("Palettes");
                ui.separator();

                ui.monospace(debugger.ppu.format_palettes(ppu));
            });
    }

    /// Show the disassembly panel
    fn show_disassembly(&mut self, ctx: &egui::Context, _debugger: &Debugger, bus: &mut Bus) {
        egui::Window::new("Disassembly")
            .open(&mut self.show_disassembly_panel)
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Address:");
                    ui.text_edit_singleline(&mut self.disasm_address);

                    ui.label("Instructions:");
                    ui.add(egui::DragValue::new(&mut self.disasm_count).range(1..=100));
                });

                ui.separator();

                if let Ok(addr) = u16::from_str_radix(&self.disasm_address, 16) {
                    let instructions =
                        crate::debug::disassemble_count(addr, self.disasm_count, bus);

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

    /// Show the execution log panel
    fn show_execution_log(&mut self, ctx: &egui::Context, debugger: &mut Debugger) {
        egui::Window::new("Execution Log")
            .open(&mut self.show_execution_log_panel)
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
}

impl Default for DebugUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_ui_creation() {
        let debug_ui = DebugUI::new();
        assert!(debug_ui.is_visible());
        assert!(debug_ui.show_cpu_panel);
        assert!(debug_ui.show_memory_panel);
        assert!(debug_ui.show_ppu_panel);
    }

    #[test]
    fn test_debug_ui_visibility() {
        let mut debug_ui = DebugUI::new();

        assert!(debug_ui.is_visible());

        debug_ui.set_visible(false);
        assert!(!debug_ui.is_visible());

        debug_ui.toggle();
        assert!(debug_ui.is_visible());

        debug_ui.toggle();
        assert!(!debug_ui.is_visible());
    }

    #[test]
    fn test_default() {
        let debug_ui = DebugUI::default();
        assert!(debug_ui.is_visible());
    }
}
