// Mapper 2 (UxROM) - Switchable PRG-ROM with fixed CHR-RAM
//
// Memory Layout:
// - CPU $8000-$BFFF: 16KB switchable PRG-ROM bank
// - CPU $C000-$FFFF: 16KB fixed PRG-ROM bank (last bank)
// - PPU $0000-$1FFF: 8KB CHR-RAM (writable)
//
// Features:
// - PRG-ROM: up to 4MB (256 banks of 16KB each)
// - CHR-RAM: 8KB (always writable)
// - No CHR-ROM support (always uses RAM)
// - Simple bank switching (write to $8000-$FFFF)
//
// Bank Switching:
// - Any write to $8000-$FFFF selects the PRG-ROM bank for $8000-$BFFF
// - The value written is the bank number (lower bits used)
// - Last bank is always fixed at $C000-$FFFF
//
// Games using Mapper 2:
// - Mega Man
// - Castlevania
// - Contra
// - Duck Tales
// - Metal Gear

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// PRG-ROM bank size (16KB)
const PRG_BANK_SIZE: usize = 16 * 1024;

/// CHR-RAM size (8KB)
const CHR_RAM_SIZE: usize = 8 * 1024;

/// Mapper 2 implementation (UxROM)
///
/// UxROM is a simple mapper featuring switchable PRG-ROM banks and fixed CHR-RAM.
/// It's used by many popular games including Mega Man, Castlevania, and Contra.
pub struct Mapper2 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-RAM data (8KB, always writable)
    chr_ram: Vec<u8>,
    /// Currently selected PRG-ROM bank (for $8000-$BFFF)
    prg_bank: u8,
    /// Total number of 16KB PRG-ROM banks
    prg_banks: usize,
    /// Mirroring type (fixed, cannot be changed by the mapper)
    mirroring: Mirroring,
}

impl Mapper2 {
    /// Create a new Mapper2 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    ///
    /// # Panics
    /// Panics if PRG-ROM size is not a multiple of 16KB
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();

        // Validate PRG-ROM size (must be a multiple of 16KB)
        assert!(
            prg_rom_size.is_multiple_of(PRG_BANK_SIZE) && prg_rom_size > 0,
            "Mapper 2 requires PRG-ROM size to be a multiple of 16KB, got {} bytes",
            prg_rom_size
        );

        // Calculate number of 16KB banks
        let prg_banks = prg_rom_size / PRG_BANK_SIZE;

        // UxROM always uses CHR-RAM (8KB), not CHR-ROM
        let chr_ram = vec![0; CHR_RAM_SIZE];

        Mapper2 {
            prg_rom: cartridge.prg_rom,
            chr_ram,
            prg_bank: 0, // Initialize to bank 0
            prg_banks,
            mirroring: cartridge.mirroring,
        }
    }

    /// Map CPU address to PRG-ROM offset
    ///
    /// # Arguments
    /// * `address` - CPU address in range $8000-$FFFF
    ///
    /// # Returns
    /// The offset into the PRG-ROM vector
    fn map_prg_address(&self, address: u16) -> usize {
        match address {
            0x8000..=0xBFFF => {
                // Switchable bank at $8000-$BFFF
                let bank_offset = (address - 0x8000) as usize;
                let bank = (self.prg_bank as usize) % self.prg_banks;
                bank * PRG_BANK_SIZE + bank_offset
            }
            0xC000..=0xFFFF => {
                // Fixed last bank at $C000-$FFFF
                let bank_offset = (address - 0xC000) as usize;
                let last_bank = self.prg_banks - 1;
                last_bank * PRG_BANK_SIZE + bank_offset
            }
            _ => 0, // Should not happen
        }
    }
}

