// Mapper 0 (NROM) - The simplest NES mapper with no bank switching
//
// Memory Layout:
// - CPU $8000-$BFFF: First 16KB of PRG-ROM
// - CPU $C000-$FFFF: Last 16KB of PRG-ROM (or mirror of first 16KB if only 16KB total)
// - PPU $0000-$1FFF: 8KB CHR-ROM or CHR-RAM
//
// Variants:
// - NROM-128: 16KB PRG-ROM (mirrored to fill 32KB space)
// - NROM-256: 32KB PRG-ROM (no mirroring)
//
// CHR Configuration:
// - CHR-ROM: 8KB read-only pattern memory
// - CHR-RAM: 8KB writable pattern memory

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// Mapper 0 implementation (NROM)
///
/// This is the simplest mapper used by games like Super Mario Bros., Donkey Kong,
/// and Balloon Fight. It has no bank switching capability.
pub struct Mapper0 {
    /// PRG-ROM data (16KB or 32KB)
    prg_rom: Vec<u8>,
    /// CHR-ROM or CHR-RAM data (8KB)
    chr_mem: Vec<u8>,
    /// Whether CHR memory is RAM (writable) or ROM (read-only)
    chr_is_ram: bool,
    /// Mirroring type (fixed, cannot be changed by the mapper)
    mirroring: Mirroring,
}

impl Mapper0 {
    /// Create a new Mapper0 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    ///
    /// # Panics
    /// Panics if PRG-ROM size is not 16KB or 32KB (should be validated before mapper creation)
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();

        // Validate PRG-ROM size (must be 16KB or 32KB)
        assert!(
            prg_rom_size == 16 * 1024 || prg_rom_size == 32 * 1024,
            "Mapper 0 requires 16KB or 32KB PRG-ROM, got {} bytes",
            prg_rom_size
        );

        // CHR-RAM is indicated by chr_rom_banks = 0 in the iNES header
        // The cartridge loader already allocated 8KB for CHR-RAM in this case
        let chr_is_ram =
            cartridge.chr_rom.len() == 8 * 1024 && cartridge.chr_rom.iter().all(|&b| b == 0);

        Mapper0 {
            prg_rom: cartridge.prg_rom,
            chr_mem: cartridge.chr_rom,
            chr_is_ram,
            mirroring: cartridge.mirroring,
        }
    }
}

