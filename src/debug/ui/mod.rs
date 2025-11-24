// Debug UI - egui integration for NES debugger
//
// This module provides an interactive debug UI using egui, with dockable panels
// for CPU state, memory viewing, PPU debugging, disassembly, and execution logs.

mod cpu_panel;
mod disasm_panel;
mod log_panel;
mod memory_panel;
mod ppu_panel;

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
    pub(super) show_cpu_panel: bool,

    /// Memory viewer panel visibility
    pub(super) show_memory_panel: bool,

    /// PPU debugger panel visibility
    pub(super) show_ppu_panel: bool,

    /// Disassembly panel visibility
    pub(super) show_disassembly_panel: bool,

    /// Execution log panel visibility
    pub(super) show_execution_log_panel: bool,

    /// Memory viewer address input
    pub(super) memory_address: String,

    /// Memory viewer byte count
    pub(super) memory_bytes: usize,

    /// Disassembly address input
    pub(super) disasm_address: String,

    /// Disassembly instruction count
    pub(super) disasm_count: usize,

    /// Breakpoint address input
    pub(super) breakpoint_input: String,
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
            cpu_panel::show(self, ctx, debugger, cpu, bus);
        }

        if self.show_memory_panel {
            memory_panel::show(self, ctx, debugger, bus);
        }

        if self.show_ppu_panel {
            ppu_panel::show(self, ctx, debugger, ppu);
        }

        if self.show_disassembly_panel {
            disasm_panel::show(self, ctx, debugger, bus);
        }

        if self.show_execution_log_panel {
            log_panel::show(self, ctx, debugger);
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
