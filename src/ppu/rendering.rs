// PPU rendering logic

use super::constants::{NAMETABLE_HEIGHT, NAMETABLE_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use super::Ppu;

/// Represents the current phase of the background tile fetch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileFetchPhase {
    /// Fetching nametable byte (tile index)
    Nametable,
    /// Fetching attribute byte (palette selection)
    Attribute,
    /// Fetching pattern table low bitplane
    PatternLow,
    /// Fetching pattern table high bitplane
    PatternHigh,
}

/// Represents a parsed sprite from OAM
#[derive(Debug, Clone, Copy)]
struct Sprite {
    /// Y position (top edge - 1)
    y: u8,
    /// Tile index (or tile bank for 8x16 mode)
    tile_index: u8,
    /// Attribute byte
    attributes: u8,
    /// X position (left edge)
    x: u8,
    /// Original OAM index (for sprite 0 detection)
    oam_index: usize,
}

impl Sprite {
    /// Check if sprite has vertical flip enabled
    fn is_vflip(&self) -> bool {
        (self.attributes & 0x80) != 0
    }

    /// Check if sprite has horizontal flip enabled
    fn is_hflip(&self) -> bool {
        (self.attributes & 0x40) != 0
    }

    /// Check if sprite is behind background
    fn is_behind_background(&self) -> bool {
        (self.attributes & 0x20) != 0
    }

    /// Get sprite palette index (0-3, for sprite palettes 4-7)
    fn palette(&self) -> u8 {
        self.attributes & 0x03
    }

