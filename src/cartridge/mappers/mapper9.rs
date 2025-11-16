// Mapper 9 (MMC2) - Advanced mapper with latch-based CHR banking
//
// Memory Layout:
// - CPU $8000-$9FFF: 8KB switchable PRG-ROM bank
// - CPU $A000-$FFFF: 24KB PRG-ROM (fixed to last 3 banks)
// - PPU $0000-$0FFF: 4KB switchable CHR-ROM bank (with latch FD/FE)
// - PPU $1000-$1FFF: 4KB switchable CHR-ROM bank (with latch FD/FE)
//
// Features:
// - 8KB PRG-ROM banking (switchable at $8000-$9FFF)
// - Latch-based 4KB CHR-ROM banking
// - Two independent CHR latches for $0000 and $1000 regions
// - Latch switches on PPU reads of specific tile addresses ($xFD8-$xFDF and $xFE8-$xFEF)
// - PRG-ROM size: up to 128KB
// - CHR-ROM size: up to 128KB
//
// Register Interface:
// - $A000-$AFFF: PRG-ROM bank select (3 bits)
// - $B000-$BFFF: CHR-ROM bank select for $0000-$0FFF when latch 0 = $FD
// - $C000-$CFFF: CHR-ROM bank select for $0000-$0FFF when latch 0 = $FE
// - $D000-$DFFF: CHR-ROM bank select for $1000-$1FFF when latch 1 = $FD
// - $E000-$EFFF: CHR-ROM bank select for $1000-$1FFF when latch 1 = $FE
// - $F000-$FFFF: Mirroring (bit 0: 0=vertical, 1=horizontal)
//
// Latch Behavior:
// - Reading from $0FD8-$0FDF sets latch 0 to $FD
// - Reading from $0FE8-$0FEF sets latch 0 to $FE
// - Reading from $1FD8-$1FDF sets latch 1 to $FD
// - Reading from $1FE8-$1FEF sets latch 1 to $FE

use crate::cartridge::{Cartridge, Mapper, Mirroring};
use std::cell::Cell;

/// PRG-ROM bank size (8KB)
const PRG_BANK_SIZE: usize = 8 * 1024;

/// CHR-ROM bank size (4KB)
const CHR_BANK_SIZE: usize = 4 * 1024;

/// Mapper 9 implementation (MMC2)
///
/// MMC2 was used primarily by Punch-Out!! and is notable for its
/// latch-based CHR banking system that allows for sprite animation tricks.
pub struct Mapper9 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-ROM data
    chr_rom: Vec<u8>,

    // Internal registers
    /// PRG-ROM bank select (for $8000-$9FFF)
    prg_bank: u8,
    /// CHR bank for $0000-$0FFF when latch 0 = $FD
    chr_bank_0_fd: u8,
    /// CHR bank for $0000-$0FFF when latch 0 = $FE
    chr_bank_0_fe: u8,
    /// CHR bank for $1000-$1FFF when latch 1 = $FD
    chr_bank_1_fd: u8,
    /// CHR bank for $1000-$1FFF when latch 1 = $FE
    chr_bank_1_fe: u8,
    /// Mirroring mode
    mirroring: Mirroring,

    // Latch state
    /// Latch 0 state (false = $FD, true = $FE)
    latch_0: Cell<bool>,
    /// Latch 1 state (false = $FD, true = $FE)
    latch_1: Cell<bool>,

    // Derived state
    /// Number of 8KB PRG-ROM banks
    prg_banks: usize,
    /// Number of 4KB CHR-ROM banks
    chr_banks: usize,
}

impl Mapper9 {
    /// Create a new Mapper9 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let chr_rom_size = cartridge.chr_rom.len();

        let prg_banks = prg_rom_size / PRG_BANK_SIZE;
        let chr_banks = chr_rom_size / CHR_BANK_SIZE;

