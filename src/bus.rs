// Bus module - Memory bus implementation
//
// This module implements the main memory bus that connects the CPU to all memory-mapped
// components in the NES system. It handles address routing, memory mirroring, and
// provides interfaces for dynamic component registration.
//
// # NES Memory Map (CPU Address Space)
//
// ```text
// $0000-$07FF: 2KB Internal RAM
// $0800-$1FFF: Mirrors of RAM (3 times)
// $2000-$2007: PPU Registers
// $2008-$3FFF: Mirrors of PPU Registers (repeating every 8 bytes)
// $4000-$4017: APU and I/O Registers
// $4018-$401F: APU and I/O test functionality (usually disabled)
// $4020-$FFFF: Cartridge space (PRG-ROM, PRG-RAM, and mapper registers)
// ```

/// Trait for memory-mapped components
///
/// This trait defines the interface for components that can be mapped into the
/// CPU's address space. Components implementing this trait can handle read and
/// write operations for their specific memory ranges.
///
/// # Examples
///
/// PPU, APU, and Cartridge all implement this trait to provide their memory-mapped
/// register interfaces.
pub trait MemoryMappedDevice {
    /// Read a byte from the device
    ///
    /// # Arguments
    /// * `addr` - The address to read from (device-specific addressing)
    ///
    /// # Returns
    /// The byte value at the specified address
    fn read(&self, addr: u16) -> u8;

    /// Write a byte to the device
    ///
    /// # Arguments
    /// * `addr` - The address to write to (device-specific addressing)
    /// * `data` - The byte value to write
    fn write(&mut self, addr: u16, data: u8);
}

/// Main memory bus structure
///
/// The Bus connects the CPU to all memory-mapped components in the NES system.
/// It handles address decoding, memory mirroring, and routes read/write operations
/// to the appropriate components.
///
/// # Memory Layout
///
/// - Internal RAM: 2KB of general-purpose memory with 3 mirrors
/// - PPU Registers: 8 registers mirrored throughout $2000-$3FFF
/// - APU/I/O: Audio and input/output registers
/// - Cartridge Space: Game ROM and mapper-controlled memory
pub struct Bus {
    /// Internal RAM: 2KB
    ///
    /// The NES has 2KB of internal RAM located at $0000-$07FF.
    /// This RAM is mirrored 3 times at $0800-$1FFF.
    ram: [u8; 2048],

    /// Temporary ROM storage for testing
    ///
    /// This will be replaced with proper cartridge/mapper implementation.
    /// Covers $4020-$FFFF (approximately 48KB).
    rom: [u8; 0xC000],

    // Future: Dynamic component registration
    // ppu: Option<Box<dyn MemoryMappedDevice>>,
    // apu: Option<Box<dyn MemoryMappedDevice>>,
    // cartridge: Option<Box<dyn MemoryMappedDevice>>,
}

