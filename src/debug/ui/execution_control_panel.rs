// Execution Control Panel - UI for controlling emulation execution
//
// This panel provides controls for:
// - Playback control (play/pause, step modes, reset)
// - Speed control (0.25x, 0.5x, 1x, 2x, 4x, uncapped)
// - Performance monitoring (FPS, cycles, frame time)
// - Execution state (frames, instructions, uptime)

use super::DebugUI;
use crate::cpu::Cpu;
use crate::debug::{Debugger, StepMode};
use crate::ppu::Ppu;

/// Show the execution control panel
///
/// # Arguments
///
/// * `ui_state` - Debug UI state
/// * `ctx` - egui context
/// * `debugger` - Reference to the debugger
/// * `cpu` - Reference to the CPU
/// * `ppu` - Reference to the PPU
pub fn show(
    _ui_state: &mut DebugUI,
    ctx: &egui::Context,
    debugger: &mut Debugger,
    cpu: &Cpu,
    ppu: &Ppu,
) {
    egui::Window::new("Execution Control")
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.heading("Execution Control Panel");
            ui.separator();

            // Playback Controls Section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Playback Controls").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    // Play/Pause button
                    if debugger.is_paused() {
                        if ui.button("▶ Play").clicked() {
                            debugger.resume();
                        }
                    } else if ui.button("⏸ Pause").clicked() {
                        debugger.pause();
                    }

                    ui.separator();

                    // Step buttons
                    if ui
                        .button("Step Instruction")
                        .on_hover_text("Execute one CPU instruction (F10)")
                        .clicked()
                    {
                        debugger.step_instruction();
                    }

                    if ui
                        .button("Step Scanline")
                        .on_hover_text("Execute until next PPU scanline (F11)")
                        .clicked()
                    {
                        debugger.step_scanline(ppu);
                    }

                    if ui
                        .button("Step Frame")
                        .on_hover_text("Execute until next VBlank (F12)")
                        .clicked()
                    {
                        debugger.step_frame();
                    }
                });
            });

            ui.add_space(8.0);

            // Performance Monitoring Section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Performance Metrics").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.label(format!("{:.2}", debugger.metrics.fps));
                });

                ui.horizontal(|ui| {
                    ui.label("CPU Cycles/Frame:");
                    ui.label(format!("{}", debugger.metrics.cpu_cycles_per_frame));
                });

                ui.horizontal(|ui| {
                    ui.label("PPU Cycles/Frame:");
                    ui.label(format!("{}", debugger.metrics.ppu_cycles_per_frame));
                });

                ui.horizontal(|ui| {
                    ui.label("Current CPU Cycles:");
                    ui.label(format!("{}", cpu.cycles));
                });

                // Frame Time Information
                if !debugger.metrics.frame_times.is_empty() {
                    ui.separator();
                    let avg_frame_time = debugger
                        .metrics
                        .frame_times
                        .iter()
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .sum::<f64>()
                        / debugger.metrics.frame_times.len() as f64;

                    ui.horizontal(|ui| {
                        ui.label("Avg Frame Time:");
                        ui.label(format!("{:.2} ms", avg_frame_time));
                    });

                    let last_frame_time = debugger
                        .metrics
                        .frame_times
                        .last()
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0);

                    ui.horizontal(|ui| {
                        ui.label("Last Frame Time:");
                        ui.label(format!("{:.2} ms", last_frame_time));
                    });
                }
            });

            ui.add_space(8.0);

            // Execution State Section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Execution State").strong());
                ui.separator();

                // Status indicator
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    if debugger.is_paused() {
                        match debugger.step_mode() {
                            StepMode::None => {
                                ui.colored_label(egui::Color32::YELLOW, "⏸ Paused");
                            }
                            StepMode::Instruction => {
                                ui.colored_label(egui::Color32::YELLOW, "⏯ Stepping (Instruction)");
                            }
                            StepMode::Scanline => {
                                ui.colored_label(egui::Color32::YELLOW, "⏯ Stepping (Scanline)");
                            }
                            StepMode::Frame => {
                                ui.colored_label(egui::Color32::YELLOW, "⏯ Stepping (Frame)");
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "▶ Running");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Total Frames:");
                    ui.label(format!("{}", debugger.metrics.total_frames));
                });

                ui.horizontal(|ui| {
                    ui.label("Total Instructions:");
                    ui.label(format!("{}", debugger.metrics.total_instructions));
                });

                ui.horizontal(|ui| {
                    ui.label("Current Scanline:");
                    ui.label(format!("{}", ppu.scanline()));
                });

                ui.horizontal(|ui| {
                    ui.label("Current Cycle:");
                    ui.label(format!("{}", ppu.cycle()));
                });

                ui.horizontal(|ui| {
                    ui.label("Uptime:");
                    ui.label(debugger.metrics.uptime_string());
                });
            });

            ui.add_space(8.0);

            // Hot Keys Display Section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Keyboard Shortcuts").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("P:");
                    ui.label("Pause/Resume");
                });

                ui.horizontal(|ui| {
                    ui.label("F10:");
                    ui.label("Step Instruction");
                });

                ui.horizontal(|ui| {
                    ui.label("F11:");
                    ui.label("Step Scanline");
                });

                ui.horizontal(|ui| {
                    ui.label("F12:");
                    ui.label("Step Frame");
                });
            });
        });
}
