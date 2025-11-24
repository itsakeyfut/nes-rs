// Pattern Table Viewer

use crate::display::palette::palette_to_rgb;
use crate::ppu::Ppu;

/// Render both pattern tables with active table indicators
pub(super) fn render_tables(ui: &mut egui::Ui, ppu: &Ppu, ppuctrl: u8) {
    ui.horizontal(|ui| {
        // Left pattern table ($0000-$0FFF)
        ui.vertical(|ui| {
            ui.label("Left ($0000-$0FFF)");
            render_single_table(ui, ppu, 0x0000);
        });

        ui.add_space(10.0);

        // Right pattern table ($1000-$1FFF)
        ui.vertical(|ui| {
            ui.label("Right ($1000-$1FFF)");
            render_single_table(ui, ppu, 0x1000);
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
fn render_single_table(ui: &mut egui::Ui, ppu: &Ppu, base_addr: u16) {
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
