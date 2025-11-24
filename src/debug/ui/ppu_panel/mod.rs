// PPU Debugger Panel

mod nametable;
mod palette;
mod pattern;

use super::DebugUI;
use crate::debug::Debugger;
use crate::ppu::Ppu;

/// Show the PPU debugger panel
pub(super) fn show(ui_state: &mut DebugUI, ctx: &egui::Context, debugger: &Debugger, ppu: &Ppu) {
    let mut show_panel = ui_state.show_ppu_panel;

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
            render_ppuctrl(ui, state.ppuctrl);

            ui.add_space(5.0);

            // PPUMASK ($2001)
            render_ppumask(ui, state.ppumask);

            ui.add_space(5.0);

            // PPUSTATUS ($2002)
            render_ppustatus(ui, state.ppustatus);

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

            palette::render(ui, ppu);

            ui.add_space(10.0);

            // Pattern Table Viewer
            ui.heading("Pattern Tables");
            ui.separator();

            pattern::render_tables(ui, ppu, state.ppuctrl);

            ui.add_space(10.0);

            // Nametable Viewer
            ui.heading("Nametables");
            ui.separator();

            nametable::render_all(ui, ppu);
        });

    ui_state.show_ppu_panel = show_panel;
}

/// Render PPUCTRL register with bit breakdown
fn render_ppuctrl(ui: &mut egui::Ui, ppuctrl: u8) {
    ui.label(egui::RichText::new("PPUCTRL ($2000)").strong());
    ui.horizontal(|ui| {
        ui.monospace(format!("${:02X}", ppuctrl));
        ui.label("|");
        ui.monospace(format!("{:08b}", ppuctrl));
    });
    ui.indent("ppuctrl_bits", |ui| {
        ui.horizontal(|ui| {
            ui.label("Bit 7:");
            let nmi_enabled = (ppuctrl & 0x80) != 0;
            if nmi_enabled {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "NMI Enabled");
            } else {
                ui.colored_label(egui::Color32::GRAY, "NMI Disabled");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 5:");
            if (ppuctrl & 0x20) != 0 {
                ui.label("Sprites 8x16");
            } else {
                ui.label("Sprites 8x8");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 4:");
            if (ppuctrl & 0x10) != 0 {
                ui.label("BG pattern table $1000");
            } else {
                ui.label("BG pattern table $0000");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 3:");
            if (ppuctrl & 0x08) != 0 {
                ui.label("Sprite pattern table $1000");
            } else {
                ui.label("Sprite pattern table $0000");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 2:");
            if (ppuctrl & 0x04) != 0 {
                ui.label("VRAM increment +32 (down)");
            } else {
                ui.label("VRAM increment +1 (across)");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bits 1-0:");
            let nametable = ppuctrl & 0x03;
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
}

/// Render PPUMASK register with bit breakdown
fn render_ppumask(ui: &mut egui::Ui, ppumask: u8) {
    ui.label(egui::RichText::new("PPUMASK ($2001)").strong());
    ui.horizontal(|ui| {
        ui.monospace(format!("${:02X}", ppumask));
        ui.label("|");
        ui.monospace(format!("{:08b}", ppumask));
    });
    ui.indent("ppumask_bits", |ui| {
        ui.horizontal(|ui| {
            ui.label("Bit 7:");
            if (ppumask & 0x80) != 0 {
                ui.label("Emphasize blue");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 6:");
            if (ppumask & 0x40) != 0 {
                ui.label("Emphasize green");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 5:");
            if (ppumask & 0x20) != 0 {
                ui.label("Emphasize red");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 4:");
            let show_sprites = (ppumask & 0x10) != 0;
            if show_sprites {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Show sprites");
            } else {
                ui.colored_label(egui::Color32::GRAY, "Hide sprites");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 3:");
            let show_bg = (ppumask & 0x08) != 0;
            if show_bg {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Show background");
            } else {
                ui.colored_label(egui::Color32::GRAY, "Hide background");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 2:");
            if (ppumask & 0x04) != 0 {
                ui.label("Show sprites in leftmost 8 pixels");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 1:");
            if (ppumask & 0x02) != 0 {
                ui.label("Show background in leftmost 8 pixels");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 0:");
            if (ppumask & 0x01) != 0 {
                ui.label("Grayscale mode");
            } else {
                ui.label("-");
            }
        });
    });
}

/// Render PPUSTATUS register with bit breakdown
fn render_ppustatus(ui: &mut egui::Ui, ppustatus: u8) {
    ui.label(egui::RichText::new("PPUSTATUS ($2002)").strong());
    ui.horizontal(|ui| {
        ui.monospace(format!("${:02X}", ppustatus));
        ui.label("|");
        ui.monospace(format!("{:08b}", ppustatus));
    });
    ui.indent("ppustatus_bits", |ui| {
        ui.horizontal(|ui| {
            ui.label("Bit 7:");
            let vblank = (ppustatus & 0x80) != 0;
            if vblank {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "VBlank");
            } else {
                ui.colored_label(egui::Color32::GRAY, "No VBlank");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 6:");
            let sprite0_hit = (ppustatus & 0x40) != 0;
            if sprite0_hit {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "Sprite 0 hit");
            } else {
                ui.label("-");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Bit 5:");
            let sprite_overflow = (ppustatus & 0x20) != 0;
            if sprite_overflow {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 0), "Sprite overflow");
            } else {
                ui.label("-");
            }
        });
    });
}
