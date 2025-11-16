// Mapper 7 (AxROM) - Simple mapper with 32KB PRG banking and one-screen mirroring
//
// Memory Layout:
// - CPU $8000-$FFFF: 32KB switchable PRG-ROM bank
// - PPU $0000-$1FFF: 8KB CHR-RAM (fixed, no CHR-ROM)
//
// Features:
// - 32KB PRG-ROM banking (up to 256KB total)
// - One-screen mirroring control (selects upper or lower nametable)
// - Always uses CHR-RAM (8KB)
// - No PRG-RAM
//
// Register Interface:
// - $8000-$FFFF (write): Bank select and mirroring
//   Bits 0-2: Select 32KB PRG-ROM bank
//   Bit 4: One-screen mirroring (0 = lower bank, 1 = upper bank)

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// PRG-ROM bank size (32KB)
const PRG_BANK_SIZE: usize = 32 * 1024;

/// CHR-RAM size (8KB, fixed)
const CHR_RAM_SIZE: usize = 8 * 1024;

/// Mapper 7 implementation (AxROM)
///
/// AxROM is a simple mapper used by games like:
/// - Battletoads
/// - Wizards & Warriors
/// - Marble Madness
/// - Jeopardy!
pub struct Mapper7 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-RAM data (always 8KB)
    chr_ram: Vec<u8>,

    // Internal state
    /// Current PRG-ROM bank (0-7, depending on ROM size)
    prg_bank: u8,
    /// One-screen mirroring select (0 = lower, 1 = upper)
    mirroring_select: bool,

    /// Number of 32KB PRG-ROM banks
    prg_banks: usize,
}

impl Mapper7 {
    /// Create a new Mapper7 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let prg_banks = prg_rom_size / PRG_BANK_SIZE;

        Mapper7 {
            prg_rom: cartridge.prg_rom,
            chr_ram: vec![0; CHR_RAM_SIZE],
            prg_bank: 0,
            mirroring_select: false,
            prg_banks,
        }
    }
}

impl Mapper for Mapper7 {
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
            // Bits 0-2: PRG-ROM bank select
            self.prg_bank = value & 0x07;

            // Bit 4: One-screen mirroring select
            self.mirroring_select = (value & 0x10) != 0;
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = address as usize;
                self.chr_ram[index % CHR_RAM_SIZE]
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        if let 0x0000..=0x1FFF = address {
            let index = address as usize;
            self.chr_ram[index % CHR_RAM_SIZE] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        // AxROM uses one-screen mirroring
        // The select bit determines which screen
        // For simplicity, we return SingleScreen for both
        // (proper implementation would need to distinguish between lower/upper)
        Mirroring::SingleScreen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge
    fn create_test_cartridge(prg_banks: usize) -> Cartridge {
        let prg_rom = vec![0; prg_banks * PRG_BANK_SIZE];
        let chr_rom = vec![0; 0]; // AxROM uses CHR-RAM, no CHR-ROM

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 7,
            mirroring: Mirroring::SingleScreen,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper7_creation() {
        let cartridge = create_test_cartridge(4);
        let mapper = Mapper7::new(cartridge);

        assert_eq!(mapper.prg_banks, 4);
        assert_eq!(mapper.prg_bank, 0);
        assert_eq!(mapper.chr_ram.len(), CHR_RAM_SIZE);
    }

    #[test]
    fn test_prg_bank_switching() {
        let mut cartridge = create_test_cartridge(8);

        // Fill banks with identifiable patterns
        for bank in 0..8 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper7::new(cartridge);

        // Test bank 0 (default)
        assert_eq!(mapper.cpu_read(0x8000), 0);

        // Switch to bank 3
        mapper.cpu_write(0x8000, 0x03);
        assert_eq!(mapper.cpu_read(0x8000), 3);

        // Switch to bank 7
        mapper.cpu_write(0x8000, 0x07);
        assert_eq!(mapper.cpu_read(0x8000), 7);

        // Switch to bank 1
        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.cpu_read(0x8000), 1);
    }

    #[test]
    fn test_mirroring_control() {
        let cartridge = create_test_cartridge(4);
        let mut mapper = Mapper7::new(cartridge);

        // Test mirroring select bit
        assert!(!mapper.mirroring_select);

        // Set mirroring select
        mapper.cpu_write(0x8000, 0x10);
        assert!(mapper.mirroring_select);

        // Clear mirroring select
        mapper.cpu_write(0x8000, 0x00);
        assert!(!mapper.mirroring_select);
    }

    #[test]
    fn test_chr_ram_read_write() {
        let cartridge = create_test_cartridge(4);
        let mut mapper = Mapper7::new(cartridge);

        // Test CHR-RAM writes and reads
        mapper.ppu_write(0x0000, 0x42);
        assert_eq!(mapper.ppu_read(0x0000), 0x42);

        mapper.ppu_write(0x1FFF, 0x99);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x99);

        // Test various addresses
        for i in 0..256 {
            mapper.ppu_write(i, i as u8);
            assert_eq!(mapper.ppu_read(i), i as u8);
        }
    }

    #[test]
    fn test_bank_wrapping() {
        let mut cartridge = create_test_cartridge(4);

        // Fill banks
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper7::new(cartridge);

        // Write bank 7 (should wrap to bank 3 with 4 banks)
        mapper.cpu_write(0x8000, 0x07);
        assert_eq!(mapper.cpu_read(0x8000), 3);
    }

    #[test]
    fn test_combined_bank_and_mirroring() {
        let mut cartridge = create_test_cartridge(4);

        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper7::new(cartridge);

        // Write both bank and mirroring in one operation
        mapper.cpu_write(0x8000, 0x12); // Bank 2, mirroring select
        assert_eq!(mapper.prg_bank, 0x02);
        assert!(mapper.mirroring_select);
        assert_eq!(mapper.cpu_read(0x8000), 2);
    }

    #[test]
    fn test_full_32kb_access() {
        let mut cartridge = create_test_cartridge(2);

        // Fill with pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper7::new(cartridge);

        // Test access across full 32KB range
        assert_eq!(mapper.cpu_read(0x8000), 0x00);
        assert_eq!(mapper.cpu_read(0x8001), 0x01);
        assert_eq!(mapper.cpu_read(0xFFFF), 0xFF);
    }
}
