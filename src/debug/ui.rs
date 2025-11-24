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
        let mut show_panel = self.show_ppu_panel;

        egui::Window::new("PPU Debugger")
            .open(&mut show_panel)
            .default_width(800.0)
            .default_height(600.0)
            .vscroll(true)
            .show(ctx, |ui| {
                let state = debugger.get_ppu_state(ppu);

                // Timing Information with color-coded VBlank/NMI status
                ui.heading("Timing Information");
                ui.separator();

                egui::Grid::new("ppu_timing_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Scanline:");
                        ui.monospace(format!("{}", state.scanline));
                        ui.end_row();

                        ui.label("Cycle:");
                        ui.monospace(format!("{}", state.cycle));
                        ui.end_row();

                        ui.label("Frame:");
                        ui.monospace(format!("{}", state.frame));
                        ui.end_row();

                        // VBlank status with color coding
                        ui.label("VBlank:");
                        let vblank_active = (state.ppustatus & 0x80) != 0;
                        if vblank_active {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Active");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Inactive");
                        }
                        ui.end_row();

                        // NMI status with color coding
                        ui.label("NMI:");
                        if state.nmi_pending {
                            ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "Pending");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "None");
                        }
                        ui.end_row();
                    });

                ui.add_space(10.0);

                // PPU Registers with detailed bit breakdowns
                ui.heading("PPU Registers");
                ui.separator();

                // PPUCTRL ($2000)
                ui.label(egui::RichText::new("PPUCTRL ($2000)").strong());
                ui.horizontal(|ui| {
                    ui.monospace(format!("${:02X}", state.ppuctrl));
                    ui.label("|");
                    ui.monospace(format!("{:08b}", state.ppuctrl));
                });
                ui.indent("ppuctrl_bits", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Bit 7:");
                        let nmi_enabled = (state.ppuctrl & 0x80) != 0;
                        if nmi_enabled {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "NMI Enabled");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "NMI Disabled");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 5:");
                        if (state.ppuctrl & 0x20) != 0 {
                            ui.label("Sprites 8x16");
                        } else {
                            ui.label("Sprites 8x8");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 4:");
                        if (state.ppuctrl & 0x10) != 0 {
                            ui.label("BG pattern table $1000");
                        } else {
                            ui.label("BG pattern table $0000");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 3:");
                        if (state.ppuctrl & 0x08) != 0 {
                            ui.label("Sprite pattern table $1000");
                        } else {
                            ui.label("Sprite pattern table $0000");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 2:");
                        if (state.ppuctrl & 0x04) != 0 {
                            ui.label("VRAM increment +32 (down)");
                        } else {
                            ui.label("VRAM increment +1 (across)");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bits 1-0:");
                        let nametable = state.ppuctrl & 0x03;
                        ui.label(format!(
                            "Base nametable {}",
                            match nametable {
                                0 => "$2000",
                                1 => "$2400",
                                2 => "$2800",
                                3 => "$2C00",
                                _ => unreachable!(),
                            }
                        ));
                    });
                });

                ui.add_space(5.0);

                // PPUMASK ($2001)
                ui.label(egui::RichText::new("PPUMASK ($2001)").strong());
                ui.horizontal(|ui| {
                    ui.monospace(format!("${:02X}", state.ppumask));
                    ui.label("|");
                    ui.monospace(format!("{:08b}", state.ppumask));
                });
                ui.indent("ppumask_bits", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Bit 7:");
                        if (state.ppumask & 0x80) != 0 {
                            ui.label("Emphasize blue");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 6:");
                        if (state.ppumask & 0x40) != 0 {
                            ui.label("Emphasize green");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 5:");
                        if (state.ppumask & 0x20) != 0 {
                            ui.label("Emphasize red");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 4:");
                        let show_sprites = (state.ppumask & 0x10) != 0;
                        if show_sprites {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Show sprites");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Hide sprites");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 3:");
                        let show_bg = (state.ppumask & 0x08) != 0;
                        if show_bg {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Show background");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Hide background");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 2:");
                        if (state.ppumask & 0x04) != 0 {
                            ui.label("Show sprites in leftmost 8 pixels");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 1:");
                        if (state.ppumask & 0x02) != 0 {
                            ui.label("Show background in leftmost 8 pixels");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 0:");
                        if (state.ppumask & 0x01) != 0 {
                            ui.label("Grayscale mode");
                        } else {
                            ui.label("-");
                        }
                    });
                });

                ui.add_space(5.0);

                // PPUSTATUS ($2002)
                ui.label(egui::RichText::new("PPUSTATUS ($2002)").strong());
                ui.horizontal(|ui| {
                    ui.monospace(format!("${:02X}", state.ppustatus));
                    ui.label("|");
                    ui.monospace(format!("{:08b}", state.ppustatus));
                });
                ui.indent("ppustatus_bits", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Bit 7:");
                        let vblank = (state.ppustatus & 0x80) != 0;
                        if vblank {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "VBlank");
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "No VBlank");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 6:");
                        let sprite0_hit = (state.ppustatus & 0x40) != 0;
                        if sprite0_hit {
                            ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "Sprite 0 hit");
                        } else {
                            ui.label("-");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bit 5:");
                        let sprite_overflow = (state.ppustatus & 0x20) != 0;
                        if sprite_overflow {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 100, 0),
                                "Sprite overflow",
                            );
                        } else {
                            ui.label("-");
                        }
                    });
                });

                ui.add_space(5.0);

                // OAMADDR ($2003)
                ui.label(egui::RichText::new("OAMADDR ($2003)").strong());
                ui.monospace(format!("${:02X}", state.oam_addr));

                ui.add_space(10.0);

                // Scroll & Address
                ui.heading("Scroll & Address Registers");
                ui.separator();

                egui::Grid::new("ppu_scroll_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("v (VRAM addr):");
                        ui.monospace(format!("${:04X}", state.v));
                        ui.end_row();

                        ui.label("t (Temp addr):");
                        ui.monospace(format!("${:04X}", state.t));
                        ui.end_row();

                        ui.label("Fine X scroll:");
                        ui.monospace(format!("{}", state.fine_x));
                        ui.end_row();

                        ui.label("Write latch:");
                        ui.monospace(format!("{}", if state.write_latch { 1 } else { 0 }));
                        ui.end_row();
                    });

                ui.add_space(10.0);

                // Palette Viewer with visual color swatches
                ui.heading("Palette Viewer");
                ui.separator();

                self.show_palette_viewer(ui, ppu);

                ui.add_space(10.0);

                // Pattern Table Viewer
                ui.heading("Pattern Tables");
                ui.separator();

                self.show_pattern_tables(ui, ppu, state.ppuctrl);

                ui.add_space(10.0);

                // Nametable Viewer
                ui.heading("Nametables");
                ui.separator();

                self.show_nametables(ui, ppu);
            });

        self.show_ppu_panel = show_panel;
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

    /// Show palette viewer with visual color swatches
    fn show_palette_viewer(&self, ui: &mut egui::Ui, ppu: &Ppu) {
        use crate::display::palette::palette_to_rgb;

        // Background Palettes
        ui.label(egui::RichText::new("Background Palettes").strong());
        for i in 0..4 {
            ui.horizontal(|ui| {
                ui.label(format!("Palette {}:", i));
                for j in 0..4 {
                    let index = i * 4 + j;
                    let color_index = ppu.palette_ram[index];
                    let rgb = palette_to_rgb(color_index);

                    // Convert RGB to egui Color32
                    let color = egui::Color32::from_rgb(
                        ((rgb >> 16) & 0xFF) as u8,
                        ((rgb >> 8) & 0xFF) as u8,
                        (rgb & 0xFF) as u8,
                    );

                    // Draw color swatch
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);

                    // Show color index on hover
                    response.on_hover_text(format!("Color: ${:02X}", color_index));
                }
            });
        }

        ui.add_space(5.0);

        // Sprite Palettes
        ui.label(egui::RichText::new("Sprite Palettes").strong());
        for i in 0..4 {
            ui.horizontal(|ui| {
                ui.label(format!("Palette {}:", i));
                for j in 0..4 {
                    let index = 16 + i * 4 + j;
                    let color_index = ppu.palette_ram[index];
                    let rgb = palette_to_rgb(color_index);

                    // Convert RGB to egui Color32
                    let color = egui::Color32::from_rgb(
                        ((rgb >> 16) & 0xFF) as u8,
                        ((rgb >> 8) & 0xFF) as u8,
                        (rgb & 0xFF) as u8,
                    );

                    // Draw color swatch
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);

                    // Show color index on hover
                    response.on_hover_text(format!("Color: ${:02X}", color_index));
                }
            });
        }
    }

    /// Show pattern tables as visual grids
    fn show_pattern_tables(&self, ui: &mut egui::Ui, ppu: &Ppu, ppuctrl: u8) {

        ui.horizontal(|ui| {
            // Left pattern table ($0000-$0FFF)
            ui.vertical(|ui| {
                ui.label("Left ($0000-$0FFF)");
                self.render_pattern_table(ui, ppu, 0x0000, 0);
            });

            ui.add_space(10.0);

            // Right pattern table ($1000-$1FFF)
            ui.vertical(|ui| {
                ui.label("Right ($1000-$1FFF)");
                self.render_pattern_table(ui, ppu, 0x1000, 1);
            });
        });

        // Show which table is being used for BG and sprites
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("Active tables:");
            let bg_table = if (ppuctrl & 0x10) != 0 {
                "Right ($1000)"
            } else {
                "Left ($0000)"
            };
            ui.colored_label(
                egui::Color32::from_rgb(0, 200, 0),
                format!("BG: {}", bg_table),
            );
            ui.label("|");
            let sprite_table = if (ppuctrl & 0x08) != 0 {
                "Right ($1000)"
            } else {
                "Left ($0000)"
            };
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 0),
                format!("Sprites: {}", sprite_table),
            );
        });
    }

    /// Render a single pattern table
    fn render_pattern_table(
        &self,
        ui: &mut egui::Ui,
        ppu: &Ppu,
        base_addr: u16,
        _table_id: usize,
    ) {
        use crate::display::palette::palette_to_rgb;

        // Each pattern table is 16x16 tiles, each tile is 8x8 pixels
        // We'll render at 2x scale for visibility (total: 256x256 pixels)
        const SCALE: f32 = 2.0;
        const TILE_SIZE: f32 = 8.0 * SCALE;
        const TABLE_SIZE: f32 = 16.0 * TILE_SIZE;

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(TABLE_SIZE, TABLE_SIZE), egui::Sense::hover());

        // Use palette 0 for preview (can make this selectable later)
        let palette = [
            palette_to_rgb(ppu.palette_ram[0]),
            palette_to_rgb(ppu.palette_ram[1]),
            palette_to_rgb(ppu.palette_ram[2]),
            palette_to_rgb(ppu.palette_ram[3]),
        ];

        // Render each tile in the pattern table
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let tile_index = tile_y * 16 + tile_x;
                let tile_addr = base_addr + (tile_index * 16);

                // Read tile data (8 bytes for low plane, 8 bytes for high plane)
                let mut tile_low = [0u8; 8];
                let mut tile_high = [0u8; 8];
                for row in 0..8 {
                    tile_low[row] = ppu.read_ppu_memory(tile_addr + row as u16);
                    tile_high[row] = ppu.read_ppu_memory(tile_addr + 8 + row as u16);
                }

                // Render the tile
                for py in 0..8 {
                    for px in 0..8 {
                        // Get pixel color (2-bit value from combining low and high planes)
                        let bit = 7 - px;
                        let low_bit = (tile_low[py] >> bit) & 1;
                        let high_bit = (tile_high[py] >> bit) & 1;
                        let pixel_value = (high_bit << 1) | low_bit;

                        let rgb = palette[pixel_value as usize];
                        let color = egui::Color32::from_rgb(
                            ((rgb >> 16) & 0xFF) as u8,
                            ((rgb >> 8) & 0xFF) as u8,
                            (rgb & 0xFF) as u8,
                        );

                        // Calculate pixel position
                        let pixel_x = rect.min.x + ((tile_x * 8) as usize + px) as f32 * SCALE;
                        let pixel_y = rect.min.y + ((tile_y * 8) as usize + py) as f32 * SCALE;
                        let pixel_rect = egui::Rect::from_min_size(
                            egui::pos2(pixel_x, pixel_y),
                            egui::vec2(SCALE, SCALE),
                        );

                        ui.painter().rect_filled(pixel_rect, 0.0, color);
                    }
                }
            }
        }

    }

    /// Show nametables
    fn show_nametables(&self, ui: &mut egui::Ui, ppu: &Ppu) {
        ui.label("Nametable viewer - displaying 4 nametables");

        ui.horizontal(|ui| {
            // Top row: Nametable 0 and 1
            ui.vertical(|ui| {
                ui.label("Nametable 0 ($2000)");
                self.render_nametable(ui, ppu, 0x2000, 0);
            });

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.label("Nametable 1 ($2400)");
                self.render_nametable(ui, ppu, 0x2400, 1);
            });
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Bottom row: Nametable 2 and 3
            ui.vertical(|ui| {
                ui.label("Nametable 2 ($2800)");
                self.render_nametable(ui, ppu, 0x2800, 2);
            });

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.label("Nametable 3 ($2C00)");
                self.render_nametable(ui, ppu, 0x2C00, 3);
            });
        });
    }

    /// Render a single nametable
    fn render_nametable(
        &self,
        ui: &mut egui::Ui,
        ppu: &Ppu,
        base_addr: u16,
        _nametable_id: usize,
    ) {

        // Each nametable is 32x30 tiles, each tile is 8x8 pixels
        // We'll render at a small scale to fit on screen (total: ~256x240 pixels at 1x)
        const SCALE: f32 = 1.0;
        const TILE_SIZE: f32 = 8.0 * SCALE;
        const TABLE_WIDTH: f32 = 32.0 * TILE_SIZE;
        const TABLE_HEIGHT: f32 = 30.0 * TILE_SIZE;

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(TABLE_WIDTH, TABLE_HEIGHT), egui::Sense::hover());

        // Get pattern table base from PPUCTRL (we'll need to access this for tile rendering)
        // For now, just render a placeholder showing nametable data exists
        // Full implementation would read tile indices and render actual tiles

        // Simple visualization: just show that nametable data exists
        // by drawing a grid pattern based on nametable bytes
        for ty in 0..30 {
            for tx in 0..32 {
                let tile_index_addr = base_addr + (ty * 32 + tx);
                let tile_index = ppu.read_ppu_memory(tile_index_addr);

                // Use tile index to determine a grayscale value
                let gray_value = tile_index;
                let color = egui::Color32::from_rgb(gray_value, gray_value, gray_value);

                let tile_x = rect.min.x + tx as f32 * TILE_SIZE;
                let tile_y = rect.min.y + ty as f32 * TILE_SIZE;
                let tile_rect = egui::Rect::from_min_size(
                    egui::pos2(tile_x, tile_y),
                    egui::vec2(TILE_SIZE, TILE_SIZE),
                );

                ui.painter().rect_filled(tile_rect, 0.0, color);
            }
        }

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
