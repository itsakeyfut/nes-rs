// Mapper 3 (CNROM) - Fixed PRG-ROM with switchable CHR-ROM
//
// Memory Layout:
// - CPU $8000-$BFFF: First 16KB of PRG-ROM
// - CPU $C000-$FFFF: Last 16KB of PRG-ROM (or mirror if only 16KB total)
// - PPU $0000-$1FFF: 8KB switchable CHR-ROM bank
//
// Features:
// - PRG-ROM: 16KB or 32KB (fixed, no switching)
// - CHR-ROM: up to 2048KB (256 banks of 8KB each)
// - Simple CHR bank switching (write to $8000-$FFFF)
//
// Bank Switching:
// - Any write to $8000-$FFFF selects the CHR-ROM bank for $0000-$1FFF
// - The value written is the bank number (lower bits used)
// - PRG-ROM is fixed (no bank switching)
//
// Games using Mapper 3:
// - Arkanoid
// - Paperboy
// - Q*bert

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// CHR-ROM bank size (8KB)
const CHR_BANK_SIZE: usize = 8 * 1024;

/// Mapper 3 implementation (CNROM)
///
/// CNROM is a simple mapper featuring fixed PRG-ROM and switchable CHR-ROM banks.
/// It's the inverse of UxROM (Mapper 2), which has switchable PRG and fixed CHR.
pub struct Mapper3 {
    /// PRG-ROM data (16KB or 32KB, fixed)
    prg_rom: Vec<u8>,
    /// CHR-ROM data (multiple 8KB banks)
    chr_rom: Vec<u8>,
    /// Currently selected CHR-ROM bank (for $0000-$1FFF)
    chr_bank: u8,
    /// Total number of 8KB CHR-ROM banks
    chr_banks: usize,
    /// Mirroring type (fixed, cannot be changed by the mapper)
    mirroring: Mirroring,
}

impl Mapper3 {
    /// Create a new Mapper3 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    ///
    /// # Panics
    /// Panics if PRG-ROM size is not 16KB or 32KB, or if CHR-ROM size is not a multiple of 8KB
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let chr_rom_size = cartridge.chr_rom.len();

        // Validate PRG-ROM size (must be 16KB or 32KB)
        assert!(
            prg_rom_size == 16 * 1024 || prg_rom_size == 32 * 1024,
            "Mapper 3 requires 16KB or 32KB PRG-ROM, got {} bytes",
            prg_rom_size
        );

        // Validate CHR-ROM size (must be a multiple of 8KB and at least 8KB)
        // Note: Using explicit modulo for stable Rust compatibility (is_multiple_of is nightly-only)
        #[allow(clippy::manual_is_multiple_of)]
        {
            assert!(
                chr_rom_size % CHR_BANK_SIZE == 0 && chr_rom_size > 0,
                "Mapper 3 requires CHR-ROM size to be a multiple of 8KB, got {} bytes",
                chr_rom_size
            );
        }

        // Calculate number of 8KB CHR banks
        let chr_banks = chr_rom_size / CHR_BANK_SIZE;

        Mapper3 {
            prg_rom: cartridge.prg_rom,
            chr_rom: cartridge.chr_rom,
            chr_bank: 0, // Initialize to bank 0
            chr_banks,
            mirroring: cartridge.mirroring,
        }
    }

    /// Map PPU address to CHR-ROM offset
    ///
    /// # Arguments
    /// * `address` - PPU address in range $0000-$1FFF
    ///
    /// # Returns
    /// The offset into the CHR-ROM vector
    fn map_chr_address(&self, address: u16) -> usize {
        let bank_offset = (address & 0x1FFF) as usize;
        let bank = (self.chr_bank as usize) % self.chr_banks;
        bank * CHR_BANK_SIZE + bank_offset
    }
}

