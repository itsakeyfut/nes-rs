//! PPU unit tests
//!
//! This module contains comprehensive tests for the PPU implementation,
//! organized by functionality.

use super::*;
use crate::cartridge::mappers::Mapper0;
use crate::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;

// ========================================
// Test Constants (PPU Register Addresses)
// ========================================

/// PPU Control Register ($2000) - Write only
pub(crate) const PPUCTRL: u16 = 0x2000;
/// PPU Mask Register ($2001) - Write only
pub(crate) const PPUMASK: u16 = 0x2001;
/// PPU Status Register ($2002) - Read only
pub(crate) const PPUSTATUS: u16 = 0x2002;
/// OAM Address Port ($2003) - Write only
pub(crate) const OAMADDR: u16 = 0x2003;
/// OAM Data Port ($2004) - Read/Write
pub(crate) const OAMDATA: u16 = 0x2004;
/// Scroll Position Register ($2005) - Write×2
pub(crate) const PPUSCROLL: u16 = 0x2005;
/// PPU Address Register ($2006) - Write×2
pub(crate) const PPUADDR: u16 = 0x2006;
/// PPU Data Port ($2007) - Read/Write
pub(crate) const PPUDATA: u16 = 0x2007;

// ========================================
// Test Helper Functions
// ========================================

/// Helper function to create a test cartridge with CHR-RAM
pub(crate) fn create_test_cartridge_chr_ram() -> Cartridge {
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

// ========================================
// Test Modules
// ========================================

mod memory;
mod registers;
mod rendering;
mod timing;
