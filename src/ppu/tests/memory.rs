//! PPU Memory Tests
//!
//! Tests for PPU memory operations including:
//! - Address space masking
//! - VRAM (nametables and palette) access
//! - PPUSCROLL internal register behavior
//! - PPUADDR internal register behavior
//! - Pattern table (CHR-ROM/RAM) access

use super::*;

// PPU Address Space Masking Tests
// ========================================

#[test]
fn test_ppuaddr_masks_high_byte() {
    let mut ppu = Ppu::new();

    // Write high byte with bits beyond 0x3F (should be masked to 6 bits)
    ppu.write(PPUADDR, 0xFF); // All bits set
    ppu.write(PPUADDR, 0xFF); // Low byte

    // Should be masked to 0x3FFF (high byte masked to 0x3F)
    assert_eq!(ppu.v, 0x3FFF);
}

#[test]
fn test_ppuaddr_wraps_at_boundary() {
    let mut ppu = Ppu::new();

    // Set address to 0x3FFF
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0xFF);
    assert_eq!(ppu.v, 0x3FFF);

    // Increment by 1 should wrap to 0x0000
    ppu.ppuctrl = 0x00; // Increment by 1
    ppu.write(PPUDATA, 0x42);
    assert_eq!(ppu.v, 0x0000);
}

#[test]
fn test_ppudata_read_masks_address() {
    let mut ppu = Ppu::new();
    ppu.ppuctrl = 0x00; // Increment by 1

    // Set address to near boundary
    ppu.v = 0x3FFF;

    // Read should increment and wrap
    ppu.read(PPUDATA);
    assert_eq!(ppu.v, 0x0000);
}

#[test]
fn test_ppudata_increment_32_wraps() {
    let mut ppu = Ppu::new();
    ppu.ppuctrl = 0x04; // Increment by 32

    // Set address near boundary
    ppu.v = 0x3FF0;

    // Write should increment by 32 and wrap
    ppu.write(PPUDATA, 0x42);
    assert_eq!(ppu.v, 0x0010); // (0x3FF0 + 32) & 0x3FFF = 0x0010
}

#[test]
fn test_ppuaddr_stays_within_14bit_range() {
    let mut ppu = Ppu::new();

    // Test various high bytes that exceed 0x3F
    for high_byte in 0x40..=0xFF {
        ppu.write(PPUADDR, high_byte);
        ppu.write(PPUADDR, 0x00);

        // Address should always be masked to 0x3FFF or less
        assert!(
            ppu.v <= 0x3FFF,
            "Address 0x{:04X} exceeds 14-bit range (high byte was 0x{:02X})",
            ppu.v,
            high_byte
        );
    }
}

// ========================================
// PPU Memory (VRAM) Tests
// ========================================

#[test]
fn test_nametable_read_write() {
    let mut ppu = Ppu::new();

    // Write to nametable
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x42);

    // Read back from nametable
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read (buffered)
    let value = ppu.read(PPUDATA); // Actual value
    assert_eq!(value, 0x42);
}

#[test]
fn test_nametable_horizontal_mirroring() {
    let mut ppu = Ppu::new();
    ppu.set_mirroring(Mirroring::Horizontal);

    // Write to $2000
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x11);

    // Read from $2400 (should be same as $2000 with horizontal mirroring)
    ppu.write(PPUADDR, 0x24);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x11);

    // Write to $2800
    ppu.write(PPUADDR, 0x28);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x22);

    // Read from $2C00 (should be same as $2800)
    ppu.write(PPUADDR, 0x2C);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x22);
}

#[test]
fn test_nametable_vertical_mirroring() {
    let mut ppu = Ppu::new();
    ppu.set_mirroring(Mirroring::Vertical);

    // Write to $2000
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x11);

    // Read from $2800 (should be same as $2000 with vertical mirroring)
    ppu.write(PPUADDR, 0x28);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x11);

    // Write to $2400
    ppu.write(PPUADDR, 0x24);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x22);

    // Read from $2C00 (should be same as $2400)
    ppu.write(PPUADDR, 0x2C);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x22);
}

#[test]
fn test_nametable_mirror_at_3000() {
    let mut ppu = Ppu::new();

    // Write to $2000
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x42);

    // Read from $3000 (mirrors $2000)
    ppu.write(PPUADDR, 0x30);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x42);
}

#[test]
fn test_palette_read_write() {
    let mut ppu = Ppu::new();

    // Write to background palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F); // Black

    // Palette reads are immediate (not buffered)
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x0F);
}

#[test]
fn test_palette_mirroring() {
    let mut ppu = Ppu::new();

    // Write to $3F00 (background palette 0, color 0)
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F);

    // Read from $3F10 (sprite palette 0, color 0 - mirrors $3F00)
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x10);
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x0F);

    // Write to $3F04
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x04);
    ppu.write(PPUDATA, 0x30);

    // Read from $3F14 (mirrors $3F04)
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x14);
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x30);
}

