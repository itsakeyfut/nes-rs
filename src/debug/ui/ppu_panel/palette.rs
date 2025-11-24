// Palette Viewer

use crate::display::palette::palette_to_rgb;
use crate::ppu::Ppu;

/// Render palette viewer with visual color swatches
pub(super) fn render(ui: &mut egui::Ui, ppu: &Ppu) {
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
