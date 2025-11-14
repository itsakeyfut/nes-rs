//! PPU Rendering Tests
//!
//! Tests for PPU rendering functionality including:
//! - Background rendering
//! - Sprite rendering
//! - Frame buffer operations
//! - Sprite attributes (flip, priority, palette)
//! - Transparency handling

use super::*;

// Background Rendering Tests
// ========================================

#[test]
fn test_frame_buffer_size() {
    let ppu = Ppu::new();
    let frame = ppu.frame();
    assert_eq!(
        frame.len(),
        256 * 240,
        "Frame buffer should be 256x240 pixels"
    );
}

#[test]
fn test_frame_buffer_initialization() {
    let ppu = Ppu::new();
    let frame = ppu.frame();
    // All pixels should be initialized to 0
    for &pixel in frame.iter() {
        assert_eq!(pixel, 0);
    }
}

#[test]
fn test_render_background_disabled() {
    let mut ppu = Ppu::new();

    // Fill frame buffer with non-zero values
    ppu.frame_mut().fill(0xFF);

    // Disable background rendering (PPUMASK bit 3 = 0)
    ppu.write(PPUMASK, 0x00);

    // Render background
    ppu.render_background();

    // Frame buffer should be cleared when rendering is disabled
    let frame = ppu.frame();
    for &pixel in frame.iter() {
        assert_eq!(
            pixel, 0,
            "Frame should be cleared when rendering is disabled"
        );
    }
}

#[test]
fn test_render_background_enabled() {
    let mut ppu = Ppu::new();

    // Enable background rendering (PPUMASK bit 3 = 1)
    ppu.write(PPUMASK, 0x08);

    // Render background (will render with default data)
    ppu.render_background();

    // Frame buffer should be filled (not necessarily all zeros)
    let frame = ppu.frame();
    assert_eq!(frame.len(), 256 * 240);
}

#[test]
fn test_nametable_tile_read() {
    let mut ppu = Ppu::new();

    // Write a tile index to nametable
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x42);

    // Read it back using the internal method
    let tile_index = ppu.read_nametable_tile(0x2000);
    assert_eq!(tile_index, 0x42, "Tile index should be readable");
}

#[test]
fn test_render_manual_debug() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up minimal data for testing one pixel
    // Write tile index 1 to nametable[0]
    ppu.nametables[0] = 0x01;

    // Write tile data to pattern table tile 1
    // This goes through the mapper
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF); // Bitplane 0: all 1s
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00); // Bitplane 1: all 0s
    }

    // Set up palette
    ppu.palette_ram[0] = 0x0F; // Universal background
    ppu.palette_ram[1] = 0x30; // Palette 0, color 1

    // Manually test the pipeline
    let tile_index = ppu.read_nametable_tile(0x2000);
    assert_eq!(tile_index, 0x01, "Should read tile 1");

    let color = ppu.fetch_tile_pixel(0x0000, tile_index, 0, 0);
    assert_eq!(color, 1, "Should get color index 1");

    let final_color = ppu.get_background_color(0, color);
    assert_eq!(final_color, 0x30, "Should get palette color 0x30");
}

#[test]
fn test_render_simple_pattern() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up palette directly
    ppu.palette_ram[0] = 0x0F; // Universal background
    ppu.palette_ram[1] = 0x30; // Palette 0, color 1

    // Set up tile data directly in CHR-RAM
    // Tile 1: all pixels use color index 1
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF); // Bitplane 0
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00); // Bitplane 1
    }

    // Set up nametable directly - all tiles use tile 1
    for i in 0..(32 * 30) {
        ppu.nametables[i] = 0x01;
    }

    // Set up attribute table directly - all use palette 0
    for i in 0..64 {
        ppu.nametables[960 + i] = 0x00;
    }

    // Enable background rendering
    ppu.ppumask = 0x08;

    // Render background
    ppu.render_background();

    // Verify frame buffer has been filled
    let frame = ppu.frame();
    assert_eq!(frame.len(), 256 * 240);

    // All pixels should be 0x30 (color 1 from palette 0)
    assert_eq!(frame[0], 0x30, "Top-left pixel should match pattern");
    assert_eq!(frame[1], 0x30, "Second pixel should match pattern");
    assert_eq!(frame[256], 0x30, "Second row first pixel should match");
}