#[test]
fn test_ppudata_read_buffering() {
    let mut ppu = Ppu::new();

    // Write to nametable
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0xAA);
    ppu.write(PPUDATA, 0xBB);

    // First read is buffered (returns stale data)
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    let first_read = ppu.read(PPUDATA);
    assert_eq!(first_read, 0x00); // Stale buffer value

    // Second read returns the first byte
    let second_read = ppu.read(PPUDATA);
    assert_eq!(second_read, 0xAA);

    // Third read returns the second byte
    let third_read = ppu.read(PPUDATA);
    assert_eq!(third_read, 0xBB);
}

#[test]
fn test_ppudata_palette_read_not_buffered() {
    let mut ppu = Ppu::new();

    // Write to palette
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x0F);

    // Palette reads are immediate (not buffered)
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x00);
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x0F); // Immediate, not buffered
}

// ========================================
// PPUCTRL Effects on Internal Registers
// ========================================

#[test]
fn test_ppuctrl_updates_t_register() {
    let mut ppu = Ppu::new();

    // Write to PPUCTRL should update bits 10-11 of t register
    ppu.write(PPUCTRL, 0x03); // Nametable select = 3
    assert_eq!(ppu.t & 0x0C00, 0x0C00); // Bits 10-11 should be set

    ppu.write(PPUCTRL, 0x01); // Nametable select = 1
    assert_eq!(ppu.t & 0x0C00, 0x0400); // Bit 10 should be set, bit 11 clear
}

// ========================================
// PPUSCROLL Internal Register Tests
// ========================================

#[test]
fn test_ppuscroll_updates_t_and_fine_x() {
    let mut ppu = Ppu::new();

    // First write: X scroll
    ppu.write(PPUSCROLL, 0xF8); // Binary: 11111000
    assert_eq!(ppu.t & 0x001F, 0x1F); // Coarse X = 11111 (0x1F)
    assert_eq!(ppu.fine_x, 0); // Fine X = 000

    // Second write: Y scroll
    ppu.write(PPUSCROLL, 0xE5); // Binary: 11100101
                                // Fine Y (top 3 bits) should be in bits 12-14 of t: 111
    assert_eq!((ppu.t >> 12) & 0x07, 0x05); // Fine Y = 101
                                            // Coarse Y (bottom 5 bits) should be in bits 5-9 of t: 00101
    assert_eq!((ppu.t >> 5) & 0x1F, 0x1C); // Coarse Y = 11100
}

#[test]
fn test_ppuscroll_write_latch() {
    let mut ppu = Ppu::new();

    assert!(!ppu.write_latch);

    // First write
    ppu.write(PPUSCROLL, 0x00);
    assert!(ppu.write_latch);

    // Second write
    ppu.write(PPUSCROLL, 0x00);
    assert!(!ppu.write_latch);

    // Third write (first again)
    ppu.write(PPUSCROLL, 0x00);
    assert!(ppu.write_latch);
}

// ========================================
// PPUADDR Internal Register Tests
// ========================================

#[test]
fn test_ppuaddr_updates_t_then_v() {
    let mut ppu = Ppu::new();

    // First write should update t but not v
    ppu.write(PPUADDR, 0x20);
    assert_eq!(ppu.t & 0xFF00, 0x2000);
    assert_eq!(ppu.v, 0x0000); // v should not change yet

    // Second write should update t and copy to v
    ppu.write(PPUADDR, 0x50);
    assert_eq!(ppu.t, 0x2050);
    assert_eq!(ppu.v, 0x2050); // v should now match t
}

#[test]
fn test_ppuaddr_and_ppuscroll_share_write_latch() {
    let mut ppu = Ppu::new();

    // Start writing to PPUSCROLL
    ppu.write(PPUSCROLL, 0x10);
    assert!(ppu.write_latch);

    // Read PPUSTATUS should reset latch
    ppu.read(PPUSTATUS);
    assert!(!ppu.write_latch);

    // Next write to PPUADDR should be first write
    ppu.write(PPUADDR, 0x20);
    assert!(ppu.write_latch);
}

// ========================================
// Pattern Table (CHR-ROM/RAM) Tests
// ========================================

/// Helper function to create a test cartridge with CHR-ROM
fn create_test_cartridge_chr_rom() -> Cartridge {
    let prg_rom = vec![0xAA; 16 * 1024]; // 16KB PRG-ROM
    let mut chr_rom = vec![0x00; 8 * 1024]; // 8KB CHR-ROM

    // Fill CHR-ROM with identifiable pattern
    for (i, byte) in chr_rom.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }

    Cartridge {
        prg_rom,
        chr_rom,
        trainer: None,
        mapper: 0,
        mirroring: Mirroring::Horizontal,
        has_battery: false,
    }
}