    /// Check if this is sprite 0
    fn is_sprite_zero(&self) -> bool {
        self.oam_index == 0
    }
}

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

    /// Parse all 64 sprites from OAM memory
    ///
    /// Reads the OAM memory and creates Sprite structures for all 64 sprites.
    ///
    /// # Returns
    ///
    /// Array of 64 sprites parsed from OAM
    fn parse_sprites(&self) -> [Sprite; 64] {
        let mut sprites = [Sprite {
            y: 0xFF,
            tile_index: 0,
            attributes: 0,
            x: 0xFF,
            oam_index: 0,
        }; 64];

        for (i, sprite) in sprites.iter_mut().enumerate() {
            let base = i * 4;
            *sprite = Sprite {
                y: self.oam[base],
                tile_index: self.oam[base + 1],
                attributes: self.oam[base + 2],
                x: self.oam[base + 3],
                oam_index: i,
            };
        }

        sprites
    }

    /// Evaluate sprites for a specific scanline
    ///
    /// This implements the sprite evaluation logic that determines which sprites
    /// should be rendered on a given scanline. The NES PPU can only render up to
    /// 8 sprites per scanline.
    ///
    /// # Arguments
    ///
    /// * `scanline` - The scanline to evaluate (0-239)
    /// * `sprites` - Array of all 64 sprites from OAM
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Vector of sprites visible on this scanline (max 8)
    /// - Whether sprite overflow occurred (more than 8 sprites on this line)
    fn evaluate_sprites_for_scanline(
        &self,
        scanline: usize,
        sprites: &[Sprite; 64],
    ) -> (Vec<Sprite>, bool) {
        let mut visible_sprites = Vec::with_capacity(8);
        let sprite_height = self.get_sprite_height();

        for sprite in sprites.iter() {
            let sprite_y = sprite.y as usize + 1; // Y position is top - 1

            // Check if sprite is visible on this scanline
            if scanline >= sprite_y && scanline < sprite_y + sprite_height {
                if visible_sprites.len() < 8 {
                    visible_sprites.push(*sprite);
                } else {
                    // More than 8 sprites on this scanline - overflow
                    return (visible_sprites, true);
                }
            }
        }

        (visible_sprites, false)
    }

    /// Get the sprite height based on PPUCTRL settings
    ///
    /// # Returns
    ///
    /// Sprite height in pixels (8 for 8x8 mode, 16 for 8x16 mode)
    fn get_sprite_height(&self) -> usize {
        if (self.ppuctrl & 0x20) != 0 {
            16 // 8x16 mode
        } else {
            8 // 8x8 mode
        }
    }

    /// Fetch a pixel from a sprite tile
    ///
    /// # Arguments
    ///
    /// * `sprite` - The sprite to fetch from
    /// * `pixel_x` - X coordinate within the sprite (0-7)
    /// * `pixel_y` - Y coordinate within the sprite (0-7 for 8x8, 0-15 for 8x16)
    ///
    /// # Returns
    ///
    /// The 2-bit color index (0-3) for the pixel
    fn fetch_sprite_pixel(&self, sprite: &Sprite, pixel_x: usize, pixel_y: usize) -> u8 {
        let sprite_height = self.get_sprite_height();

        // Apply vertical flip
        let pixel_y = if sprite.is_vflip() {
            sprite_height - 1 - pixel_y
        } else {
            pixel_y
        };

        // Apply horizontal flip
        let pixel_x = if sprite.is_hflip() {
            7 - pixel_x
        } else {
            pixel_x
        };

        if sprite_height == 8 {
            // 8x8 sprite mode
            let pattern_table_base = if (self.ppuctrl & 0x08) != 0 {
                0x1000
            } else {
                0x0000
            };
            self.fetch_tile_pixel(pattern_table_base, sprite.tile_index, pixel_x, pixel_y)
        } else {
            // 8x16 sprite mode
            // In 8x16 mode, bit 0 of tile_index selects pattern table
            // and bits 1-7 select the tile pair
            let pattern_table_base = if (sprite.tile_index & 0x01) != 0 {
                0x1000
            } else {
                0x0000
            };

            let tile_pair = sprite.tile_index & 0xFE;
            let (tile_index, tile_y) = if pixel_y < 8 {
                // Top half
                (tile_pair, pixel_y)
            } else {
                // Bottom half
                (tile_pair + 1, pixel_y - 8)
            };

            self.fetch_tile_pixel(pattern_table_base, tile_index, pixel_x, tile_y)
        }
    }

    /// Get the final sprite color from palette RAM
    ///
    /// # Arguments
    ///
    /// * `palette_index` - Palette index (0-3) for sprite palettes
    /// * `color_index` - Color index (0-3) from pattern table
    ///
    /// # Returns
    ///
    /// The palette color index (0-63) to be displayed
    fn get_sprite_color(&self, palette_index: u8, color_index: u8) -> u8 {
        // If color index is 0, sprite pixel is transparent
        if color_index == 0 {
            return 0; // Return 0 to indicate transparency
        }

        // Sprite palettes are at $3F10-$3F1F (offset by 16 from background)
        let palette_addr = 16 + (palette_index as usize) * 4 + (color_index as usize);
        self.palette_ram[palette_addr]
    }

    /// Render sprites to the frame buffer
    ///
    /// This method renders all visible sprites on the screen, respecting:
    /// - Sprite priority (behind/in front of background)
    /// - 8 sprites per scanline limit
    /// - Sprite 0 hit detection
    /// - Horizontal and vertical flipping
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    /// ppu.render_sprites();
    /// let frame = ppu.frame();
    /// // frame now contains the rendered sprites
    /// ```
    pub fn render_sprites(&mut self) {
        // Check if sprite rendering is enabled
        if (self.ppumask & 0x10) == 0 {
            // Sprite rendering is disabled
            return;
        }

        // Parse all sprites from OAM
        let sprites = self.parse_sprites();

        // Clear sprite 0 hit flag at the start of rendering
        let mut sprite_0_hit = false;
        let mut sprite_overflow_occurred = false;

        // Render each scanline
        for scanline in 0..SCREEN_HEIGHT {
            // Evaluate which sprites are visible on this scanline
            let (visible_sprites, overflow) =
                self.evaluate_sprites_for_scanline(scanline, &sprites);

            if overflow {
                sprite_overflow_occurred = true;
            }

            // Render sprites in reverse priority order (lower index = higher priority)
            for sprite in visible_sprites.iter().rev() {
                let sprite_y = sprite.y as usize + 1;
                let pixel_y = scanline - sprite_y;

                // Render each pixel of the sprite on this scanline
                for pixel_x in 0..8 {
                    let screen_x = sprite.x as usize + pixel_x;

                    // Check if pixel is within screen bounds
                    if screen_x >= SCREEN_WIDTH {
                        continue;
                    }

                    // Fetch sprite pixel color
                    let color_index = self.fetch_sprite_pixel(sprite, pixel_x, pixel_y);

                    // Skip transparent pixels (color 0)
                    if color_index == 0 {
                        continue;
                    }

                    let sprite_color = self.get_sprite_color(sprite.palette(), color_index);
                    let buffer_index = scanline * SCREEN_WIDTH + screen_x;
                    let background_color = self.frame_buffer[buffer_index];

                    // Check for sprite 0 hit
                    if sprite.is_sprite_zero()
                        && background_color != self.palette_ram[0]
                        && screen_x != 255
                    {
                        sprite_0_hit = true;
                    }

                    // Handle sprite priority
                    if sprite.is_behind_background() {
                        // Sprite is behind background
                        // Only draw if background pixel is transparent (universal bg color)
                        if background_color == self.palette_ram[0] {
                            self.frame_buffer[buffer_index] = sprite_color;
                        }
                    } else {
                        // Sprite is in front of background
                        self.frame_buffer[buffer_index] = sprite_color;
                    }
                }
            }
        }

        // Update PPUSTATUS flags
        if sprite_0_hit {
            self.ppustatus |= 0x40; // Set sprite 0 hit flag
        }
        if sprite_overflow_occurred {
            self.ppustatus |= 0x20; // Set sprite overflow flag
        }
    }

    /// Render a complete frame (background + sprites)
    ///
    /// This is a convenience method that renders both background and sprites
    /// in the correct order.
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    /// ppu.render_frame();
    /// let frame = ppu.frame();
    /// ```
    pub fn render_frame(&mut self) {
        // Clear sprite flags at the start of frame
        self.ppustatus &= !(0x40 | 0x20); // Clear sprite 0 hit and overflow flags

        // Render background first
        self.render_background();

        // Then render sprites on top
        self.render_sprites();
    }

    // ========================================
    // Scanline-based Rendering Methods
    // ========================================

    /// Determine which phase of tile fetching we're in based on the cycle
    ///
    /// The tile fetch happens in 8-cycle intervals with 4 phases:
    /// - Cycles 0-1: Nametable byte
    /// - Cycles 2-3: Attribute byte
    /// - Cycles 4-5: Pattern table low
    /// - Cycles 6-7: Pattern table high
    fn get_tile_fetch_phase(&self, cycle: u16) -> TileFetchPhase {
        match cycle % 8 {
            0 | 1 => TileFetchPhase::Nametable,
            2 | 3 => TileFetchPhase::Attribute,
            4 | 5 => TileFetchPhase::PatternLow,
            6 | 7 => TileFetchPhase::PatternHigh,
            _ => unreachable!(),
        }
    }

    /// Fetch the nametable byte for the current tile
    ///
    /// Uses the v register to determine which tile to fetch.
    fn fetch_nametable_byte(&mut self) {
        // v register layout: yyy NN YYYYY XXXXX
        // Nametable address = 0x2000 | (v & 0x0FFF)
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.bg_nametable_byte = self.read_ppu_memory(addr);
    }

    /// Fetch the attribute byte for the current tile
    ///
    /// Uses the v register to determine which attribute byte to fetch.
    fn fetch_attribute_byte(&mut self) {
        // v register layout: yyy NN YYYYY XXXXX
        // Attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
        let v = self.v;
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let attr_byte = self.read_ppu_memory(addr);

        // Extract the 2-bit palette index based on the current tile position
        // The attribute byte covers a 4x4 tile area (2x2 blocks of 2x2 tiles)
        let coarse_x = v & 0x001F;
        let coarse_y = (v >> 5) & 0x001F;

        // Determine which 2x2 tile block within the 4x4 area
        let shift = ((coarse_y & 0x02) << 1) | (coarse_x & 0x02);
        self.bg_attribute_byte = (attr_byte >> shift) & 0x03;
    }

    /// Fetch the pattern table low bitplane byte for the current tile
    fn fetch_pattern_low_byte(&mut self) {
        // Pattern table address based on PPUCTRL bit 4
        let pattern_table_base = if (self.ppuctrl & 0x10) != 0 {
            0x1000
        } else {
            0x0000
        };

        // Fine Y scroll from v register (bits 12-14)
        let fine_y = (self.v >> 12) & 0x07;

        // Tile address = pattern_table_base + tile_index * 16 + fine_y
        let addr = pattern_table_base + (self.bg_nametable_byte as u16) * 16 + fine_y;
        self.bg_pattern_low = self.read_ppu_memory(addr);
    }

    /// Fetch the pattern table high bitplane byte for the current tile
    fn fetch_pattern_high_byte(&mut self) {
        // Pattern table address based on PPUCTRL bit 4
        let pattern_table_base = if (self.ppuctrl & 0x10) != 0 {
            0x1000
        } else {
            0x0000
        };

        // Fine Y scroll from v register (bits 12-14)
        let fine_y = (self.v >> 12) & 0x07;

        // Tile address = pattern_table_base + tile_index * 16 + fine_y + 8 (high bitplane)
        let addr = pattern_table_base + (self.bg_nametable_byte as u16) * 16 + fine_y + 8;
        self.bg_pattern_high = self.read_ppu_memory(addr);
    }

    /// Perform background tile fetch based on the current cycle
    ///
    /// This is called during rendering cycles to fetch tile data in the pipeline.
    pub(super) fn perform_background_fetch(&mut self, cycle: u16) {
        let phase = self.get_tile_fetch_phase(cycle);

        // Perform the appropriate fetch on odd cycles (the second cycle of each phase)
        if (cycle & 1) == 1 {
            match phase {
                TileFetchPhase::Nametable => self.fetch_nametable_byte(),
                TileFetchPhase::Attribute => self.fetch_attribute_byte(),
                TileFetchPhase::PatternLow => self.fetch_pattern_low_byte(),
                TileFetchPhase::PatternHigh => self.fetch_pattern_high_byte(),
            }
        }
    }

    /// Load the fetched tile data into the shift registers
    ///
    /// This is called every 8 cycles after a complete tile has been fetched.
    pub(super) fn load_shift_registers(&mut self) {
        // Load the pattern data into the low 8 bits of the shift registers
        self.bg_pattern_shift_low =
            (self.bg_pattern_shift_low & 0xFF00) | (self.bg_pattern_low as u16);
        self.bg_pattern_shift_high =
            (self.bg_pattern_shift_high & 0xFF00) | (self.bg_pattern_high as u16);

        // Load the attribute data (extend 2 bits to 8 bits)
        let attr_low = if (self.bg_attribute_byte & 0x01) != 0 {
            0xFF
        } else {
            0x00
        };
        let attr_high = if (self.bg_attribute_byte & 0x02) != 0 {
            0xFF
        } else {
            0x00
        };

        self.bg_attribute_shift_low = (self.bg_attribute_shift_low & 0xFF00) | (attr_low as u16);
        self.bg_attribute_shift_high = (self.bg_attribute_shift_high & 0xFF00) | (attr_high as u16);
    }

    /// Shift the background shift registers by 1 pixel
    ///
    /// This is called every cycle during rendering to advance to the next pixel.
    pub(super) fn shift_background_registers(&mut self) {
        // Shift pattern registers left by 1
        self.bg_pattern_shift_low <<= 1;
        self.bg_pattern_shift_high <<= 1;

        // Shift attribute registers left by 1
        self.bg_attribute_shift_low <<= 1;
        self.bg_attribute_shift_high <<= 1;
    }

    /// Get the background color index from the shift registers
    ///
    /// Returns the 2-bit color index (0-3) from the pattern data, without
    /// applying palette lookup. This is used for sprite 0 hit detection.
    pub(super) fn get_background_color_index(&self) -> u8 {
        let bit_position = 15 - self.fine_x;
        let bit_0 = (self.bg_pattern_shift_low >> bit_position) & 0x01;
        let bit_1 = (self.bg_pattern_shift_high >> bit_position) & 0x01;
        ((bit_1 << 1) | bit_0) as u8
    }

    /// Get the current background pixel color from the shift registers
    ///
    /// Returns the palette index (0-63) for the current pixel.
    pub(super) fn get_background_pixel(&self) -> u8 {
        // Get the bit at position (15 - fine_x) from each shift register
        let bit_position = 15 - self.fine_x;

        let bit_0 = (self.bg_pattern_shift_low >> bit_position) & 0x01;
        let bit_1 = (self.bg_pattern_shift_high >> bit_position) & 0x01;
        let color_index = ((bit_1 << 1) | bit_0) as u8;

        let attr_0 = (self.bg_attribute_shift_low >> bit_position) & 0x01;
        let attr_1 = (self.bg_attribute_shift_high >> bit_position) & 0x01;
        let palette_index = ((attr_1 << 1) | attr_0) as u8;

        // Get the final color from palette RAM
        self.get_background_color(palette_index, color_index)
    }

    /// Increment the coarse X scroll in the v register
    ///
    /// This is called after rendering each tile (every 8 pixels).
    pub(super) fn increment_scroll_x(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        // Increment coarse X (bits 0-4)
        if (self.v & 0x001F) == 31 {
            // Coarse X wraps to 0
            self.v &= !0x001F;
            // Switch horizontal nametable
            self.v ^= 0x0400;
        } else {
            // Increment coarse X
            self.v += 1;
        }
    }

    /// Increment the Y scroll in the v register
    ///
    /// This is called at the end of each visible scanline (dot 256).
    pub(super) fn increment_scroll_y(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        // Increment fine Y (bits 12-14)
        if (self.v & 0x7000) != 0x7000 {
            // Increment fine Y
            self.v += 0x1000;
        } else {
            // Fine Y wraps to 0
            self.v &= !0x7000;

            // Increment coarse Y
            let mut coarse_y = (self.v >> 5) & 0x1F;
            if coarse_y == 29 {
                // Coarse Y wraps to 0
                coarse_y = 0;
                // Switch vertical nametable
                self.v ^= 0x0800;
            } else if coarse_y == 31 {
                // Coarse Y wraps to 0 (without switching nametable)
                coarse_y = 0;
            } else {
                // Increment coarse Y
                coarse_y += 1;
            }

            // Update v with new coarse Y
            self.v = (self.v & !0x03E0) | (coarse_y << 5);
        }
    }

    /// Copy horizontal scroll bits from t to v
    ///
    /// This is called at dot 257 of each scanline.
    pub(super) fn copy_horizontal_scroll(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        // Copy bits 0-4 (coarse X) and bit 10 (horizontal nametable) from t to v
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    /// Copy vertical scroll bits from t to v
    ///
    /// This is called at dot 280-304 of the pre-render scanline.
    pub(super) fn copy_vertical_scroll(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        // Copy bits 5-9 (coarse Y), bits 12-14 (fine Y), and bit 11 (vertical nametable) from t to v
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    /// Evaluate sprites for the next scanline
    ///
    /// This scans through all 64 sprites in OAM and finds up to 8 sprites
    /// that will be visible on the next scanline. The results are stored
    /// in secondary OAM.
    ///
    /// This also sets the sprite overflow flag if more than 8 sprites are found.
    pub(super) fn evaluate_sprites_for_next_scanline(&mut self) {
        // The next scanline is the current scanline + 1
        let next_scanline = self.scanline + 1;

        // Skip if we're past visible scanlines
        if next_scanline >= SCREEN_HEIGHT as u16 {
            self.sprite_count = 0;
            self.sprite_0_present = false;
            return;
        }

        let sprite_height = self.get_sprite_height();
        let mut count = 0;
        let mut overflow = false;

        self.sprite_0_present = false;

        // Scan through all 64 sprites in OAM
        for i in 0..64 {
            let base = i * 4;
            let sprite_y = self.oam[base] as u16 + 1; // Y position is top - 1

            // Check if sprite is visible on the next scanline
            if next_scanline >= sprite_y && next_scanline < sprite_y + sprite_height as u16 {
                if count < 8 {
                    // Add sprite to secondary OAM
                    self.secondary_oam[count] = (
                        self.oam[base],     // Y position
                        self.oam[base + 1], // Tile index
                        self.oam[base + 2], // Attributes
                        self.oam[base + 3], // X position
                    );

                    // Check if this is sprite 0
                    if i == 0 {
                        self.sprite_0_present = true;
                    }

                    count += 1;
                } else {
                    // More than 8 sprites on this scanline - set overflow flag
                    overflow = true;
                    break;
                }
            }
        }

        self.sprite_count = count;

        // Update sprite overflow flag
        if overflow {
            self.ppustatus |= 0x20;
        } else {
            self.ppustatus &= !0x20;
        }

        // Load sprite shift registers for the next scanline
        self.load_sprite_shift_registers(next_scanline);
    }

    /// Load sprite pattern data into shift registers
    ///
    /// This fetches the pattern data for all sprites in secondary OAM
    /// and loads them into the sprite shift registers.
    fn load_sprite_shift_registers(&mut self, scanline: u16) {
        for i in 0..8 {
            if i < self.sprite_count {
                let (sprite_y, tile_index, attributes, x_pos) = self.secondary_oam[i];

                // Calculate which row of the sprite we're rendering
                let sprite_y = sprite_y as u16 + 1;
                let row = (scanline - sprite_y) as usize;

                // Fetch sprite pixel data for this row
                let sprite_height = self.get_sprite_height();

                // Apply vertical flip
                let row = if (attributes & 0x80) != 0 {
                    sprite_height - 1 - row
                } else {
                    row
                };

                // Fetch pattern data based on sprite size
                let (pattern_low, pattern_high) = if sprite_height == 8 {
                    // 8x8 sprite mode
                    let pattern_table_base = if (self.ppuctrl & 0x08) != 0 {
                        0x1000
                    } else {
                        0x0000
                    };

                    let tile_addr = pattern_table_base + (tile_index as u16) * 16;
                    let low = self.read_ppu_memory(tile_addr + row as u16);
                    let high = self.read_ppu_memory(tile_addr + row as u16 + 8);
                    (low, high)
                } else {
                    // 8x16 sprite mode
                    let pattern_table_base = if (tile_index & 0x01) != 0 {
                        0x1000
                    } else {
                        0x0000
                    };

                    let tile_pair = tile_index & 0xFE;
                    let (tile, tile_row) = if row < 8 {
                        (tile_pair, row)
                    } else {
                        (tile_pair + 1, row - 8)
                    };

                    let tile_addr = pattern_table_base + (tile as u16) * 16;
                    let low = self.read_ppu_memory(tile_addr + tile_row as u16);
                    let high = self.read_ppu_memory(tile_addr + tile_row as u16 + 8);
                    (low, high)
                };

                // Apply horizontal flip if needed
                let (pattern_low, pattern_high) = if (attributes & 0x40) != 0 {
                    // Flip horizontally by reversing the bits
                    (pattern_low.reverse_bits(), pattern_high.reverse_bits())
                } else {
                    (pattern_low, pattern_high)
                };

                // Load into shift registers
                self.sprite_pattern_shift_low[i] = pattern_low;
                self.sprite_pattern_shift_high[i] = pattern_high;
                self.sprite_attributes[i] = attributes;
                self.sprite_x_positions[i] = x_pos;
            } else {
                // No sprite in this slot
                self.sprite_pattern_shift_low[i] = 0;
                self.sprite_pattern_shift_high[i] = 0;
                self.sprite_attributes[i] = 0;
                self.sprite_x_positions[i] = 0xFF;
            }
        }
    }

    /// Decrement sprite X positions and shift active sprites
    ///
    /// This is called every cycle to update sprite X positions.
    /// When a sprite's X position reaches 0, it becomes active and starts shifting.
    pub(super) fn update_sprite_shifters(&mut self) {
        for i in 0..8 {
            if self.sprite_x_positions[i] == 0 {
                // Sprite is active, shift its pattern
                self.sprite_pattern_shift_low[i] <<= 1;
                self.sprite_pattern_shift_high[i] <<= 1;
            } else if self.sprite_x_positions[i] < 0xFF {
                // Decrement X position (0xFF means no sprite)
                self.sprite_x_positions[i] = self.sprite_x_positions[i].saturating_sub(1);
            }
        }
    }

    /// Get sprite pixel and palette information at the current position
    ///
    /// Returns (sprite_index, color_index, palette_index, priority, is_sprite_0)
    /// or None if no sprite pixel is visible.
    pub(super) fn get_sprite_pixel(&self) -> Option<(usize, u8, u8, bool, bool)> {
        // Check if sprite rendering is enabled
        if (self.ppumask & 0x10) == 0 {
            return None;
        }

        // Find the first non-transparent sprite pixel (priority order)
        for i in 0..self.sprite_count {
            // Only check sprites that are active (X position is 0)
            if self.sprite_x_positions[i] == 0 {
                // Get the leftmost bit from the shift registers
                let bit_0 = (self.sprite_pattern_shift_low[i] >> 7) & 0x01;
                let bit_1 = (self.sprite_pattern_shift_high[i] >> 7) & 0x01;
                let color_index = (bit_1 << 1) | bit_0;

                // Skip transparent pixels (color 0)
                if color_index != 0 {
                    let palette_index = self.sprite_attributes[i] & 0x03;
                    let priority = (self.sprite_attributes[i] & 0x20) != 0; // Behind background if set
                    let is_sprite_0 = i == 0 && self.sprite_0_present;

                    return Some((i, color_index, palette_index, priority, is_sprite_0));
                }
            }
        }

        None
    }

    /// Composite background and sprite pixels to get the final pixel color
    ///
    /// This handles sprite priority and sprite 0 hit detection.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the pixel
    /// * `bg_pixel` - Background pixel color
    ///
    /// # Returns
    ///
    /// The final pixel color to display
    pub(super) fn composite_pixel(&mut self, x: usize, bg_pixel: u8) -> u8 {
        // Get sprite pixel if any
        if let Some((_, color_index, palette_index, behind_bg, is_sprite_0)) =
            self.get_sprite_pixel()
        {
            // Get the sprite color from palette RAM
            let sprite_color = self.get_sprite_color(palette_index, color_index);

            // Get background color index for transparency detection
            // This is separate from the final palette color to handle cases where
            // a non-zero color index happens to match the backdrop color
            let bg_color_index = self.get_background_color_index();

            // Check for sprite 0 hit
            // Sprite 0 hit occurs when:
            // - Sprite 0 is being rendered
            // - A non-transparent sprite pixel overlaps a non-transparent background pixel
            // - X coordinate is not 255
            // - Rendering is enabled for both sprites and background
            if is_sprite_0 && x != 255 {
                // Check if background pixel is non-transparent
                // Background is transparent only when color index is 0
                let bg_is_transparent = bg_color_index == 0;

                if !bg_is_transparent {
                    // Set sprite 0 hit flag
                    self.ppustatus |= 0x40;
                }
            }

            // Handle sprite priority
            if behind_bg {
                // Sprite is behind background
                // Only draw sprite if background pixel is transparent (color index 0)
                if bg_color_index == 0 {
                    sprite_color
                } else {
                    bg_pixel
                }
            } else {
                // Sprite is in front of background
                sprite_color
            }
        } else {
            // No sprite pixel, use background
            bg_pixel
        }
    }
}