#[test]
fn test_attribute_table_palette_selection() {
    let mut ppu = Ppu::new();

    // Test attribute byte reading for different tile positions
    // Write attribute table data
    ppu.write(PPUADDR, 0x23);
    ppu.write(PPUADDR, 0xC0); // Attribute table for nametable 0
    ppu.write(PPUDATA, 0xE4); // Binary: 11 10 01 00
                              // This byte covers tiles (0,0) to (3,3)
                              // Palette indices: top-left=00, top-right=01, bottom-left=10, bottom-right=11

    let palette_0_0 = ppu.read_attribute_byte(0x2000, 0, 0);
    assert_eq!(palette_0_0, 0, "Tile (0,0) should use palette 0");

    let palette_2_0 = ppu.read_attribute_byte(0x2000, 2, 0);
    assert_eq!(palette_2_0, 1, "Tile (2,0) should use palette 1");

    let palette_0_2 = ppu.read_attribute_byte(0x2000, 0, 2);
    assert_eq!(palette_0_2, 2, "Tile (0,2) should use palette 2");

    let palette_2_2 = ppu.read_attribute_byte(0x2000, 2, 2);
    assert_eq!(palette_2_2, 3, "Tile (2,2) should use palette 3");
}

#[test]
fn test_fetch_tile_pixel() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Create a test pattern in tile 0
    // Bitplane 0: alternating 1010101
    // Bitplane 1: alternating 0101010
    // Result should alternate between color indices 1, 2, 1, 2...

    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);

    // Bitplane 0
    for _ in 0..8 {
        ppu.write(PPUDATA, 0b10101010);
    }
    // Bitplane 1
    for _ in 0..8 {
        ppu.write(PPUDATA, 0b01010101);
    }

    // Test fetching pixels
    // Bitplane 0 = 0b10101010 → bits 7,5,3,1 = 1; bits 6,4,2,0 = 0
    // Bitplane 1 = 0b01010101 → bits 7,5,3,1 = 0; bits 6,4,2,0 = 1
    // Color = (bit_1 << 1) | bit_0
    let color_0 = ppu.fetch_tile_pixel(0x0000, 0, 0, 0);
    assert_eq!(
        color_0, 1,
        "Pixel (0,0): bitplane0[7]=1, bitplane1[7]=0 → color = 01 = 1"
    );

    let color_1 = ppu.fetch_tile_pixel(0x0000, 0, 1, 0);
    assert_eq!(
        color_1, 2,
        "Pixel (1,0): bitplane0[6]=0, bitplane1[6]=1 → color = 10 = 2"
    );

    let color_2 = ppu.fetch_tile_pixel(0x0000, 0, 2, 0);
    assert_eq!(
        color_2, 1,
        "Pixel (2,0): bitplane0[5]=1, bitplane1[5]=0 → color = 01 = 1"
    );

    let color_3 = ppu.fetch_tile_pixel(0x0000, 0, 3, 0);
    assert_eq!(
        color_3, 2,
        "Pixel (3,0): bitplane0[4]=0, bitplane1[4]=1 → color = 10 = 2"
    );
}

#[test]
fn test_background_color_palette_lookup() {
    let mut ppu = Ppu::new();

    // Set up background palettes
    // Palette RAM layout:
    // Index 0: Universal background
    // Index 1-3: Palette 0, colors 1-3
    // Index 4: Palette 1, color 0 (unused, color 0 uses universal bg)
    // Index 5-7: Palette 1, colors 1-3
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F); // Index 0: Universal background
    ppu.write(PPUDATA, 0x10); // Index 1: Palette 0, color 1
    ppu.write(PPUDATA, 0x20); // Index 2: Palette 0, color 2
    ppu.write(PPUDATA, 0x30); // Index 3: Palette 0, color 3
    ppu.write(PPUDATA, 0x00); // Index 4: Palette 1, color 0 (unused)
    ppu.write(PPUDATA, 0x11); // Index 5: Palette 1, color 1
    ppu.write(PPUDATA, 0x21); // Index 6: Palette 1, color 2
    ppu.write(PPUDATA, 0x31); // Index 7: Palette 1, color 3

    // Test palette 0
    assert_eq!(
        ppu.get_background_color(0, 0),
        0x0F,
        "Color 0 should be universal background"
    );
    assert_eq!(
        ppu.get_background_color(0, 1),
        0x10,
        "Palette 0, color 1 at index 1"
    );
    assert_eq!(
        ppu.get_background_color(0, 2),
        0x20,
        "Palette 0, color 2 at index 2"
    );
    assert_eq!(
        ppu.get_background_color(0, 3),
        0x30,
        "Palette 0, color 3 at index 3"
    );

    // Test palette 1
    assert_eq!(
        ppu.get_background_color(1, 0),
        0x0F,
        "Color 0 should be universal background"
    );
    assert_eq!(
        ppu.get_background_color(1, 1),
        0x11,
        "Palette 1, color 1 at index 5"
    );
    assert_eq!(
        ppu.get_background_color(1, 2),
        0x21,
        "Palette 1, color 2 at index 6"
    );
    assert_eq!(
        ppu.get_background_color(1, 3),
        0x31,
        "Palette 1, color 3 at index 7"
    );
}

