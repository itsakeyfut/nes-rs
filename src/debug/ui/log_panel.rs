// Execution Log Panel - Display execution log with search and filters

use super::DebugUI;
use crate::debug::{Debugger, ExecutionLogEntry, LogFilter};
use egui::{ScrollArea, TextEdit};

/// UI state for the execution log panel
pub struct LogPanelState {
    /// Search query
    pub search_query: String,
    /// Log filter settings
    pub filter: LogFilter,
    /// Auto-scroll enabled
    pub auto_scroll: bool,
    /// Export file path
    pub export_path: String,
}

impl Default for LogPanelState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            filter: LogFilter::default(),
            auto_scroll: true,
            export_path: "execution_log.txt".to_string(),
        }
    }
}

/// Show the execution log panel
pub(super) fn show(ui_state: &mut DebugUI, ctx: &egui::Context, debugger: &mut Debugger) {
    // Initialize log panel state if needed
    if ui_state.log_panel_state.is_none() {
        ui_state.log_panel_state = Some(LogPanelState::default());
    }

    let log_panel_state = ui_state.log_panel_state.as_mut().unwrap();

    egui::Window::new("Execution Log")
        .open(&mut ui_state.show_execution_log_panel)
        .default_width(900.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            // Top controls
            ui.horizontal(|ui| {
                ui.heading("Execution Log");

                ui.separator();

                // Buffer size display
                ui.label(format!(
                    "Entries: {} / {}",
                    debugger.execution_log.len(),
                    debugger.execution_log.max_entries()
                ));
            });

            ui.separator();

            // Logging controls
            ui.horizontal(|ui| {
                ui.label("Enable:");

                let mut instr_enabled = debugger.execution_log.is_instruction_logging_enabled();
                if ui.checkbox(&mut instr_enabled, "Instructions").changed() {
                    if instr_enabled {
                        debugger.execution_log.enable_instruction_logging();
                    } else {
                        debugger.execution_log.disable_instruction_logging();
                    }
                }

                let mut mem_read_enabled = debugger.execution_log.is_memory_read_logging_enabled();
                if ui.checkbox(&mut mem_read_enabled, "Memory Reads").changed() {
                    if mem_read_enabled {
                        debugger.execution_log.enable_memory_read_logging();
                    } else {
                        debugger.execution_log.disable_memory_read_logging();
                    }
                }

                let mut mem_write_enabled =
                    debugger.execution_log.is_memory_write_logging_enabled();
                if ui
                    .checkbox(&mut mem_write_enabled, "Memory Writes")
                    .changed()
                {
                    if mem_write_enabled {
                        debugger.execution_log.enable_memory_write_logging();
                    } else {
                        debugger.execution_log.disable_memory_write_logging();
                    }
                }

                let mut ppu_events_enabled = debugger.execution_log.is_ppu_event_logging_enabled();
                if ui.checkbox(&mut ppu_events_enabled, "PPU Events").changed() {
                    if ppu_events_enabled {
                        debugger.execution_log.enable_ppu_event_logging();
                    } else {
                        debugger.execution_log.disable_ppu_event_logging();
                    }
                }
            });

            ui.separator();

            // Filter and search controls
            ui.horizontal(|ui| {
                ui.label("Show:");

                ui.checkbox(
                    &mut log_panel_state.filter.show_instructions,
                    "Instructions",
                );
                ui.checkbox(&mut log_panel_state.filter.show_memory_reads, "Mem Reads");
                ui.checkbox(&mut log_panel_state.filter.show_memory_writes, "Mem Writes");
                ui.checkbox(&mut log_panel_state.filter.show_ppu_events, "PPU Events");
            });

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(
                    TextEdit::singleline(&mut log_panel_state.search_query)
                        .desired_width(200.0)
                        .hint_text("Filter by mnemonic, address, etc."),
                );

                if ui.button("Clear Search").clicked() {
                    log_panel_state.search_query.clear();
                }
            });

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Clear Log").clicked() {
                    debugger.execution_log.clear();
                }

                ui.checkbox(&mut log_panel_state.auto_scroll, "Auto-scroll");

                ui.separator();

                ui.label("Export to:");
                ui.add(
                    TextEdit::singleline(&mut log_panel_state.export_path)
                        .desired_width(200.0)
                        .hint_text("execution_log.txt"),
                );

                if ui.button("Export").clicked() {
                    if let Err(e) = debugger
                        .execution_log
                        .export_to_file(&log_panel_state.export_path, Some(&log_panel_state.filter))
                    {
                        eprintln!("Failed to export log: {}", e);
                    } else {
                        println!("Log exported to {}", log_panel_state.export_path);
                    }
                }
            });

            ui.separator();

            // Log entries
            let filtered_entries = debugger
                .execution_log
                .get_filtered_entries(&log_panel_state.search_query, &log_panel_state.filter);

            ui.label(format!("Showing {} entries", filtered_entries.len()));

            ui.separator();

            // Scrollable log area
            ScrollArea::vertical()
                .stick_to_bottom(log_panel_state.auto_scroll)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for entry in filtered_entries {
                            // Color code entries by type
                            let color = match entry {
                                ExecutionLogEntry::Instruction { .. } => {
                                    egui::Color32::from_rgb(200, 200, 200)
                                }
                                ExecutionLogEntry::MemoryRead { .. } => {
                                    egui::Color32::from_rgb(150, 200, 255)
                                }
                                ExecutionLogEntry::MemoryWrite { .. } => {
                                    egui::Color32::from_rgb(255, 180, 150)
                                }
                                ExecutionLogEntry::PpuEvent { .. } => {
                                    egui::Color32::from_rgb(200, 255, 150)
                                }
                            };

                            ui.colored_label(color, format!("{}", entry));
                        }
                    });
                });
        });
}
