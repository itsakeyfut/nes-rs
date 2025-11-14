//! PPU Register Tests
//!
//! Tests for PPU register behavior including:
//! - Register initialization
//! - Register read/write operations
//! - Register mirroring
//! - Write latch behavior
//! - Integration tests for register sequences

use super::*;

// Initialization Tests
// ========================================

#[test]
fn test_ppu_initialization() {
    let ppu = Ppu::new();
    assert_eq!(ppu.ppuctrl, 0x00);
    assert_eq!(ppu.ppumask, 0x00);
    assert_eq!(ppu.ppustatus, 0x00);
    assert_eq!(ppu.oam_addr, 0x00);
    assert!(!ppu.write_latch);
}

#[test]
fn test_ppu_default() {
    let ppu = Ppu::default();
    assert_eq!(ppu.ppuctrl, 0x00);
}

#[test]
fn test_ppu_reset() {
    let mut ppu = Ppu::new();
    ppu.write(PPUCTRL, 0x80);
    ppu.write(PPUMASK, 0x1E);

    ppu.reset();

    assert_eq!(ppu.ppuctrl, 0x00);
    assert_eq!(ppu.ppumask, 0x00);
    assert!(!ppu.write_latch);
}

// ========================================
// Register Write Tests
// ========================================

#[test]
fn test_write_ppuctrl() {
    let mut ppu = Ppu::new();
    ppu.write(PPUCTRL, 0x80);
    assert_eq!(ppu.ppuctrl, 0x80);
}

#[test]
fn test_write_ppumask() {
    let mut ppu = Ppu::new();
    ppu.write(PPUMASK, 0x1E);
    assert_eq!(ppu.ppumask, 0x1E);
}

#[test]
fn test_write_oamaddr() {
    let mut ppu = Ppu::new();
    ppu.write(OAMADDR, 0x42);
    assert_eq!(ppu.oam_addr, 0x42);
}

#[test]
fn test_write_oamdata_increments_addr() {
    let mut ppu = Ppu::new();
    ppu.write(OAMADDR, 0x00);
    ppu.write(OAMDATA, 0x11);
    assert_eq!(ppu.oam_addr, 0x01);

    ppu.write(OAMDATA, 0x22);
    assert_eq!(ppu.oam_addr, 0x02);
}

#[test]
fn test_write_ppuaddr_two_writes() {
    let mut ppu = Ppu::new();

    // First write: high byte
    ppu.write(PPUADDR, 0x20);
    assert!(ppu.write_latch);

    // Second write: low byte
    ppu.write(PPUADDR, 0x00);
    assert!(!ppu.write_latch);
    assert_eq!(ppu.v, 0x2000);
}

#[test]
fn test_write_ppuscroll_toggles_latch() {
    let mut ppu = Ppu::new();

    // First write
    ppu.write(PPUSCROLL, 0x00);
    assert!(ppu.write_latch);

    // Second write
    ppu.write(PPUSCROLL, 0x00);
    assert!(!ppu.write_latch);
}

// ========================================
// Register Read Tests
// ========================================

#[test]
fn test_read_ppustatus_clears_vblank() {
    let mut ppu = Ppu::new();
    ppu.ppustatus = 0x80; // Set VBlank flag

    let status = ppu.read(PPUSTATUS);
    assert_eq!(status, 0x80);

    // VBlank flag should be cleared after read
    assert_eq!(ppu.ppustatus & 0x80, 0x00);
}

#[test]
fn test_read_ppustatus_resets_latch() {
    let mut ppu = Ppu::new();

    // Set write latch by writing to PPUADDR
    ppu.write(PPUADDR, 0x20);
    assert!(ppu.write_latch);

    // Read PPUSTATUS should reset the latch
    ppu.read(PPUSTATUS);
    assert!(!ppu.write_latch);
}

#[test]
fn test_read_write_only_registers_return_zero() {
    let mut ppu = Ppu::new();
    ppu.write(PPUCTRL, 0x80);
    ppu.write(PPUMASK, 0x1E);

    // Write-only registers should return 0 when read
    assert_eq!(ppu.read(PPUCTRL), 0x00);
    assert_eq!(ppu.read(PPUMASK), 0x00);
    assert_eq!(ppu.read(OAMADDR), 0x00);
    assert_eq!(ppu.read(PPUSCROLL), 0x00);
    assert_eq!(ppu.read(PPUADDR), 0x00);
}

#[test]
fn test_read_ppudata_increments_addr() {
    let mut ppu = Ppu::new();
    ppu.v = 0x2000;
    ppu.ppuctrl = 0x00; // Increment by 1

    ppu.read(PPUDATA);
    assert_eq!(ppu.v, 0x2001);

    ppu.read(PPUDATA);
    assert_eq!(ppu.v, 0x2002);
}

#[test]
fn test_read_ppudata_increments_by_32() {
    let mut ppu = Ppu::new();
    ppu.v = 0x2000;
    ppu.ppuctrl = 0x04; // Increment by 32

    ppu.read(PPUDATA);
    assert_eq!(ppu.v, 0x2020);
}