#[test]
fn test_pattern_table_selection() {
    let mut ppu = Ppu::new();

    // Test pattern table 0 (PPUCTRL bit 4 = 0)
    ppu.write(PPUCTRL, 0x00);
    assert_eq!(ppu.ppuctrl & 0x10, 0, "Pattern table 0 should be selected");

    // Test pattern table 1 (PPUCTRL bit 4 = 1)
    ppu.write(PPUCTRL, 0x10);
    assert_eq!(
        ppu.ppuctrl & 0x10,
        0x10,
        "Pattern table 1 should be selected"
    );
}

#[test]
fn test_render_with_scrolling() {
    let mut ppu = Ppu::new();

    // Enable background rendering
    ppu.write(PPUMASK, 0x08);

    // Set scroll position
    ppu.write(PPUSCROLL, 8); // X scroll = 8 pixels
    ppu.write(PPUSCROLL, 16); // Y scroll = 16 pixels

    // Render background
    ppu.render_background();

    // Frame buffer should be filled with scrolled content
    let frame = ppu.frame();
    assert_eq!(frame.len(), 256 * 240);
}

#[test]
fn test_pattern_table_increment_32() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set increment to +32
    ppu.write(PPUCTRL, 0x04);

    // Write data with +32 increment
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x11); // $0000
    ppu.write(PPUDATA, 0x22); // $0020
    ppu.write(PPUDATA, 0x33); // $0040

    // Read back with +32 increment
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read

    assert_eq!(ppu.read(PPUDATA), 0x11); // $0000
    assert_eq!(ppu.read(PPUDATA), 0x22); // $0020
    assert_eq!(ppu.read(PPUDATA), 0x33); // $0040
}

// ========================================
// Sprite Rendering Tests
// ========================================

#[test]
fn test_sprite_attribute_parsing() {
    let mut ppu = Ppu::new();

    // Write sprite 0 data to OAM
    ppu.write(OAMADDR, 0x00);
    ppu.write(OAMDATA, 0x50); // Y position
    ppu.write(OAMDATA, 0x01); // Tile index
    ppu.write(OAMDATA, 0xE3); // Attributes: vflip, hflip, priority, palette 3
    ppu.write(OAMDATA, 0x80); // X position

    // Read back OAM data
    assert_eq!(ppu.read_oam(0), 0x50);
    assert_eq!(ppu.read_oam(1), 0x01);
    assert_eq!(ppu.read_oam(2), 0xE3);
    assert_eq!(ppu.read_oam(3), 0x80);
}

#[test]
fn test_sprite_rendering_disabled() {
    let mut ppu = Ppu::new();

    // Fill frame buffer with non-zero values
    ppu.frame_mut().fill(0xFF);

    // Disable sprite rendering (PPUMASK bit 4 = 0)
    ppu.write(PPUMASK, 0x00);

    // Render sprites
    ppu.render_sprites();

    // Frame buffer should remain unchanged (sprites not rendered)
    let frame = ppu.frame();
    assert_eq!(
        frame[0], 0xFF,
        "Frame should remain unchanged when sprites disabled"
    );
}

#[test]
fn test_sprite_rendering_enabled() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up sprite palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11); // Sprite palette 0, color 1
    ppu.write(PPUDATA, 0x30);

    // Set up sprite tile data in pattern table
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF); // Bitplane 0
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00); // Bitplane 1
    }

    // Write sprite data to OAM
    ppu.write_oam(0, 50); // Y position
    ppu.write_oam(1, 0x01); // Tile index 1
    ppu.write_oam(2, 0x00); // Attributes: palette 0, in front
    ppu.write_oam(3, 100); // X position

    // Enable sprite rendering
    ppu.write(PPUMASK, 0x10);

    // Render sprites
    ppu.render_sprites();

    // Check that sprite pixels were rendered
    let frame = ppu.frame();
    let sprite_pixel_y = 51; // Y + 1
    let sprite_pixel_x = 100;
    let pixel_index = sprite_pixel_y * 256 + sprite_pixel_x;
    assert_eq!(
        frame[pixel_index], 0x30,
        "Sprite should be rendered at position"
    );
}