impl Mapper for Mapper0 {
    /// Read from CPU address space
    ///
    /// For NROM:
    /// - $8000-$BFFF: First 16KB of PRG-ROM
    /// - $C000-$FFFF: Last 16KB of PRG-ROM (or mirror of first 16KB for 16KB ROMs)
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => {
                // Map address to PRG-ROM index
                let index = (address - 0x8000) as usize;

                // Use modulo to handle mirroring for 16KB ROMs
                // For 32KB ROMs, modulo has no effect since index < prg_rom.len()
                self.prg_rom[index % self.prg_rom.len()]
            }
            _ => {
                // Unmapped address (shouldn't happen if bus is implemented correctly)
                0
            }
        }
    }

    /// Write to CPU address space
    ///
    /// NROM has no writable registers, so all writes are ignored
    fn cpu_write(&mut self, _address: u16, _value: u8) {
        // NROM has no bank switching or other mapper registers
        // Writes to PRG-ROM space are ignored
    }

    /// Read from PPU address space
    ///
    /// For NROM:
    /// - $0000-$1FFF: 8KB CHR-ROM or CHR-RAM
    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = address as usize;
                self.chr_mem[index]
            }
            _ => {
                // Unmapped address (shouldn't happen if bus is implemented correctly)
                0
            }
        }
    }

    /// Write to PPU address space
    ///
    /// For NROM:
    /// - CHR-RAM: writes are allowed
    /// - CHR-ROM: writes are ignored
    fn ppu_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                if self.chr_is_ram {
                    let index = address as usize;
                    self.chr_mem[index] = value;
                }
                // For CHR-ROM, writes are silently ignored
            }
            _ => {
                // Unmapped address (shouldn't happen if bus is implemented correctly)
            }
        }
    }

    /// Get the mirroring mode
    ///
    /// For NROM, mirroring is fixed and determined by the cartridge header
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge with specified configuration
    fn create_test_cartridge(
        prg_rom_size: usize,
        chr_rom_size: usize,
        mirroring: Mirroring,
        fill_chr: bool,
    ) -> Cartridge {
        let prg_rom = vec![0xAA; prg_rom_size];
        let chr_rom = if fill_chr {
            vec![0xBB; chr_rom_size]
        } else {
            vec![0x00; chr_rom_size]
        };

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 0,
            mirroring,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper0_creation_16kb() {
        let cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);
        let mapper = Mapper0::new(cartridge);

        assert_eq!(mapper.prg_rom.len(), 16 * 1024);
        assert_eq!(mapper.chr_mem.len(), 8 * 1024);
        assert_eq!(mapper.mirroring, Mirroring::Horizontal);
    }

    #[test]
    fn test_mapper0_creation_32kb() {
        let cartridge = create_test_cartridge(32 * 1024, 8 * 1024, Mirroring::Vertical, true);
        let mapper = Mapper0::new(cartridge);

        assert_eq!(mapper.prg_rom.len(), 32 * 1024);
        assert_eq!(mapper.chr_mem.len(), 8 * 1024);
        assert_eq!(mapper.mirroring, Mirroring::Vertical);
    }

    #[test]
    #[should_panic(expected = "Mapper 0 requires 16KB or 32KB PRG-ROM")]
    fn test_mapper0_invalid_prg_size() {
        let cartridge = create_test_cartridge(8 * 1024, 8 * 1024, Mirroring::Horizontal, true);
        Mapper0::new(cartridge);
    }

    #[test]
    fn test_cpu_read_16kb_mirroring() {
        let mut cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);

        // Fill PRG-ROM with identifiable pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper0::new(cartridge);

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
    fn test_cpu_read_32kb_no_mirroring() {
        let mut cartridge = create_test_cartridge(32 * 1024, 8 * 1024, Mirroring::Horizontal, true);

        // Fill PRG-ROM with identifiable pattern
        for i in 0..cartridge.prg_rom.len() {
            cartridge.prg_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper0::new(cartridge);

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
    fn test_cpu_write_ignored() {
        let cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);
        let mut mapper = Mapper0::new(cartridge);

        // Store original value
        let original = mapper.cpu_read(0x8000);

        // Try to write
        mapper.cpu_write(0x8000, 0xFF);

        // Value should be unchanged
        assert_eq!(mapper.cpu_read(0x8000), original);
    }

    #[test]
    fn test_ppu_read_chr_rom() {
        let mut cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);

        // Fill CHR-ROM with identifiable pattern
        for i in 0..cartridge.chr_rom.len() {
            cartridge.chr_rom[i] = (i & 0xFF) as u8;
        }

        let mapper = Mapper0::new(cartridge);

        // Test reading from various CHR addresses
        assert_eq!(mapper.ppu_read(0x0000), 0x00);
        assert_eq!(mapper.ppu_read(0x0001), 0x01);
        assert_eq!(mapper.ppu_read(0x1000), 0x00); // (0x1000 & 0xFF)
        assert_eq!(mapper.ppu_read(0x1FFF), 0xFF); // (0x1FFF & 0xFF)
    }

    #[test]
    fn test_ppu_write_chr_ram() {
        let cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, false);
        let mut mapper = Mapper0::new(cartridge);

        // Verify it's CHR-RAM
        assert!(mapper.chr_is_ram);

        // Write and read back
        mapper.ppu_write(0x0000, 0x42);
        assert_eq!(mapper.ppu_read(0x0000), 0x42);

        mapper.ppu_write(0x1FFF, 0x99);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x99);
    }

    #[test]
    fn test_ppu_write_chr_rom_ignored() {
        let mut cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);

        // Set a specific pattern in CHR-ROM
        cartridge.chr_rom[0] = 0xAA;
        cartridge.chr_rom[0x1FFF] = 0xBB;

        let mut mapper = Mapper0::new(cartridge);

        // Verify it's CHR-ROM (not RAM)
        assert!(!mapper.chr_is_ram);

        // Try to write
        mapper.ppu_write(0x0000, 0xFF);
        mapper.ppu_write(0x1FFF, 0xFF);

        // Values should be unchanged
        assert_eq!(mapper.ppu_read(0x0000), 0xAA);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xBB);
    }

    #[test]
    fn test_mirroring_modes() {
        // Test horizontal mirroring
        let cartridge_h = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);
        let mapper_h = Mapper0::new(cartridge_h);
        assert_eq!(mapper_h.mirroring(), Mirroring::Horizontal);

        // Test vertical mirroring
        let cartridge_v = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Vertical, true);
        let mapper_v = Mapper0::new(cartridge_v);
        assert_eq!(mapper_v.mirroring(), Mirroring::Vertical);

        // Test four-screen VRAM
        let cartridge_fs = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::FourScreen, true);
        let mapper_fs = Mapper0::new(cartridge_fs);
        assert_eq!(mapper_fs.mirroring(), Mirroring::FourScreen);
    }

    #[test]
    fn test_address_boundary_conditions() {
        let cartridge = create_test_cartridge(16 * 1024, 8 * 1024, Mirroring::Horizontal, true);
        let mapper = Mapper0::new(cartridge);

        // Test CPU address boundaries
        let _ = mapper.cpu_read(0x8000); // Start of PRG-ROM
        let _ = mapper.cpu_read(0xFFFF); // End of PRG-ROM

        // Test PPU address boundaries
        let _ = mapper.ppu_read(0x0000); // Start of CHR
        let _ = mapper.ppu_read(0x1FFF); // End of CHR

        // Test unmapped regions return 0
        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
        assert_eq!(mapper.ppu_read(0x2000), 0);
    }
}