impl Mapper for Mapper2 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => {
                let index = self.map_prg_address(address);
                self.prg_rom[index]
            }
            _ => 0, // Unmapped address
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0xFFFF => {
                // Any write to $8000-$FFFF selects the PRG-ROM bank
                // Only the lower bits are used (enough to address all banks)
                self.prg_bank = value;
            }
            _ => {
                // Writes to other addresses are ignored
            }
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = address as usize;
                self.chr_ram[index % CHR_RAM_SIZE]
            }
            _ => 0, // Unmapped address
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // CHR-RAM is always writable
                let index = address as usize;
                self.chr_ram[index % CHR_RAM_SIZE] = value;
            }
            _ => {
                // Unmapped address
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn prg_ram(&self) -> Option<&[u8]> {
        // UxROM doesn't typically have PRG-RAM
        None
    }

    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        // UxROM doesn't typically have PRG-RAM
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge
    fn create_test_cartridge(prg_banks: usize, mirroring: Mirroring) -> Cartridge {
        let prg_rom = vec![0; prg_banks * PRG_BANK_SIZE];
        let chr_rom = vec![0; CHR_RAM_SIZE]; // UxROM uses CHR-RAM, not CHR-ROM

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 2,
            mirroring,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper2_creation() {
        let cartridge = create_test_cartridge(8, Mirroring::Horizontal);
        let mapper = Mapper2::new(cartridge);

        assert_eq!(mapper.prg_banks, 8);
        assert_eq!(mapper.chr_ram.len(), CHR_RAM_SIZE);
        assert_eq!(mapper.prg_bank, 0);
        assert_eq!(mapper.mirroring, Mirroring::Horizontal);
    }

    #[test]
    #[should_panic(expected = "Mapper 2 requires PRG-ROM size to be a multiple of 16KB")]
    fn test_mapper2_invalid_prg_size() {
        let mut cartridge = create_test_cartridge(2, Mirroring::Horizontal);
        cartridge.prg_rom = vec![0; 10 * 1024]; // 10KB, not a multiple of 16KB
        Mapper2::new(cartridge);
    }

    #[test]
    fn test_prg_bank_switching() {
        let mut cartridge = create_test_cartridge(4, Mirroring::Horizontal);

        // Fill each bank with identifiable pattern
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..PRG_BANK_SIZE {
                cartridge.prg_rom[start + i] = (bank as u8).wrapping_mul(0x10);
            }
        }

        let mut mapper = Mapper2::new(cartridge);

        // Initially, bank 0 should be at $8000-$BFFF
        assert_eq!(mapper.cpu_read(0x8000), 0x00);
        // Last bank (bank 3) should always be at $C000-$FFFF
        assert_eq!(mapper.cpu_read(0xC000), 0x30);

        // Switch to bank 1
        mapper.cpu_write(0x8000, 1);
        assert_eq!(mapper.cpu_read(0x8000), 0x10);
        // Last bank should still be at $C000
        assert_eq!(mapper.cpu_read(0xC000), 0x30);

        // Switch to bank 2
        mapper.cpu_write(0xFFFF, 2); // Any address in $8000-$FFFF works
        assert_eq!(mapper.cpu_read(0x8000), 0x20);
        assert_eq!(mapper.cpu_read(0xC000), 0x30);
    }

    #[test]
    fn test_fixed_last_bank() {
        let mut cartridge = create_test_cartridge(8, Mirroring::Horizontal);

        // Fill banks with identifiable patterns
        for bank in 0..8 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..256 {
                cartridge.prg_rom[start + i] = (bank as u8).wrapping_mul(0x11);
            }
        }

        let mut mapper = Mapper2::new(cartridge);

        // Test that last bank (bank 7) is always at $C000-$FFFF
        for bank_num in 0..8 {
            mapper.cpu_write(0x8000, bank_num);
            // Last bank should always be bank 7
            assert_eq!(
                mapper.cpu_read(0xC000),
                0x77,
                "Last bank changed when switching to bank {}",
                bank_num
            );
        }
    }

    #[test]
    fn test_chr_ram_read_write() {
        let cartridge = create_test_cartridge(2, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Test writing and reading CHR-RAM
        mapper.ppu_write(0x0000, 0x42);
        assert_eq!(mapper.ppu_read(0x0000), 0x42);

        mapper.ppu_write(0x1FFF, 0x99);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x99);

        // Test pattern across CHR-RAM
        for i in 0..256 {
            mapper.ppu_write(i, i as u8);
        }
        for i in 0..256 {
            assert_eq!(mapper.ppu_read(i), i as u8);
        }
    }

    #[test]
    fn test_chr_ram_persistence() {
        let cartridge = create_test_cartridge(4, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Write pattern to CHR-RAM
        for i in 0..CHR_RAM_SIZE {
            mapper.ppu_write(i as u16, (i & 0xFF) as u8);
        }

        // Switch PRG banks and verify CHR-RAM persists
        for bank in 0..4 {
            mapper.cpu_write(0x8000, bank);
            // Check a few CHR-RAM locations
            assert_eq!(mapper.ppu_read(0x0000), 0x00);
            assert_eq!(mapper.ppu_read(0x0100), 0x00);
            assert_eq!(mapper.ppu_read(0x1234), 0x34);
        }
    }

    #[test]
    fn test_bank_wrapping() {
        let cartridge = create_test_cartridge(4, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Write bank number larger than available banks
        mapper.cpu_write(0x8000, 255);

        // Bank register stores the raw value
        assert_eq!(mapper.prg_bank, 255);

        // Reading should still work (wrapping handled in map_prg_address)
        let _ = mapper.cpu_read(0x8000); // Should not panic
    }

    #[test]
    fn test_mirroring_modes() {
        // Test horizontal mirroring
        let cartridge_h = create_test_cartridge(2, Mirroring::Horizontal);
        let mapper_h = Mapper2::new(cartridge_h);
        assert_eq!(mapper_h.mirroring(), Mirroring::Horizontal);

        // Test vertical mirroring
        let cartridge_v = create_test_cartridge(2, Mirroring::Vertical);
        let mapper_v = Mapper2::new(cartridge_v);
        assert_eq!(mapper_v.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn test_prg_bank_boundaries() {
        let mut cartridge = create_test_cartridge(2, Mirroring::Horizontal);

        // Fill with sequential pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mut mapper = Mapper2::new(cartridge);

        // Test bank 0 boundaries
        mapper.cpu_write(0x8000, 0);
        assert_eq!(mapper.cpu_read(0x8000), 0x00); // First byte of bank 0
        assert_eq!(mapper.cpu_read(0xBFFF), 0xFF); // Last byte of bank 0 (0x3FFF & 0xFF)

        // Test fixed last bank boundaries
        assert_eq!(mapper.cpu_read(0xC000), 0x00); // First byte of bank 1 (0x4000 & 0xFF)
        assert_eq!(mapper.cpu_read(0xFFFF), 0xFF); // Last byte of bank 1 (0x7FFF & 0xFF)
    }

    #[test]
    fn test_bank_switching_all_registers() {
        let mut cartridge = create_test_cartridge(8, Mirroring::Horizontal);

        // Fill banks with distinct values
        for bank in 0..8 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper2::new(cartridge);

        // Test that writes to any address in $8000-$FFFF work
        let test_addresses = [
            0x8000, 0x9000, 0xA000, 0xB000, 0xC000, 0xD000, 0xE000, 0xF000, 0xFFFF,
        ];

        for (i, &addr) in test_addresses.iter().enumerate() {
            let bank = i % 8;
            mapper.cpu_write(addr, bank as u8);
            assert_eq!(
                mapper.cpu_read(0x8000),
                bank as u8,
                "Bank switching failed at address {:#X}",
                addr
            );
        }
    }

    #[test]
    fn test_large_rom() {
        // Test with maximum practical size (256 banks = 4MB)
        let cartridge = create_test_cartridge(256, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        assert_eq!(mapper.prg_banks, 256);

        // Switch to various banks
        mapper.cpu_write(0x8000, 0);
        let _ = mapper.cpu_read(0x8000);

        mapper.cpu_write(0x8000, 128);
        let _ = mapper.cpu_read(0x8000);

        mapper.cpu_write(0x8000, 255);
        let _ = mapper.cpu_read(0x8000);
    }

    #[test]
    fn test_chr_ram_full_range() {
        let cartridge = create_test_cartridge(2, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Test all 8KB of CHR-RAM
        let test_addresses = [
            0x0000, 0x0001, 0x00FF, 0x0100, 0x07FF, 0x0800, 0x0FFF, 0x1000, 0x17FF, 0x1800, 0x1FFF,
        ];

        for &addr in &test_addresses {
            let value = ((addr ^ 0xAA) & 0xFF) as u8;
            mapper.ppu_write(addr, value);
            assert_eq!(
                mapper.ppu_read(addr),
                value,
                "CHR-RAM access failed at {:#X}",
                addr
            );
        }
    }

    #[test]
    fn test_no_prg_ram() {
        let cartridge = create_test_cartridge(2, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // UxROM doesn't have PRG-RAM
        assert!(mapper.prg_ram().is_none());
        assert!(mapper.prg_ram_mut().is_none());
    }

    #[test]
    fn test_multiple_bank_switches() {
        let mut cartridge = create_test_cartridge(4, Mirroring::Horizontal);

        // Fill banks with sequential values
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..PRG_BANK_SIZE {
                cartridge.prg_rom[start + i] = ((bank * 0x40) + (i / 1024)) as u8;
            }
        }

        let mut mapper = Mapper2::new(cartridge);

        // Perform multiple bank switches and verify
        for iteration in 0..3 {
            for bank in 0..4 {
                mapper.cpu_write(0x8000, bank);
                let expected = bank * 0x40;
                assert_eq!(
                    mapper.cpu_read(0x8000),
                    expected as u8,
                    "Iteration {}, Bank {}",
                    iteration,
                    bank
                );
            }
        }
    }

    #[test]
    fn test_chr_ram_after_bank_switch() {
        let cartridge = create_test_cartridge(4, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Write to CHR-RAM
        mapper.ppu_write(0x1000, 0xAB);

        // Switch banks multiple times
        for bank in 0..4 {
            mapper.cpu_write(0x8000, bank);
        }

        // CHR-RAM should be unchanged
        assert_eq!(mapper.ppu_read(0x1000), 0xAB);
    }

    #[test]
    fn test_sequential_prg_reads() {
        let mut cartridge = create_test_cartridge(2, Mirroring::Horizontal);

        // Fill with sequential bytes
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i % 256) as u8;
        }

        let mapper = Mapper2::new(cartridge);

        // Test sequential reads from both banks
        for i in 0..256 {
            assert_eq!(mapper.cpu_read(0x8000 + i), (i as u8));
            assert_eq!(mapper.cpu_read(0xC000 + i), (i as u8));
        }
    }

    #[test]
    fn test_bank_select_value_preservation() {
        let cartridge = create_test_cartridge(4, Mirroring::Horizontal);
        let mut mapper = Mapper2::new(cartridge);

        // Write specific bank values
        mapper.cpu_write(0x8000, 2);
        assert_eq!(mapper.prg_bank, 2);

        // Write another value
        mapper.cpu_write(0xC000, 3);
        assert_eq!(mapper.prg_bank, 3);

        // Value should be preserved even if out of range
        mapper.cpu_write(0x8000, 42);
        assert_eq!(mapper.prg_bank, 42);
    }

    #[test]
    fn test_minimum_rom_size() {
        // Test with minimum size (2 banks = 32KB)
        let cartridge = create_test_cartridge(2, Mirroring::Vertical);
        let mut mapper = Mapper2::new(cartridge);

        assert_eq!(mapper.prg_banks, 2);

        // Switch between the two banks
        mapper.cpu_write(0x8000, 0);
        let _ = mapper.cpu_read(0x8000);

        mapper.cpu_write(0x8000, 1);
        let _ = mapper.cpu_read(0x8000);

        // Last bank is always bank 1
        let _ = mapper.cpu_read(0xC000);
    }
}