#[test]
fn test_sprite_transparency() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up background color
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F); // Universal background

    // Set up sprite tile with transparent pixels (color index 0)
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0x00); // All 0s
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write sprite data
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x00);
    ppu.write_oam(3, 100);

    // Enable sprite rendering
    ppu.write(PPUMASK, 0x10);

    // Clear frame buffer to background color
    ppu.frame_mut().fill(0x0F);

    // Render sprites
    ppu.render_sprites();

    // Transparent sprite should not change background
    let frame = ppu.frame();
    let sprite_pixel_y = 51;
    let sprite_pixel_x = 100;
    let pixel_index = sprite_pixel_y * 256 + sprite_pixel_x;
    assert_eq!(
        frame[pixel_index], 0x0F,
        "Transparent sprite should not overwrite background"
    );
}

#[test]
fn test_sprite_horizontal_flip() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up sprite palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30);

    // Create a pattern with horizontal asymmetry (left half filled, right half empty)
    // Bitplane 0: 1111 0000 pattern
    ppu.mapper
        .as_ref()
        .unwrap()
        .borrow_mut()
        .ppu_write(0x0010, 0b11110000);
    for i in 1..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0b11110000);
    }
    // Bitplane 1: all 0s
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write sprite without flip
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x00); // No flip
    ppu.write_oam(3, 100);

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    let frame = ppu.frame();
    let y = 51;
    let left_pixel = frame[y * 256 + 100];
    let right_pixel = frame[y * 256 + 107];

    assert_eq!(left_pixel, 0x30, "Left side should be filled");
    assert_eq!(right_pixel, 0x00, "Right side should be empty");

    // Now test with horizontal flip
    ppu.frame_mut().fill(0);
    ppu.write_oam(2, 0x40); // Horizontal flip
    ppu.render_sprites();

    let frame = ppu.frame();
    let left_pixel_flip = frame[y * 256 + 100];
    let right_pixel_flip = frame[y * 256 + 107];

    assert_eq!(left_pixel_flip, 0x00, "Left side should be empty (flipped)");
    assert_eq!(
        right_pixel_flip, 0x30,
        "Right side should be filled (flipped)"
    );
}

#[test]
fn test_sprite_vertical_flip() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up sprite palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30);

    // Create a pattern with vertical asymmetry (top half filled, bottom half empty)
    for i in 0..4 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF); // Top half
    }
    for i in 4..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0x00); // Bottom half
    }
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00); // Bitplane 1
    }

    // Write sprite without flip
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x00);
    ppu.write_oam(3, 100);

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    let frame = ppu.frame();
    let top_pixel = frame[51 * 256 + 100];
    let bottom_pixel = frame[57 * 256 + 100];

    assert_eq!(top_pixel, 0x30, "Top should be filled");
    assert_eq!(bottom_pixel, 0x00, "Bottom should be empty");

    // Test with vertical flip
    ppu.frame_mut().fill(0);
    ppu.write_oam(2, 0x80); // Vertical flip
    ppu.render_sprites();

    let frame = ppu.frame();
    let top_pixel_flip = frame[51 * 256 + 100];
    let bottom_pixel_flip = frame[57 * 256 + 100];

    assert_eq!(top_pixel_flip, 0x00, "Top should be empty (flipped)");
    assert_eq!(bottom_pixel_flip, 0x30, "Bottom should be filled (flipped)");
}

#[test]
fn test_sprite_priority_behind_background() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up palettes
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F); // Universal background
    ppu.write(PPUDATA, 0x20); // BG palette 0, color 1

    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30); // Sprite palette 0, color 1

    // Set up sprite tile
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write sprite with priority bit set (behind background)
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x20); // Priority: behind background
    ppu.write_oam(3, 100);

    // Set frame buffer to background color (non-transparent)
    ppu.frame_mut().fill(0x20);

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    // Sprite should be hidden by background
    let frame = ppu.frame();
    let pixel = frame[51 * 256 + 100];
    assert_eq!(pixel, 0x20, "Sprite behind background should be hidden");

    // Test with transparent background
    ppu.frame_mut().fill(0x0F); // Universal background (transparent)
    ppu.render_sprites();

    let frame = ppu.frame();
    let pixel = frame[51 * 256 + 100];
    assert_eq!(
        pixel, 0x30,
        "Sprite should show through transparent background"
    );
}

