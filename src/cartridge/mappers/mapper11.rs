// Mapper 11 (Color Dreams) - Simple mapper with PRG and CHR banking
//
// Memory Layout:
// - CPU $8000-$FFFF: 32KB switchable PRG-ROM bank
// - PPU $0000-$1FFF: 8KB switchable CHR-ROM bank
//
// Features:
// - 32KB PRG-ROM banking (up to 128KB total, 4 banks)
// - 8KB CHR-ROM banking (up to 128KB total, 16 banks)
// - No mirroring control (fixed in cartridge header)
// - No PRG-RAM
//
// Register Interface:
// - $8000-$FFFF (write): Bank select
//   Bits 0-3: Select 8KB CHR-ROM bank
//   Bits 4-5: Select 32KB PRG-ROM bank
//
// Note: Mapper 11 is very similar to Mapper 66, but with different
// bit assignments and support for more CHR banks.

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// PRG-ROM bank size (32KB)
const PRG_BANK_SIZE: usize = 32 * 1024;

/// CHR-ROM bank size (8KB)
const CHR_BANK_SIZE: usize = 8 * 1024;

/// Mapper 11 implementation (Color Dreams)
///
/// Color Dreams is used by unlicensed games like:
/// - Crystal Mines
/// - Bible Adventures
/// - Sunday Funday
/// - Menace Beach
pub struct Mapper11 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-ROM data
    chr_rom: Vec<u8>,

    // Internal state
    /// Current PRG-ROM bank (0-3)
    prg_bank: u8,
    /// Current CHR-ROM bank (0-15)
    chr_bank: u8,
    /// Mirroring type (fixed)
    mirroring: Mirroring,

    /// Number of 32KB PRG-ROM banks
    prg_banks: usize,
    /// Number of 8KB CHR-ROM banks
    chr_banks: usize,
}

impl Mapper11 {
    /// Create a new Mapper11 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let chr_rom_size = cartridge.chr_rom.len();

        let prg_banks = prg_rom_size / PRG_BANK_SIZE;
        let chr_banks = chr_rom_size / CHR_BANK_SIZE;

        Mapper11 {
            prg_rom: cartridge.prg_rom,
            chr_rom: cartridge.chr_rom,
            prg_bank: 0,
            chr_bank: 0,
            mirroring: cartridge.mirroring,
            prg_banks,
            chr_banks,
        }
    }
}

impl Mapper for Mapper11 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => {
                // Map to selected 32KB PRG-ROM bank
                let offset = (address - 0x8000) as usize;
                let bank = (self.prg_bank as usize) % self.prg_banks;
                self.prg_rom[bank * PRG_BANK_SIZE + offset]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        if let 0x8000..=0xFFFF = address {
            // Bits 0-3: CHR-ROM bank select
            self.chr_bank = value & 0x0F;

            // Bits 4-5: PRG-ROM bank select
            self.prg_bank = (value >> 4) & 0x03;
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                // Map to selected 8KB CHR-ROM bank
                let offset = address as usize;
                let bank = (self.chr_bank as usize) % self.chr_banks;
                self.chr_rom[bank * CHR_BANK_SIZE + offset]
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, _address: u16, _value: u8) {
        // Color Dreams uses CHR-ROM, writes are ignored
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
            mapper: 11,
            mirroring: Mirroring::Vertical,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper11_creation() {
        let cartridge = create_test_cartridge(4, 16);
        let mapper = Mapper11::new(cartridge);

        assert_eq!(mapper.prg_banks, 4);
        assert_eq!(mapper.chr_banks, 16);
        assert_eq!(mapper.prg_bank, 0);
        assert_eq!(mapper.chr_bank, 0);
    }

    #[test]
    fn test_prg_bank_switching() {
        let mut cartridge = create_test_cartridge(4, 16);

        // Fill PRG banks with identifiable patterns
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = (bank * 10) as u8;
        }

        let mut mapper = Mapper11::new(cartridge);

        // Test bank 0 (default)
        assert_eq!(mapper.cpu_read(0x8000), 0);

        // Switch to bank 1 (bits 4-5 = 01)
        mapper.cpu_write(0x8000, 0x10);
        assert_eq!(mapper.prg_bank, 1);
        assert_eq!(mapper.cpu_read(0x8000), 10);

        // Switch to bank 2 (bits 4-5 = 10)
        mapper.cpu_write(0x8000, 0x20);
        assert_eq!(mapper.prg_bank, 2);
        assert_eq!(mapper.cpu_read(0x8000), 20);

        // Switch to bank 3 (bits 4-5 = 11)
        mapper.cpu_write(0x8000, 0x30);
        assert_eq!(mapper.prg_bank, 3);
        assert_eq!(mapper.cpu_read(0x8000), 30);
    }

