// Nametable Viewer

use crate::ppu::Ppu;

/// Render all 4 nametables in a 2x2 grid
pub(super) fn render_all(ui: &mut egui::Ui, ppu: &Ppu) {
    ui.label("Nametable viewer - displaying 4 nametables");

    ui.horizontal(|ui| {
        // Top row: Nametable 0 and 1
        ui.vertical(|ui| {
            ui.label("Nametable 0 ($2000)");
            render_single(ui, ppu, 0x2000);
        });

        ui.add_space(10.0);

        ui.vertical(|ui| {
            ui.label("Nametable 1 ($2400)");
            render_single(ui, ppu, 0x2400);
        });
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        // Bottom row: Nametable 2 and 3
        ui.vertical(|ui| {
            ui.label("Nametable 2 ($2800)");
            render_single(ui, ppu, 0x2800);
        });

        ui.add_space(10.0);

        ui.vertical(|ui| {
            ui.label("Nametable 3 ($2C00)");
            render_single(ui, ppu, 0x2C00);
        });
    });
}

/// Render a single nametable
fn render_single(ui: &mut egui::Ui, ppu: &Ppu, base_addr: u16) {
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