        Mapper9 {
            prg_rom: cartridge.prg_rom,
            chr_rom: cartridge.chr_rom,
            prg_bank: 0,
            chr_bank_0_fd: 0,
            chr_bank_0_fe: 0,
            chr_bank_1_fd: 0,
            chr_bank_1_fe: 0,
            mirroring: cartridge.mirroring,
            latch_0: Cell::new(false), // Start with $FD
            latch_1: Cell::new(false), // Start with $FD
            prg_banks,
            chr_banks,
        }
    }

    /// Get the current CHR bank for $0000-$0FFF
    fn get_chr_bank_0(&self) -> u8 {
        if self.latch_0.get() {
            self.chr_bank_0_fe
        } else {
            self.chr_bank_0_fd
        }
    }

    /// Get the current CHR bank for $1000-$1FFF
    fn get_chr_bank_1(&self) -> u8 {
        if self.latch_1.get() {
            self.chr_bank_1_fe
        } else {
            self.chr_bank_1_fd
        }
    }

    /// Update latch based on PPU access address
    fn update_latch(&self, address: u16) {
        match address {
            // Latch 0 (for $0000-$0FFF)
            0x0FD8..=0x0FDF => {
                self.latch_0.set(false); // Set to $FD
            }
            0x0FE8..=0x0FEF => {
                self.latch_0.set(true); // Set to $FE
            }
            // Latch 1 (for $1000-$1FFF)
            0x1FD8..=0x1FDF => {
                self.latch_1.set(false); // Set to $FD
            }
            0x1FE8..=0x1FEF => {
                self.latch_1.set(true); // Set to $FE
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper9 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => {
                // Switchable 8KB bank
                let offset = (address - 0x8000) as usize;
                let bank = (self.prg_bank as usize) % self.prg_banks;
                self.prg_rom[bank * PRG_BANK_SIZE + offset]
            }
            0xA000..=0xFFFF => {
                // Fixed to last 3 banks (24KB)
                let offset = (address - 0xA000) as usize;
                let base = (self.prg_banks - 3) * PRG_BANK_SIZE;
                self.prg_rom[base + offset]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0xA000..=0xAFFF => {
                // PRG-ROM bank select
                self.prg_bank = value & 0x0F;
            }
            0xB000..=0xBFFF => {
                // CHR bank for $0000-$0FFF when latch 0 = $FD
                self.chr_bank_0_fd = value & 0x1F;
            }
            0xC000..=0xCFFF => {
                // CHR bank for $0000-$0FFF when latch 0 = $FE
                self.chr_bank_0_fe = value & 0x1F;
            }
            0xD000..=0xDFFF => {
                // CHR bank for $1000-$1FFF when latch 1 = $FD
                self.chr_bank_1_fd = value & 0x1F;
            }
            0xE000..=0xEFFF => {
                // CHR bank for $1000-$1FFF when latch 1 = $FE
                self.chr_bank_1_fe = value & 0x1F;
            }
            0xF000..=0xFFFF => {
                // Mirroring control
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            _ => {}
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        // Update latches based on address (this is the critical behavior for MMC2)
        self.update_latch(address);

        match address {
            0x0000..=0x0FFF => {
                let offset = address as usize;
                let bank = (self.get_chr_bank_0() as usize) % self.chr_banks;
                self.chr_rom[bank * CHR_BANK_SIZE + offset]
            }
            0x1000..=0x1FFF => {
                let offset = (address - 0x1000) as usize;
                let bank = (self.get_chr_bank_1() as usize) % self.chr_banks;
                self.chr_rom[bank * CHR_BANK_SIZE + offset]
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, address: u16, _value: u8) {
        // Update latches on PPU access
        self.update_latch(address);
        // MMC2 uses CHR-ROM, writes are ignored
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge
    fn create_test_cartridge(prg_banks: usize, chr_banks: usize) -> Cartridge {
        let prg_rom = vec![0; prg_banks * PRG_BANK_SIZE];
        let chr_rom = vec![0; chr_banks * CHR_BANK_SIZE];

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 9,
            mirroring: Mirroring::Vertical,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper9_creation() {
        let cartridge = create_test_cartridge(16, 32);
        let mapper = Mapper9::new(cartridge);

        assert_eq!(mapper.prg_banks, 16);
        assert_eq!(mapper.chr_banks, 32);
        assert_eq!(mapper.prg_bank, 0);
        assert!(!mapper.latch_0.get());
        assert!(!mapper.latch_1.get());
    }

    #[test]
    fn test_prg_bank_switching() {
        let mut cartridge = create_test_cartridge(16, 32);

        // Fill banks with identifiable patterns
        for bank in 0..16 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper9::new(cartridge);

        // Test default bank 0
        assert_eq!(mapper.cpu_read(0x8000), 0);

        // Switch to bank 5
        mapper.cpu_write(0xA000, 0x05);
        assert_eq!(mapper.prg_bank, 5);
        assert_eq!(mapper.cpu_read(0x8000), 5);

        // Last 3 banks should be fixed (banks 13, 14, 15)
        assert_eq!(mapper.cpu_read(0xA000), 13);
        assert_eq!(mapper.cpu_read(0xC000), 14);
        assert_eq!(mapper.cpu_read(0xE000), 15);
    }

    #[test]
    fn test_chr_latch_switching() {
        let mut cartridge = create_test_cartridge(16, 32);

        // Fill CHR banks with identifiable patterns
        for bank in 0..32 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper9::new(cartridge);

        // Set CHR banks
        mapper.cpu_write(0xB000, 0x05); // Latch 0 FD = bank 5
        mapper.cpu_write(0xC000, 0x07); // Latch 0 FE = bank 7
        mapper.cpu_write(0xD000, 0x09); // Latch 1 FD = bank 9
        mapper.cpu_write(0xE000, 0x0B); // Latch 1 FE = bank 11

        // Initially latches are at FD
        assert_eq!(mapper.ppu_read(0x0000), 5); // Latch 0 = FD, bank 5
        assert_eq!(mapper.ppu_read(0x1000), 9); // Latch 1 = FD, bank 9

        // Switch latch 0 to FE
        mapper.update_latch(0x0FE8);
        assert_eq!(mapper.ppu_read(0x0000), 7); // Latch 0 = FE, bank 7

        // Switch latch 1 to FE
        mapper.update_latch(0x1FE8);
        assert_eq!(mapper.ppu_read(0x1000), 11); // Latch 1 = FE, bank 11

        // Switch latch 0 back to FD
        mapper.update_latch(0x0FD8);
        assert_eq!(mapper.ppu_read(0x0000), 5); // Latch 0 = FD, bank 5
    }

    #[test]
    fn test_latch_update_ranges() {
        let cartridge = create_test_cartridge(16, 32);
        let mapper = Mapper9::new(cartridge);

        // Test latch 0 FD range
        assert!(!mapper.latch_0.get());
        mapper.update_latch(0x0FD8);
        assert!(!mapper.latch_0.get());
        mapper.update_latch(0x0FDF);
        assert!(!mapper.latch_0.get());

        // Test latch 0 FE range
        mapper.update_latch(0x0FE8);
        assert!(mapper.latch_0.get());
        mapper.update_latch(0x0FEF);
        assert!(mapper.latch_0.get());

        // Test latch 1 FD range
        mapper.latch_1.set(true);
        mapper.update_latch(0x1FD8);
        assert!(!mapper.latch_1.get());
        mapper.update_latch(0x1FDF);
        assert!(!mapper.latch_1.get());

        // Test latch 1 FE range
        mapper.update_latch(0x1FE8);
        assert!(mapper.latch_1.get());
        mapper.update_latch(0x1FEF);
        assert!(mapper.latch_1.get());
    }

    #[test]
    fn test_mirroring_control() {
        let cartridge = create_test_cartridge(16, 32);
        let mut mapper = Mapper9::new(cartridge);

        // Initial mirroring
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        // Switch to horizontal
        mapper.cpu_write(0xF000, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        // Switch back to vertical
        mapper.cpu_write(0xF000, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn test_fixed_prg_banks() {
        let mut cartridge = create_test_cartridge(16, 32);

        // Fill all banks
        for bank in 0..16 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..PRG_BANK_SIZE {
                cartridge.prg_rom[start + i] = bank as u8;
            }
        }

        let mapper = Mapper9::new(cartridge);

        // Test that last 3 banks are properly mapped
        // $A000-$BFFF should be bank 13
        assert_eq!(mapper.cpu_read(0xA000), 13);
        assert_eq!(mapper.cpu_read(0xBFFF), 13);

        // $C000-$DFFF should be bank 14
        assert_eq!(mapper.cpu_read(0xC000), 14);
        assert_eq!(mapper.cpu_read(0xDFFF), 14);

        // $E000-$FFFF should be bank 15
        assert_eq!(mapper.cpu_read(0xE000), 15);
        assert_eq!(mapper.cpu_read(0xFFFF), 15);
    }

    #[test]
    fn test_independent_latches() {
        let mut cartridge = create_test_cartridge(16, 32);

        for bank in 0..32 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper9::new(cartridge);

        // Set up different banks for each latch state
        mapper.cpu_write(0xB000, 0x02); // Latch 0 FD
        mapper.cpu_write(0xC000, 0x04); // Latch 0 FE
        mapper.cpu_write(0xD000, 0x06); // Latch 1 FD
        mapper.cpu_write(0xE000, 0x08); // Latch 1 FE

        // Both latches start at FD
        assert_eq!(mapper.ppu_read(0x0000), 2);
        assert_eq!(mapper.ppu_read(0x1000), 6);

        // Switch only latch 0 to FE
        mapper.update_latch(0x0FE8);
        assert_eq!(mapper.ppu_read(0x0000), 4);
        assert_eq!(mapper.ppu_read(0x1000), 6); // Latch 1 unchanged

        // Switch only latch 1 to FE
        mapper.update_latch(0x1FE8);
        assert_eq!(mapper.ppu_read(0x0000), 4); // Latch 0 unchanged
        assert_eq!(mapper.ppu_read(0x1000), 8);
    }
}