    #[test]
    fn test_chr_bank_switching() {
        let mut cartridge = create_test_cartridge(4, 16);

        // Fill CHR banks with identifiable patterns
        for bank in 0..16 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = (bank * 5) as u8;
        }

        let mut mapper = Mapper11::new(cartridge);

        // Test various CHR banks
        for bank in 0..16 {
            mapper.cpu_write(0x8000, bank);
            assert_eq!(mapper.chr_bank, bank);
            assert_eq!(mapper.ppu_read(0x0000), bank * 5);
        }
    }

    #[test]
    fn test_combined_prg_chr_switching() {
        let mut cartridge = create_test_cartridge(4, 16);

        // Fill banks
        for bank in 0..4 {
            let prg_start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[prg_start] = (bank * 10) as u8;
        }

        for bank in 0..16 {
            let chr_start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[chr_start] = (bank * 5) as u8;
        }

        let mut mapper = Mapper11::new(cartridge);

        // Switch both PRG and CHR banks
        // PRG bank 2 (bits 4-5 = 10), CHR bank 7 (bits 0-3 = 0111)
        mapper.cpu_write(0x8000, 0x27);
        assert_eq!(mapper.prg_bank, 2);
        assert_eq!(mapper.chr_bank, 7);
        assert_eq!(mapper.cpu_read(0x8000), 20);
        assert_eq!(mapper.ppu_read(0x0000), 35);
    }

    #[test]
    fn test_chr_bank_masking() {
        let mut cartridge = create_test_cartridge(4, 8);

        // Fill CHR banks
        for bank in 0..8 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = (bank * 10) as u8;
        }

        let mut mapper = Mapper11::new(cartridge);

        // Write CHR bank 15 (0x0F), should wrap to bank 7 with 8 banks
        mapper.cpu_write(0x8000, 0x0F);
        assert_eq!(mapper.chr_bank, 15);
        assert_eq!(mapper.ppu_read(0x0000), 70); // Bank 7
    }

    #[test]
    fn test_mirroring_fixed() {
        let cartridge = create_test_cartridge(4, 16);
        let mapper = Mapper11::new(cartridge);

        // Mirroring should be fixed
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        // Create another with horizontal mirroring
        let mut cartridge2 = create_test_cartridge(4, 16);
        cartridge2.mirroring = Mirroring::Horizontal;
        let mapper2 = Mapper11::new(cartridge2);
        assert_eq!(mapper2.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn test_chr_rom_write_ignored() {
        let cartridge = create_test_cartridge(4, 16);
        let mut mapper = Mapper11::new(cartridge);

        // CHR-ROM writes should be ignored
        let original = mapper.ppu_read(0x0000);
        mapper.ppu_write(0x0000, 0xFF);
        assert_eq!(mapper.ppu_read(0x0000), original);
    }

    #[test]
    fn test_full_32kb_prg_access() {
        let mut cartridge = create_test_cartridge(2, 8);

        // Fill with pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper11::new(cartridge);

        // Test access across full 32KB range
        assert_eq!(mapper.cpu_read(0x8000), 0x00);
        assert_eq!(mapper.cpu_read(0x8001), 0x01);
        assert_eq!(mapper.cpu_read(0xFFFF), 0xFF);
    }

    #[test]
    fn test_sequential_bank_switching() {
        let mut cartridge = create_test_cartridge(4, 16);

        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper11::new(cartridge);

        // Switch through all PRG banks sequentially
        for bank in 0..4 {
            mapper.cpu_write(0x8000, (bank << 4) as u8);
            assert_eq!(mapper.prg_bank, bank as u8);
            assert_eq!(mapper.cpu_read(0x8000), bank as u8);
        }
    }
}
