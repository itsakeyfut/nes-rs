// PPU rendering logic

use super::constants::{NAMETABLE_HEIGHT, NAMETABLE_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use super::Ppu;

impl Ppu {
    /// Render the background to the frame buffer
    ///
    /// This method renders the entire 256x240 pixel background based on:
    /// - Nametable data (tile indices)
    /// - Attribute table data (palette selection)
    /// - Pattern table data (tile graphics)
    /// - Scroll position (from internal registers)
    ///
    /// The rendering respects the current scroll position (X and Y).
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    /// ppu.render_background();
    /// let frame = ppu.frame();
    /// // frame now contains the rendered background
    /// ```
    pub fn render_background(&mut self) {
        // Check if background rendering is enabled
        if (self.ppumask & 0x08) == 0 {
            // Background rendering is disabled, clear the frame buffer
            self.frame_buffer.fill(0);
            return;
        }

        // Get scroll position from internal registers (coarse/fine scroll + base nametable)
        let coarse_x = (self.t & 0x001F) as usize;
        let coarse_y = ((self.t & 0x03E0) >> 5) as usize;
        let fine_x = self.fine_x as usize;
        let fine_y = ((self.t >> 12) & 0x07) as usize;

        let nametable_select = ((self.t >> 10) & 0x03) as usize;
        let base_nt_x = nametable_select & 0x01;
        let base_nt_y = (nametable_select >> 1) & 0x01;

        let scroll_x = base_nt_x * NAMETABLE_WIDTH * TILE_SIZE + coarse_x * TILE_SIZE + fine_x;
        let scroll_y = base_nt_y * NAMETABLE_HEIGHT * TILE_SIZE + coarse_y * TILE_SIZE + fine_y;

        // Render each pixel on the screen
        for screen_y in 0..SCREEN_HEIGHT {
            for screen_x in 0..SCREEN_WIDTH {
                // Calculate the position in the nametable with scrolling
                let nt_x = (screen_x + scroll_x) % (NAMETABLE_WIDTH * TILE_SIZE * 2);
                let nt_y = (screen_y + scroll_y) % (NAMETABLE_HEIGHT * TILE_SIZE * 2);

                // Determine which nametable to use based on position
                let nt_index = (nt_y / (NAMETABLE_HEIGHT * TILE_SIZE)) * 2
                    + (nt_x / (NAMETABLE_WIDTH * TILE_SIZE));
                let nametable_addr = 0x2000 | ((nt_index as u16) << 10);

                // Calculate tile coordinates within the nametable
                let tile_x = (nt_x % (NAMETABLE_WIDTH * TILE_SIZE)) / TILE_SIZE;
                let tile_y = (nt_y % (NAMETABLE_HEIGHT * TILE_SIZE)) / TILE_SIZE;

                // Calculate pixel position within the tile
                let pixel_x = nt_x % TILE_SIZE;
                let pixel_y = nt_y % TILE_SIZE;

                // Read tile index from nametable
                let tile_addr = nametable_addr + (tile_y * NAMETABLE_WIDTH + tile_x) as u16;
                let tile_index = self.read_nametable_tile(tile_addr);

                // Read attribute byte for palette selection
                let palette_index = self.read_attribute_byte(nametable_addr, tile_x, tile_y);

                // Fetch tile pixel from pattern table
                let pattern_table_base = if (self.ppuctrl & 0x10) != 0 {
                    0x1000
                } else {
                    0x0000
                };
                let color_index =
                    self.fetch_tile_pixel(pattern_table_base, tile_index, pixel_x, pixel_y);

                // Get final palette color
                let palette_color = self.get_background_color(palette_index, color_index);

                // Write to frame buffer
                let buffer_index = screen_y * SCREEN_WIDTH + screen_x;
                self.frame_buffer[buffer_index] = palette_color;
            }
        }
    }

    /// Read a tile index from the nametable
    ///
    /// # Arguments
    ///
    /// * `addr` - Nametable address ($2000-$2FFF)
    ///
    /// # Returns
    ///
    /// The tile index (0-255)
    pub(super) fn read_nametable_tile(&self, addr: u16) -> u8 {
        self.read_ppu_memory(addr)
    }

    /// Read attribute byte for palette selection
    ///
    /// The attribute table covers 2x2 tile blocks, with each byte containing
    /// palette information for four 2x2 tile blocks.
    ///
    /// # Arguments
    ///
    /// * `nametable_base` - Base address of the nametable ($2000, $2400, $2800, or $2C00)
    /// * `tile_x` - Tile X coordinate (0-31)
    /// * `tile_y` - Tile Y coordinate (0-29)
    ///
    /// # Returns
    ///
    /// The palette index (0-3) for the specified tile
    pub(super) fn read_attribute_byte(
        &self,
        nametable_base: u16,
        tile_x: usize,
        tile_y: usize,
    ) -> u8 {
        // Attribute table starts at nametable_base + 0x3C0
        let attr_table_base = nametable_base + 0x3C0;

        // Each attribute byte covers a 4x4 tile area (2x2 blocks of 2x2 tiles)
        let attr_x = tile_x / 4;
        let attr_y = tile_y / 4;
        let attr_addr = attr_table_base + (attr_y * 8 + attr_x) as u16;

        let attr_byte = self.read_ppu_memory(attr_addr);

        // Determine which 2x2 tile block within the 4x4 area
        let block_x = (tile_x % 4) / 2;
        let block_y = (tile_y % 4) / 2;
        let shift = (block_y * 2 + block_x) * 2;

        // Extract 2-bit palette index
        (attr_byte >> shift) & 0x03
    }

    /// Fetch a pixel color index from the pattern table
    ///
    /// Each tile is 8x8 pixels stored as two bitplanes (16 bytes total).
    /// The two bitplanes are combined to form a 2-bit color index.
    ///
    /// # Arguments
    ///
    /// * `pattern_table_base` - Base address of pattern table ($0000 or $1000)
    /// * `tile_index` - Tile index (0-255)
    /// * `pixel_x` - Pixel X coordinate within tile (0-7)
    /// * `pixel_y` - Pixel Y coordinate within tile (0-7)
    ///
    /// # Returns
    ///
    /// The 2-bit color index (0-3) for the pixel
    pub(super) fn fetch_tile_pixel(
        &self,
        pattern_table_base: u16,
        tile_index: u8,
        pixel_x: usize,
        pixel_y: usize,
    ) -> u8 {
        // Each tile is 16 bytes (8 bytes per bitplane)
        let tile_addr = pattern_table_base + (tile_index as u16) * 16;

        // Read the two bitplanes for this row
        let bitplane_0 = self.read_ppu_memory(tile_addr + pixel_y as u16);
        let bitplane_1 = self.read_ppu_memory(tile_addr + pixel_y as u16 + 8);

        // Extract the bit for this pixel (MSB is leftmost pixel)
        let bit_pos = 7 - pixel_x;
        let bit_0 = (bitplane_0 >> bit_pos) & 0x01;
        let bit_1 = (bitplane_1 >> bit_pos) & 0x01;

        // Combine bits to form 2-bit color index
        (bit_1 << 1) | bit_0
    }

    /// Get the final background color from palette RAM
    ///
    /// # Arguments
    ///
    /// * `palette_index` - Palette index (0-3) from attribute table
    /// * `color_index` - Color index (0-3) from pattern table
    ///
    /// # Returns
    ///
    /// The palette color index (0-63) to be displayed
    pub(super) fn get_background_color(&self, palette_index: u8, color_index: u8) -> u8 {
        // If color index is 0, use the universal background color
        if color_index == 0 {
            return self.palette_ram[0];
        }

        // Calculate palette RAM address
        // Background palettes are at $3F00-$3F0F
        let palette_addr = (palette_index as usize) * 4 + (color_index as usize);
        self.palette_ram[palette_addr]
    }
}