impl Mapper for Mapper3 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => {
                // Map address to PRG-ROM index
                let index = (address - 0x8000) as usize;
                // Use modulo to handle mirroring for 16KB ROMs
                // For 32KB ROMs, modulo has no effect since index < prg_rom.len()
                self.prg_rom[index % self.prg_rom.len()]
            }
            _ => 0, // Unmapped address
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0xFFFF => {
                // Any write to $8000-$FFFF selects the CHR-ROM bank
                // Only the lower bits are used (enough to address all banks)
                self.chr_bank = value;
            }
            _ => {
                // Writes to other addresses are ignored
            }
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = self.map_chr_address(address);
                self.chr_rom[index]
            }
            _ => 0, // Unmapped address
        }
    }

    fn ppu_write(&mut self, _address: u16, _value: u8) {
        // CHR-ROM is read-only, writes are ignored
        // (Some variants may have CHR-RAM, but standard CNROM uses ROM)
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn prg_ram(&self) -> Option<&[u8]> {
        // CNROM doesn't typically have PRG-RAM
        None
    }

    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        // CNROM doesn't typically have PRG-RAM
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge
    fn create_test_cartridge(
        prg_size: usize,
        chr_banks: usize,
        mirroring: Mirroring,
    ) -> Cartridge {
        let prg_rom = vec![0; prg_size];
        let chr_rom = vec![0; chr_banks * CHR_BANK_SIZE];

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 3,
            mirroring,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper3_creation() {
        let cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);
        let mapper = Mapper3::new(cartridge);

        assert_eq!(mapper.chr_banks, 4);
        assert_eq!(mapper.chr_rom.len(), 4 * CHR_BANK_SIZE);
        assert_eq!(mapper.chr_bank, 0);
        assert_eq!(mapper.mirroring, Mirroring::Horizontal);
    }

    #[test]
    #[should_panic(expected = "Mapper 3 requires 16KB or 32KB PRG-ROM")]
    fn test_mapper3_invalid_prg_size() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        cartridge.prg_rom = vec![0; 64 * 1024]; // 64KB is not valid for Mapper 3
        Mapper3::new(cartridge);
    }

    #[test]
    #[should_panic(expected = "Mapper 3 requires CHR-ROM size to be a multiple of 8KB")]
    fn test_mapper3_invalid_chr_size() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        cartridge.chr_rom = vec![0; 10 * 1024]; // 10KB is not a multiple of 8KB
        Mapper3::new(cartridge);
    }

    #[test]
    fn test_prg_rom_fixed_16kb() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);

        // Fill PRG-ROM with identifiable pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper3::new(cartridge);

        // Test first 16KB ($8000-$BFFF)
        assert_eq!(mapper.cpu_read(0x8000), 0x00);
        assert_eq!(mapper.cpu_read(0x8001), 0x01);
        assert_eq!(mapper.cpu_read(0xBFFF), 0xFF); // (0x3FFF & 0xFF)

        // Test mirrored 16KB ($C000-$FFFF) - should mirror $8000-$BFFF
        assert_eq!(mapper.cpu_read(0xC000), 0x00); // Same as 0x8000
        assert_eq!(mapper.cpu_read(0xC001), 0x01); // Same as 0x8001
        assert_eq!(mapper.cpu_read(0xFFFF), 0xFF); // Same as 0xBFFF
    }

    #[test]
    fn test_prg_rom_fixed_32kb() {
        let mut cartridge = create_test_cartridge(32 * 1024, 2, Mirroring::Horizontal);

        // Fill PRG-ROM with identifiable pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper3::new(cartridge);

        // Test first 16KB ($8000-$BFFF)
        assert_eq!(mapper.cpu_read(0x8000), 0x00);
        assert_eq!(mapper.cpu_read(0x8001), 0x01);
        assert_eq!(mapper.cpu_read(0xBFFF), 0xFF);

        // Test second 16KB ($C000-$FFFF) - should be different from first 16KB
        assert_eq!(mapper.cpu_read(0xC000), 0x00); // (0x4000 & 0xFF)
        assert_eq!(mapper.cpu_read(0xC001), 0x01); // (0x4001 & 0xFF)
        assert_eq!(mapper.cpu_read(0xFFFF), 0xFF); // (0x7FFF & 0xFF)
    }

    #[test]
    fn test_chr_bank_switching() {
        let mut cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);

        // Fill each CHR bank with identifiable pattern
        for bank in 0..4 {
            let start = bank * CHR_BANK_SIZE;
            for i in 0..CHR_BANK_SIZE {
                cartridge.chr_rom[start + i] = (bank as u8).wrapping_mul(0x10);
            }
        }

        let mut mapper = Mapper3::new(cartridge);

        // Initially, bank 0 should be at $0000-$1FFF
        assert_eq!(mapper.ppu_read(0x0000), 0x00);

        // Switch to bank 1
        mapper.cpu_write(0x8000, 1);
        assert_eq!(mapper.ppu_read(0x0000), 0x10);

        // Switch to bank 2
        mapper.cpu_write(0xFFFF, 2); // Any address in $8000-$FFFF works
        assert_eq!(mapper.ppu_read(0x0000), 0x20);

        // Switch to bank 3
        mapper.cpu_write(0xC000, 3);
        assert_eq!(mapper.ppu_read(0x0000), 0x30);
    }

    #[test]
    fn test_chr_bank_wrapping() {
        let cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);
        let mut mapper = Mapper3::new(cartridge);

        // Write bank number larger than available banks
        mapper.cpu_write(0x8000, 255);

        // Bank register stores the raw value
        assert_eq!(mapper.chr_bank, 255);

        // Reading should still work (wrapping handled in map_chr_address)
        let _ = mapper.ppu_read(0x0000); // Should not panic
    }

    #[test]
    fn test_prg_rom_immutable() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        cartridge.prg_rom[0] = 0x42;

        let mapper = Mapper3::new(cartridge);

        // PRG-ROM should be read-only
        let value1 = mapper.cpu_read(0x8000);
        let value2 = mapper.cpu_read(0x8000);
        assert_eq!(value1, value2);
        assert_eq!(value1, 0x42);
    }

    #[test]
    fn test_chr_rom_read_only() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        cartridge.chr_rom[0] = 0xAA;
        cartridge.chr_rom[0x1FFF] = 0xBB;

        let mut mapper = Mapper3::new(cartridge);

        // Try to write to CHR-ROM
        mapper.ppu_write(0x0000, 0xFF);
        mapper.ppu_write(0x1FFF, 0xFF);

        // Values should be unchanged
        assert_eq!(mapper.ppu_read(0x0000), 0xAA);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xBB);
    }

    #[test]
    fn test_mirroring_modes() {
        // Test horizontal mirroring
        let cartridge_h = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        let mapper_h = Mapper3::new(cartridge_h);
        assert_eq!(mapper_h.mirroring(), Mirroring::Horizontal);

        // Test vertical mirroring
        let cartridge_v = create_test_cartridge(16 * 1024, 2, Mirroring::Vertical);
        let mapper_v = Mapper3::new(cartridge_v);
        assert_eq!(mapper_v.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn test_bank_switching_all_addresses() {
        let mut cartridge = create_test_cartridge(16 * 1024, 8, Mirroring::Horizontal);

        // Fill banks with distinct values
        for bank in 0..8 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper3::new(cartridge);

        // Test that writes to any address in $8000-$FFFF work
        let test_addresses = [
            0x8000, 0x9000, 0xA000, 0xB000, 0xC000, 0xD000, 0xE000, 0xF000, 0xFFFF,
        ];

        for (i, &addr) in test_addresses.iter().enumerate() {
            let bank = i % 8;
            mapper.cpu_write(addr, bank as u8);
            assert_eq!(
                mapper.ppu_read(0x0000),
                bank as u8,
                "Bank switching failed at address {:#X}",
                addr
            );
        }
    }

    #[test]
    fn test_chr_bank_persistence() {
        let mut cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);

        // Fill banks with patterns
        for bank in 0..4 {
            let start = bank * CHR_BANK_SIZE;
            for i in 0..256 {
                cartridge.chr_rom[start + i] = ((bank * 0x11) + i) as u8;
            }
        }

        let mut mapper = Mapper3::new(cartridge);

        // Switch to bank 2
        mapper.cpu_write(0x8000, 2);

        // Read multiple locations in the bank
        for i in 0..256 {
            assert_eq!(
                mapper.ppu_read(i as u16),
                ((2 * 0x11) + i) as u8,
                "Failed at offset {}",
                i
            );
        }

        // Switch to bank 1
        mapper.cpu_write(0x8000, 1);

        // Verify bank switched correctly
        for i in 0..256 {
            assert_eq!(
                mapper.ppu_read(i as u16),
                ((1 * 0x11) + i) as u8,
                "Failed at offset {} after switching",
                i
            );
        }
    }

    #[test]
    fn test_maximum_chr_banks() {
        // Test with maximum number of banks (256 banks = 2048KB)
        let cartridge = create_test_cartridge(16 * 1024, 256, Mirroring::Horizontal);
        let mut mapper = Mapper3::new(cartridge);

        assert_eq!(mapper.chr_banks, 256);

        // Switch to various banks
        mapper.cpu_write(0x8000, 0);
        let _ = mapper.ppu_read(0x0000);

        mapper.cpu_write(0x8000, 128);
        let _ = mapper.ppu_read(0x0000);

        mapper.cpu_write(0x8000, 255);
        let _ = mapper.ppu_read(0x0000);
    }

    #[test]
    fn test_no_prg_ram() {
        let cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        let mut mapper = Mapper3::new(cartridge);

        // CNROM doesn't have PRG-RAM
        assert!(mapper.prg_ram().is_none());
        assert!(mapper.prg_ram_mut().is_none());
    }

    #[test]
    fn test_power_on_state() {
        let cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);
        let mapper = Mapper3::new(cartridge);

        // Initial bank should be 0
        assert_eq!(mapper.chr_bank, 0);
        assert_eq!(mapper.chr_banks, 4);

        // Mirroring should match cartridge
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn test_bank_boundary_reads() {
        let mut cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);

        // Set specific values at bank boundaries
        cartridge.chr_rom[0x1FFF] = 0xAA; // Last byte of bank 0
        cartridge.chr_rom[0x2000] = 0xBB; // First byte of bank 1
        cartridge.chr_rom[0x3FFF] = 0xCC; // Last byte of bank 1

        let mut mapper = Mapper3::new(cartridge);

        // Test bank 0 boundaries
        mapper.cpu_write(0x8000, 0);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xAA);

        // Test bank 1 boundaries
        mapper.cpu_write(0x8000, 1);
        assert_eq!(mapper.ppu_read(0x0000), 0xBB);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xCC);
    }

    #[test]
    fn test_consecutive_same_bank_writes() {
        let mut cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);
        cartridge.chr_rom[0] = 0xAA;

        let mut mapper = Mapper3::new(cartridge);

        // Write same bank multiple times
        for _ in 0..10 {
            mapper.cpu_write(0x8000, 0);
            assert_eq!(mapper.chr_bank, 0);
            assert_eq!(mapper.ppu_read(0x0000), 0xAA);
        }
    }

    #[test]
    fn test_rapid_bank_switching() {
        let mut cartridge = create_test_cartridge(16 * 1024, 8, Mirroring::Horizontal);

        for bank in 0..8 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper3::new(cartridge);

        // Rapidly switch banks and verify each time
        for _ in 0..100 {
            for bank in 0..8 {
                mapper.cpu_write(0x8000, bank as u8);
                assert_eq!(mapper.ppu_read(0x0000), bank as u8);
            }
        }
    }

    #[test]
    fn test_full_chr_range_access() {
        let mut cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);

        // Fill CHR with pattern
        for i in 0..cartridge.chr_rom.len() {
            cartridge.chr_rom[i] = (i & 0xFF) as u8;
        }

        let mut mapper = Mapper3::new(cartridge);

        // Test bank 0
        mapper.cpu_write(0x8000, 0);
        assert_eq!(mapper.ppu_read(0x0000), 0x00);
        assert_eq!(mapper.ppu_read(0x1000), 0x00); // (0x1000 & 0xFF)
        assert_eq!(mapper.ppu_read(0x1FFF), 0xFF); // (0x1FFF & 0xFF)

        // Test bank 1
        mapper.cpu_write(0x8000, 1);
        assert_eq!(mapper.ppu_read(0x0000), 0x00); // (0x2000 & 0xFF)
        assert_eq!(mapper.ppu_read(0x1FFF), 0xFF); // (0x3FFF & 0xFF)
    }

    #[test]
    fn test_unmapped_address_reads() {
        let cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        let mapper = Mapper3::new(cartridge);

        // CPU addresses below $8000 should return 0
        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);

        // PPU addresses above $1FFF should return 0
        assert_eq!(mapper.ppu_read(0x2000), 0);
        assert_eq!(mapper.ppu_read(0x3FFF), 0);
    }

    #[test]
    fn test_unmapped_address_writes() {
        let cartridge = create_test_cartridge(16 * 1024, 2, Mirroring::Horizontal);
        let mut mapper = Mapper3::new(cartridge);

        // Writes to unmapped CPU addresses should be ignored
        mapper.cpu_write(0x0000, 0xFF);
        mapper.cpu_write(0x6000, 0xFF);
        mapper.cpu_write(0x7FFF, 0xFF);

        // Verify no crash and state is consistent
        assert_eq!(mapper.chr_bank, 0);
    }

    #[test]
    fn test_all_mirroring_modes() {
        let test_cases = [
            Mirroring::Horizontal,
            Mirroring::Vertical,
            Mirroring::SingleScreen,
            Mirroring::FourScreen,
        ];

        for mirroring in test_cases {
            let cartridge = create_test_cartridge(16 * 1024, 2, mirroring);
            let mapper = Mapper3::new(cartridge);
            assert_eq!(mapper.mirroring(), mirroring);
        }
    }

    #[test]
    fn test_mirroring_persistence_after_bank_switch() {
        let cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Vertical);
        let mut mapper = Mapper3::new(cartridge);

        let original_mirroring = mapper.mirroring();

        // Mirroring should not change after bank switches
        for bank in 0..4 {
            mapper.cpu_write(0x8000, bank);
            assert_eq!(mapper.mirroring(), original_mirroring);
        }
    }

    #[test]
    fn test_bank_modulo_wrapping() {
        let mut cartridge = create_test_cartridge(16 * 1024, 4, Mirroring::Horizontal);

        for bank in 0..4 {
            let start = bank * CHR_BANK_SIZE;
            cartridge.chr_rom[start] = (bank * 0x10) as u8;
        }

        let mut mapper = Mapper3::new(cartridge);

        // Test wrapping: bank 4 should map to bank 0, bank 5 to bank 1, etc.
        mapper.cpu_write(0x8000, 4);
        assert_eq!(mapper.ppu_read(0x0000), 0x00); // Wraps to bank 0

        mapper.cpu_write(0x8000, 5);
        assert_eq!(mapper.ppu_read(0x0000), 0x10); // Wraps to bank 1

        mapper.cpu_write(0x8000, 8);
        assert_eq!(mapper.ppu_read(0x0000), 0x00); // Wraps to bank 0

        mapper.cpu_write(0x8000, 11);
        assert_eq!(mapper.ppu_read(0x0000), 0x30); // Wraps to bank 3
    }

    #[test]
    fn test_minimum_chr_banks() {
        // Test with minimum size (1 bank = 8KB)
        let cartridge = create_test_cartridge(16 * 1024, 1, Mirroring::Horizontal);
        let mut mapper = Mapper3::new(cartridge);

        assert_eq!(mapper.chr_banks, 1);

        // Only bank 0 available
        mapper.cpu_write(0x8000, 0);
        let _ = mapper.ppu_read(0x0000);

        // Should wrap to bank 0
        mapper.cpu_write(0x8000, 1);
        let _ = mapper.ppu_read(0x0000);
    }

    #[test]
    fn test_prg_rom_not_affected_by_chr_switch() {
        let mut cartridge = create_test_cartridge(32 * 1024, 4, Mirroring::Horizontal);

        // Set specific PRG values
        cartridge.prg_rom[0] = 0x11;
        cartridge.prg_rom[0x4000] = 0x22;

        let mut mapper = Mapper3::new(cartridge);

        // Read PRG initially
        let prg_val1 = mapper.cpu_read(0x8000);
        let prg_val2 = mapper.cpu_read(0xC000);

        // Switch CHR banks
        for bank in 0..4 {
            mapper.cpu_write(0x8000, bank);

            // PRG should remain unchanged
            assert_eq!(mapper.cpu_read(0x8000), prg_val1);
            assert_eq!(mapper.cpu_read(0xC000), prg_val2);
        }
    }
}