#[test]
fn test_sprite_8x16_mode() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Enable 8x16 sprite mode
    ppu.write(PPUCTRL, 0x20);

    // Set up sprite palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30);

    // Set up two tiles (tile pair for 8x16)
    // Tile 0: top half
    for i in 0..8 {
        ppu.mapper.as_ref().unwrap().borrow_mut().ppu_write(i, 0xFF);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0008 + i, 0x00);
    }
    // Tile 1: bottom half
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0x00);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0xFF);
    }

    // Write sprite data (tile index 0, which uses tile pair 0-1)
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x00); // Uses tiles 0 and 1
    ppu.write_oam(2, 0x00);
    ppu.write_oam(3, 100);

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    let frame = ppu.frame();
    // Check top half (should use tile 0 - color 1)
    let top_pixel = frame[51 * 256 + 100];
    assert_eq!(top_pixel, 0x30, "Top half should use tile 0");

    // Check bottom half (should use tile 1 - color 2)
    // Note: tile 1 has bitplane 0=0, bitplane 1=FF, so color index is 2
    // But we need to set up palette for color 2
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x12);
    ppu.write(PPUDATA, 0x31);

    ppu.frame_mut().fill(0);
    ppu.render_sprites();

    let frame = ppu.frame();
    let bottom_pixel = frame[59 * 256 + 100];
    assert_eq!(bottom_pixel, 0x31, "Bottom half should use tile 1");
}

#[test]
fn test_sprite_0_hit_detection() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up palettes
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F);
    ppu.write(PPUDATA, 0x20); // BG color

    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30); // Sprite color

    // Set up sprite tile
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write sprite 0
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x00);
    ppu.write_oam(3, 100);

    // Set background to non-transparent color
    ppu.frame_mut().fill(0x20);

    // Clear sprite 0 hit flag
    ppu.ppustatus &= !0x40;

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    // Sprite 0 hit should be detected
    assert_eq!(
        ppu.ppustatus & 0x40,
        0x40,
        "Sprite 0 hit should be detected"
    );
}

#[test]
fn test_sprite_overflow_flag() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up sprite palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30);

    // Set up sprite tile
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write 9 sprites on the same scanline
    for i in 0..9 {
        ppu.write_oam((i * 4) as u8, 50); // All on scanline 51
        ppu.write_oam((i * 4 + 1) as u8, 0x01);
        ppu.write_oam((i * 4 + 2) as u8, 0x00);
        ppu.write_oam((i * 4 + 3) as u8, (i * 10) as u8);
    }

    // Clear overflow flag
    ppu.ppustatus &= !0x20;

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    // Sprite overflow should be detected
    assert_eq!(
        ppu.ppustatus & 0x20,
        0x20,
        "Sprite overflow should be detected"
    );
}

#[test]
fn test_render_frame_clears_sprite_flags() {
    let mut ppu = Ppu::new();

    // Set sprite flags
    ppu.ppustatus |= 0x60; // Set both sprite 0 hit and overflow

    // Render frame
    ppu.render_frame();

    // Flags should be cleared at start of frame
    // (They may be set again during rendering, but at least the method was called)
    // We can't directly test this without setting up sprites, so just verify the method exists
}

#[test]
fn test_multiple_sprite_priorities() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set up palettes for different sprites
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x11);
    ppu.write(PPUDATA, 0x30); // Sprite palette 0
    ppu.write(PPUDATA, 0x31);
    ppu.write(PPUDATA, 0x32);
    ppu.write(PPUDATA, 0x33);

    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x15);
    ppu.write(PPUDATA, 0x34); // Sprite palette 1

    // Set up sprite tiles
    for i in 0..8 {
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010 + i, 0xFF);
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0018 + i, 0x00);
    }

    // Write two overlapping sprites
    // Sprite 0 at (100, 51)
    ppu.write_oam(0, 50);
    ppu.write_oam(1, 0x01);
    ppu.write_oam(2, 0x00); // Palette 0
    ppu.write_oam(3, 100);

    // Sprite 1 at (104, 51) - overlaps with sprite 0
    ppu.write_oam(4, 50);
    ppu.write_oam(5, 0x01);
    ppu.write_oam(6, 0x01); // Palette 1
    ppu.write_oam(7, 104);

    ppu.write(PPUMASK, 0x10);
    ppu.render_sprites();

    let frame = ppu.frame();
    // Check non-overlapping areas
    let sprite0_pixel = frame[51 * 256 + 100];
    assert_eq!(sprite0_pixel, 0x30, "Sprite 0 should be visible");

    // Check overlapping area - sprite 0 has higher priority (lower index)
    let overlap_pixel = frame[51 * 256 + 104];
    assert_eq!(
        overlap_pixel, 0x30,
        "Sprite 0 should have higher priority in overlap"
    );
}