/// Helper function to create a test cartridge with CHR-RAM
fn create_test_cartridge_chr_ram() -> Cartridge {
    let prg_rom = vec![0xAA; 16 * 1024]; // 16KB PRG-ROM
    let chr_rom = vec![0x00; 8 * 1024]; // 8KB CHR-RAM (all zeros indicates RAM)

    Cartridge {
        prg_rom,
        chr_rom,
        trainer: None,
        mapper: 0,
        mirroring: Mirroring::Horizontal,
        has_battery: false,
    }
}

#[test]
fn test_pattern_table_without_mapper() {
    let mut ppu = Ppu::new();

    // Without a mapper, pattern table reads should return 0
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read (buffered)
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x00, "Pattern table should return 0 without mapper");

    // Writes should be ignored
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0xFF);

    // Read back should still be 0
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x00);
}

#[test]
fn test_pattern_table_read_chr_rom() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_rom();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Read from pattern table 0 ($0000-$0FFF)
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read (buffered)
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x00, "CHR-ROM[0x0000] should be 0x00");

    // Read from pattern table 0, offset 0x0100
    ppu.write(PPUADDR, 0x01);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x00, "CHR-ROM[0x0100] should be 0x00");

    // Read from pattern table 1 ($1000-$1FFF)
    ppu.write(PPUADDR, 0x10);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x00, "CHR-ROM[0x1000] should be 0x00");
}

#[test]
fn test_pattern_table_write_chr_ram() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Write to pattern table 0
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x42);

    // Read back
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read (buffered)
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x42, "CHR-RAM should be writable");

    // Write to pattern table 1
    ppu.write(PPUADDR, 0x10);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0x99);

    // Read back
    ppu.write(PPUADDR, 0x10);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0x99, "CHR-RAM pattern table 1 should be writable");
}

#[test]
fn test_pattern_table_chr_rom_write_ignored() {
    let mut ppu = Ppu::new();
    let mut cartridge = create_test_cartridge_chr_rom();

    // Set specific values in CHR-ROM
    cartridge.chr_rom[0x0000] = 0xAA;
    cartridge.chr_rom[0x1000] = 0xBB;

    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Try to write to CHR-ROM (should be ignored)
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0xFF);

    // Read back - should still have original value
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(value, 0xAA, "CHR-ROM should not be writable");

    // Try to write to pattern table 1
    ppu.write(PPUADDR, 0x10);
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUDATA, 0xFF);

    // Read back
    ppu.write(PPUADDR, 0x10);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read
    let value = ppu.read(PPUDATA);
    assert_eq!(
        value, 0xBB,
        "CHR-ROM pattern table 1 should not be writable"
    );
}

#[test]
fn test_set_mapper_updates_mirroring() {
    let mut ppu = Ppu::new();

    // Initial mirroring is Horizontal
    assert_eq!(ppu.mirroring, Mirroring::Horizontal);

    // Create a cartridge with Vertical mirroring
    let mut cartridge = create_test_cartridge_chr_rom();
    cartridge.mirroring = Mirroring::Vertical;

    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Mirroring should be updated
    assert_eq!(ppu.mirroring, Mirroring::Vertical);
}

#[test]
fn test_pattern_table_full_range() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Write to various addresses across both pattern tables
    let test_addresses = [0x0000, 0x0001, 0x00FF, 0x0100, 0x0FFF, 0x1000, 0x1FFF];

    for (i, &addr) in test_addresses.iter().enumerate() {
        let test_value = (0x10 + i) as u8;

        // Write
        ppu.write(PPUADDR, (addr >> 8) as u8);
        ppu.write(PPUADDR, (addr & 0xFF) as u8);
        ppu.write(PPUDATA, test_value);

        // Read back
        ppu.write(PPUADDR, (addr >> 8) as u8);
        ppu.write(PPUADDR, (addr & 0xFF) as u8);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);

        assert_eq!(
            value, test_value,
            "CHR-RAM at address ${:04X} should be writable",
            addr
        );
    }
}

#[test]
fn test_pattern_table_sequential_access() {
    let mut ppu = Ppu::new();
    let cartridge = create_test_cartridge_chr_ram();
    let mapper = Rc::new(RefCell::new(
        Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
    ));
    ppu.set_mapper(mapper);

    // Set increment to +1
    ppu.write(PPUCTRL, 0x00);

    // Write sequential data
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    for i in 0..16 {
        ppu.write(PPUDATA, i);
    }

    // Read back sequential data
    ppu.write(PPUADDR, 0x00);
    ppu.write(PPUADDR, 0x00);
    let _ = ppu.read(PPUDATA); // Dummy read

    for i in 0..16 {
        let value = ppu.read(PPUDATA);
        assert_eq!(value, i, "Sequential CHR-RAM read failed at index {}", i);
    }
}