impl Bus {
    /// Create a new bus instance with zero-initialized memory
    ///
    /// # Returns
    /// A new Bus with all memory initialized to zero
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let bus = Bus::new();
    /// ```
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            rom: [0; 0xC000],
        }
    }

    /// Read a byte from the bus
    ///
    /// Routes the read operation to the appropriate memory region or device
    /// based on the address. Handles mirroring for RAM and PPU registers.
    ///
    /// # Arguments
    /// * `addr` - The 16-bit address to read from
    ///
    /// # Returns
    /// The byte value at the specified address
    ///
    /// # Memory Regions
    ///
    /// - $0000-$1FFF: Internal RAM (2KB) with mirroring
    /// - $2000-$3FFF: PPU registers (8 bytes) with mirroring
    /// - $4000-$4017: APU and I/O registers
    /// - $4018-$401F: APU/I/O test mode (usually returns open bus)
    /// - $4020-$FFFF: Cartridge space
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let bus = Bus::new();
    /// let value = bus.read(0x0000); // Read from RAM
    /// ```
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // Internal RAM: $0000-$07FF (2KB)
            // Mirrored at: $0800-$0FFF, $1000-$17FF, $1800-$1FFF
            // Total range: $0000-$1FFF
            0x0000..=0x1FFF => {
                // Mirror using mask: only keep lowest 11 bits (0x07FF = 2KB)
                let ram_addr = (addr & 0x07FF) as usize;
                self.ram[ram_addr]
            }

            // PPU Registers: $2000-$2007 (8 bytes)
            // Mirrored throughout: $2008-$3FFF (repeating every 8 bytes)
            // Total range: $2000-$3FFF
            0x2000..=0x3FFF => {
                // Mirror using mask: only keep lowest 3 bits for register selection
                let _ppu_reg = addr & 0x0007;
                // TODO: Route to PPU component when registered
                0
            }

            // APU and I/O Registers: $4000-$4017
            0x4000..=0x4017 => {
                // TODO: Route to APU/Controller components when registered
                0
            }

            // APU/I/O Test Mode: $4018-$401F
            // Usually disabled on retail NES hardware
            0x4018..=0x401F => {
                // Return open bus value (typically last value on bus)
                0
            }

            // Cartridge Space: $4020-$FFFF
            // This includes PRG-ROM, PRG-RAM, and mapper registers
            0x4020..=0xFFFF => {
                // TODO: Route to cartridge/mapper when registered
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0
                }
            }
        }
    }

    /// Write a byte to the bus
    ///
    /// Routes the write operation to the appropriate memory region or device
    /// based on the address. Handles mirroring for RAM and PPU registers.
    ///
    /// # Arguments
    /// * `addr` - The 16-bit address to write to
    /// * `data` - The byte value to write
    ///
    /// # Memory Regions
    ///
    /// - $0000-$1FFF: Internal RAM (2KB) with mirroring
    /// - $2000-$3FFF: PPU registers (8 bytes) with mirroring
    /// - $4000-$4017: APU and I/O registers
    /// - $4018-$401F: APU/I/O test mode (usually ignored)
    /// - $4020-$FFFF: Cartridge space (writes may affect mapper state)
    ///
    /// # Note on ROM writes
    /// Writes to ROM addresses ($8000-$FFFF) don't modify ROM data but can
    /// trigger mapper functionality (bank switching, etc.)
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write(0x0000, 0x42); // Write to RAM
    /// ```
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Internal RAM: $0000-$07FF (2KB)
            // Mirrored at: $0800-$0FFF, $1000-$17FF, $1800-$1FFF
            // Total range: $0000-$1FFF
            0x0000..=0x1FFF => {
                // Mirror using mask: only keep lowest 11 bits (0x07FF = 2KB)
                let ram_addr = (addr & 0x07FF) as usize;
                self.ram[ram_addr] = data;
            }

            // PPU Registers: $2000-$2007 (8 bytes)
            // Mirrored throughout: $2008-$3FFF (repeating every 8 bytes)
            // Total range: $2000-$3FFF
            0x2000..=0x3FFF => {
                // Mirror using mask: only keep lowest 3 bits for register selection
                let _ppu_reg = addr & 0x0007;
                // TODO: Route to PPU component when registered
            }

            // APU and I/O Registers: $4000-$4017
            0x4000..=0x4017 => {
                // TODO: Route to APU/Controller components when registered
            }

            // APU/I/O Test Mode: $4018-$401F
            // Usually disabled on retail NES hardware
            0x4018..=0x401F => {
                // Ignore writes to this region
            }

            // Cartridge Space: $4020-$FFFF
            // Writes here may trigger mapper functionality (e.g., bank switching)
            0x4020..=0xFFFF => {
                // TODO: Route to cartridge/mapper when registered
                // For now, allow writes to our temporary ROM array for testing
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr] = data;
                }
            }
        }
    }

    /// Load ROM data into cartridge space
    ///
    /// This is a temporary helper method for testing. It will be replaced
    /// with proper cartridge/mapper loading in the future.
    ///
    /// # Arguments
    /// * `data` - Slice of bytes to load into ROM
    /// * `offset` - Starting address offset (relative to $4020)
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// let rom_data = vec![0x4C, 0x00, 0x80]; // JMP $8000
    /// bus.load_rom(&rom_data, 0x3FE0); // Load at $8000
    /// ```
    pub fn load_rom(&mut self, data: &[u8], offset: usize) {
        let end = (offset + data.len()).min(self.rom.len());
        self.rom[offset..end].copy_from_slice(&data[..(end - offset)]);
    }

    /// Read a 16-bit word from the bus (little-endian)
    ///
    /// Reads two consecutive bytes and combines them into a 16-bit value.
    /// The first byte is the low byte, the second is the high byte (little-endian).
    ///
    /// # Arguments
    /// * `addr` - The address of the low byte
    ///
    /// # Returns
    /// A 16-bit value constructed from two consecutive bytes
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write(0x0000, 0x34);
    /// bus.write(0x0001, 0x12);
    /// assert_eq!(bus.read_u16(0x0000), 0x1234);
    /// ```
    pub fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a 16-bit word to the bus (little-endian)
    ///
    /// Writes a 16-bit value as two consecutive bytes.
    /// The low byte is written first, then the high byte (little-endian).
    ///
    /// # Arguments
    /// * `addr` - The address to write the low byte to
    /// * `data` - The 16-bit value to write
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write_u16(0x0000, 0x1234);
    /// assert_eq!(bus.read(0x0000), 0x34);
    /// assert_eq!(bus.read(0x0001), 0x12);
    /// ```
    pub fn write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Bus Initialization Tests
    // ========================================

    #[test]
    fn test_bus_initialization() {
        let bus = Bus::new();
        // Verify RAM is zero-initialized
        assert_eq!(bus.read(0x0000), 0, "RAM start should be zero");
        assert_eq!(bus.read(0x07FF), 0, "RAM end should be zero");
        assert_eq!(bus.read(0x0400), 0, "RAM middle should be zero");
    }

    #[test]
    fn test_bus_default() {
        let bus1 = Bus::new();
        let bus2 = Bus::default();
        // Both should have same initial state
        assert_eq!(bus1.read(0x0000), bus2.read(0x0000));
    }

    // ========================================
    // RAM Tests ($ 0000-$1FFF)
    // ========================================

    #[test]
    fn test_ram_read_write() {
        let mut bus = Bus::new();
        bus.write(0x0000, 0x42);
        assert_eq!(bus.read(0x0000), 0x42);
    }

    #[test]
    fn test_ram_multiple_writes() {
        let mut bus = Bus::new();
        // Write to different RAM addresses
        bus.write(0x0000, 0x11);
        bus.write(0x0100, 0x22);
        bus.write(0x0200, 0x33);
        bus.write(0x07FF, 0x44);

        assert_eq!(bus.read(0x0000), 0x11);
        assert_eq!(bus.read(0x0100), 0x22);
        assert_eq!(bus.read(0x0200), 0x33);
        assert_eq!(bus.read(0x07FF), 0x44);
    }

    #[test]
    fn test_ram_mirroring_first_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0000
        bus.write(0x0000, 0x42);
        // Read from first mirror at $0800
        assert_eq!(bus.read(0x0800), 0x42, "First mirror should reflect base RAM");
    }

    #[test]
    fn test_ram_mirroring_second_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0100
        bus.write(0x0100, 0x55);
        // Read from second mirror at $1100
        assert_eq!(
            bus.read(0x1100),
            0x55,
            "Second mirror should reflect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_third_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0200
        bus.write(0x0200, 0x88);
        // Read from third mirror at $1A00
        assert_eq!(
            bus.read(0x1A00),
            0x88,
            "Third mirror should reflect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_bidirectional() {
        let mut bus = Bus::new();
        // Write to mirror
        bus.write(0x0800, 0x99);
        // Read from base RAM
        assert_eq!(bus.read(0x0000), 0x99, "Mirror write should affect base RAM");

        // Write to another mirror
        bus.write(0x1500, 0xAA);
        // Read from base RAM
        assert_eq!(
            bus.read(0x0500),
            0xAA,
            "Mirror write should affect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_all_regions() {
        let mut bus = Bus::new();
        // Test that all four regions are properly mirrored
        let test_addr = 0x0123; // Arbitrary address within 2KB

        bus.write(test_addr, 0xAB);

        assert_eq!(bus.read(test_addr), 0xAB, "Base RAM");
        assert_eq!(bus.read(test_addr + 0x0800), 0xAB, "First mirror");
        assert_eq!(bus.read(test_addr + 0x1000), 0xAB, "Second mirror");
        assert_eq!(bus.read(test_addr + 0x1800), 0xAB, "Third mirror");
    }

    #[test]
    fn test_ram_boundary_addresses() {
        let mut bus = Bus::new();
        // Test boundary addresses of RAM
        bus.write(0x0000, 0x11); // Start of RAM
        bus.write(0x07FF, 0x22); // End of RAM
        bus.write(0x0800, 0x33); // Start of first mirror
        bus.write(0x1FFF, 0x44); // End of third mirror

        assert_eq!(bus.read(0x0000), 0x33, "Start should mirror to $0800");
        assert_eq!(bus.read(0x07FF), 0x44, "End should mirror to $1FFF");
    }

    // ========================================
    // PPU Register Tests ($2000-$3FFF)
    // ========================================

    #[test]
    fn test_ppu_register_range() {
        let bus = Bus::new();
        // PPU registers should return 0 (stub implementation)
        assert_eq!(bus.read(0x2000), 0, "PPUCTRL");
        assert_eq!(bus.read(0x2001), 0, "PPUMASK");
        assert_eq!(bus.read(0x2002), 0, "PPUSTATUS");
        assert_eq!(bus.read(0x2007), 0, "PPUDATA");
    }

    #[test]
    fn test_ppu_register_mirroring() {
        let bus = Bus::new();
        // PPU registers repeat every 8 bytes
        assert_eq!(bus.read(0x2000), bus.read(0x2008), "$2000 mirrors at $2008");
        assert_eq!(bus.read(0x2000), bus.read(0x2010), "$2000 mirrors at $2010");
        assert_eq!(bus.read(0x2007), bus.read(0x200F), "$2007 mirrors at $200F");
        assert_eq!(
            bus.read(0x2000),
            bus.read(0x3FF8),
            "$2000 mirrors at $3FF8"
        );
    }

    #[test]
    fn test_ppu_register_write_does_not_crash() {
        let mut bus = Bus::new();
        // Writes to PPU registers should not crash (even if stubbed)
        bus.write(0x2000, 0x80);
        bus.write(0x2001, 0x1E);
        bus.write(0x2006, 0x20);
        bus.write(0x2007, 0x00);
    }

    #[test]
    fn test_ppu_mirror_write() {
        let mut bus = Bus::new();
        // Writes to mirrored PPU addresses should not crash
        bus.write(0x2008, 0x80); // Mirror of $2000
        bus.write(0x3000, 0x00); // Deep in mirror region
        bus.write(0x3FFF, 0xFF); // Last address in PPU region
    }

    // ========================================
    // APU and I/O Tests ($4000-$401F)
    // ========================================

    #[test]
    fn test_apu_registers() {
        let bus = Bus::new();
        // APU registers should return 0 (stub implementation)
        assert_eq!(bus.read(0x4000), 0, "SQ1_VOL");
        assert_eq!(bus.read(0x4015), 0, "SND_CHN");
    }

    #[test]
    fn test_apu_write_does_not_crash() {
        let mut bus = Bus::new();
        // Writes to APU registers should not crash
        bus.write(0x4000, 0x30);
        bus.write(0x4015, 0x0F);
    }

    #[test]
    fn test_io_test_region() {
        let bus = Bus::new();
        // Test region $4018-$401F should return 0
        assert_eq!(bus.read(0x4018), 0);
        assert_eq!(bus.read(0x401F), 0);
    }

    #[test]
    fn test_io_test_region_write_ignored() {
        let mut bus = Bus::new();
        // Writes to test region should be ignored
        bus.write(0x4018, 0xFF);
        bus.write(0x401F, 0xFF);
        // Should still read as 0
        assert_eq!(bus.read(0x4018), 0);
        assert_eq!(bus.read(0x401F), 0);
    }

    // ========================================
    // Cartridge Space Tests ($4020-$FFFF)
    // ========================================

    #[test]
    fn test_rom_read_write() {
        let mut bus = Bus::new();
        // Write to ROM space (for testing)
        bus.write(0x8000, 0x4C);
        assert_eq!(bus.read(0x8000), 0x4C);
    }

    #[test]
    fn test_load_rom() {
        let mut bus = Bus::new();
        let rom_data = vec![0x4C, 0x00, 0x80]; // JMP $8000

        // Load at offset 0x3FE0 (which maps to $8000)
        bus.load_rom(&rom_data, 0x3FE0);

        assert_eq!(bus.read(0x8000), 0x4C);
        assert_eq!(bus.read(0x8001), 0x00);
        assert_eq!(bus.read(0x8002), 0x80);
    }

    #[test]
    fn test_rom_boundary() {
        let mut bus = Bus::new();
        bus.write(0x4020, 0x11); // First address in cartridge space
        bus.write(0xFFFF, 0x22); // Last address in memory map

        assert_eq!(bus.read(0x4020), 0x11);
        assert_eq!(bus.read(0xFFFF), 0x22);
    }

    // ========================================
    // 16-bit Read/Write Tests
    // ========================================

    #[test]
    fn test_read_u16() {
        let mut bus = Bus::new();
        bus.write(0x0000, 0x34); // Low byte
        bus.write(0x0001, 0x12); // High byte

        let value = bus.read_u16(0x0000);
        assert_eq!(value, 0x1234, "Should read little-endian 16-bit value");
    }

    #[test]
    fn test_write_u16() {
        let mut bus = Bus::new();
        bus.write_u16(0x0000, 0x1234);

        assert_eq!(bus.read(0x0000), 0x34, "Low byte should be first");
        assert_eq!(bus.read(0x0001), 0x12, "High byte should be second");
    }

    #[test]
    fn test_u16_roundtrip() {
        let mut bus = Bus::new();
        let test_value = 0xABCD;

        bus.write_u16(0x0100, test_value);
        let read_value = bus.read_u16(0x0100);

        assert_eq!(read_value, test_value, "16-bit roundtrip should preserve value");
    }

    #[test]
    fn test_u16_across_pages() {
        let mut bus = Bus::new();
        // Write 16-bit value that crosses page boundary
        bus.write_u16(0x00FF, 0x5678);

        assert_eq!(bus.read(0x00FF), 0x78);
        assert_eq!(bus.read(0x0100), 0x56);
        assert_eq!(bus.read_u16(0x00FF), 0x5678);
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_stack_operations() {
        let mut bus = Bus::new();
        // Stack is at $0100-$01FF

        // Push values (typical stack usage)
        bus.write(0x01FD, 0x11);
        bus.write(0x01FC, 0x22);
        bus.write(0x01FB, 0x33);

        assert_eq!(bus.read(0x01FD), 0x11);
        assert_eq!(bus.read(0x01FC), 0x22);
        assert_eq!(bus.read(0x01FB), 0x33);
    }

    #[test]
    fn test_zero_page_operations() {
        let mut bus = Bus::new();
        // Zero page is $0000-$00FF
        bus.write(0x0000, 0xAA);
        bus.write(0x00FF, 0xBB);

        assert_eq!(bus.read(0x0000), 0xAA);
        assert_eq!(bus.read(0x00FF), 0xBB);
    }

    #[test]
    fn test_interrupt_vectors() {
        let mut bus = Bus::new();
        // Set interrupt vectors
        bus.write_u16(0xFFFA, 0x9000); // NMI vector
        bus.write_u16(0xFFFC, 0x8000); // RESET vector
        bus.write_u16(0xFFFE, 0xA000); // IRQ/BRK vector

        assert_eq!(bus.read_u16(0xFFFA), 0x9000);
        assert_eq!(bus.read_u16(0xFFFC), 0x8000);
        assert_eq!(bus.read_u16(0xFFFE), 0xA000);
    }

    #[test]
    fn test_memory_independence() {
        let mut bus = Bus::new();
        // Write to different memory regions
        bus.write(0x0000, 0x11); // RAM
        bus.write(0x2000, 0x22); // PPU
        bus.write(0x4000, 0x33); // APU
        bus.write(0x8000, 0x44); // ROM

        // Verify they don't interfere with each other
        assert_eq!(bus.read(0x0000), 0x11);
        assert_eq!(bus.read(0x8000), 0x44);
    }

    #[test]
    fn test_comprehensive_memory_map() {
        let mut bus = Bus::new();

        // Test each major memory region
        bus.write(0x0010, 0x01); // RAM
        bus.write(0x0810, 0x02); // RAM mirror
        bus.write(0x2003, 0x03); // PPU
        bus.write(0x300B, 0x04); // PPU mirror
        bus.write(0x4005, 0x05); // APU
        bus.write(0x8050, 0x06); // ROM

        assert_eq!(bus.read(0x0010), 0x02, "RAM mirror should overwrite");
        assert_eq!(bus.read(0x0810), 0x02, "RAM mirror bidirectional");
        assert_eq!(bus.read(0x8050), 0x06, "ROM independent");
    }
}