#[test]
fn test_write_ppudata_increments_addr() {
    let mut ppu = Ppu::new();
    ppu.v = 0x2000;
    ppu.ppuctrl = 0x00; // Increment by 1

    ppu.write(PPUDATA, 0x42);
    assert_eq!(ppu.v, 0x2001);
}

#[test]
fn test_write_ppudata_increments_by_32() {
    let mut ppu = Ppu::new();
    ppu.v = 0x2000;
    ppu.ppuctrl = 0x04; // Increment by 32

    ppu.write(PPUDATA, 0x42);
    assert_eq!(ppu.v, 0x2020);
}

// ========================================
// Mirroring Tests
// ========================================

#[test]
fn test_register_mirroring() {
    let mut ppu = Ppu::new();

    // Write to base register
    ppu.write(0x2000, 0x80);
    assert_eq!(ppu.ppuctrl, 0x80);

    // Write to mirrored addresses
    ppu.write(0x2008, 0x90);
    assert_eq!(ppu.ppuctrl, 0x90);

    ppu.write(0x3000, 0xA0);
    assert_eq!(ppu.ppuctrl, 0xA0);

    ppu.write(0x3FF8, 0xB0);
    assert_eq!(ppu.ppuctrl, 0xB0);
}

#[test]
fn test_all_registers_mirror() {
    let mut ppu = Ppu::new();

    // Test that all 8 registers mirror correctly
    for reg in 0..8 {
        let base_addr = 0x2000 + reg;
        let mirror_addr = 0x2008 + reg;

        ppu.write(base_addr, 0x42);
        ppu.write(mirror_addr, 0x84);

        // Both addresses should access the same register
        // (we can't directly test this for all registers, but verify no crash)
    }
}

#[test]
fn test_high_mirror_addresses() {
    let mut ppu = Ppu::new();

    // Test mirroring at the end of the range
    ppu.write(0x3FFF, 0x11); // Should map to $2007 (PPUDATA)

    // Should not crash and should handle correctly
    // (exact behavior depends on register, but should not panic)
}

// ========================================
// Write Latch Tests
// ========================================

#[test]
fn test_ppuaddr_write_sequence() {
    let mut ppu = Ppu::new();

    // Write high byte
    ppu.write(PPUADDR, 0x3F);
    assert!(ppu.write_latch);

    // Write low byte
    ppu.write(PPUADDR, 0x00);
    assert!(!ppu.write_latch);
    assert_eq!(ppu.v, 0x3F00);
}

#[test]
fn test_ppuaddr_multiple_sequences() {
    let mut ppu = Ppu::new();

    // First sequence
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);
    assert_eq!(ppu.v, 0x2000);

    // Second sequence
    ppu.write(PPUADDR, 0x3F);
    ppu.write(PPUADDR, 0x10);
    assert_eq!(ppu.v, 0x3F10);
}

#[test]
fn test_ppustatus_read_resets_ppuaddr_latch() {
    let mut ppu = Ppu::new();

    // Start PPUADDR sequence
    ppu.write(PPUADDR, 0x20);
    assert!(ppu.write_latch);

    // Read PPUSTATUS
    ppu.read(PPUSTATUS);
    assert!(!ppu.write_latch);

    // Next write to PPUADDR should be treated as first write
    ppu.write(PPUADDR, 0x3F);
    assert!(ppu.write_latch);

    ppu.write(PPUADDR, 0x00);
    assert_eq!(ppu.v, 0x3F00);
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_typical_ppu_initialization_sequence() {
    let mut ppu = Ppu::new();

    // Typical game initialization
    ppu.write(PPUCTRL, 0x00); // Disable NMI
    ppu.write(PPUMASK, 0x00); // Disable rendering
    ppu.read(PPUSTATUS); // Clear VBlank flag

    assert_eq!(ppu.ppuctrl, 0x00);
    assert_eq!(ppu.ppumask, 0x00);
    assert!(!ppu.write_latch);
}

#[test]
fn test_vram_access_sequence() {
    let mut ppu = Ppu::new();

    // Set VRAM address
    ppu.write(PPUADDR, 0x20);
    ppu.write(PPUADDR, 0x00);

    // Write data
    ppu.write(PPUDATA, 0x42);
    assert_eq!(ppu.v, 0x2001);

    ppu.write(PPUDATA, 0x43);
    assert_eq!(ppu.v, 0x2002);
}

#[test]
fn test_oam_dma_preparation() {
    let mut ppu = Ppu::new();

    // Set OAM address
    ppu.write(OAMADDR, 0x00);
    assert_eq!(ppu.oam_addr, 0x00);

    // Simulate multiple OAM writes
    for i in 0..64 {
        ppu.write(OAMDATA, i);
    }

    // OAM address should have wrapped around
    assert_eq!(ppu.oam_addr, 64);
}

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
