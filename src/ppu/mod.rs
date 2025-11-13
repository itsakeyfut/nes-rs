// PPU module - Picture Processing Unit implementation
// This module contains the PPU (2C02) emulation
//
// # PPU Registers (Phase 2 - Stub Implementation)
//
// The PPU has 8 registers mapped at $2000-$2007 in CPU memory space.
// These registers are mirrored throughout $2008-$3FFF (repeating every 8 bytes).
//
// This is a stub implementation for Phase 2. Full PPU functionality will be
// implemented in Phase 4.
//
// ## Register Map
//
// | Address | Name       | Access | Description                    |
// |---------|------------|--------|--------------------------------|
// | $2000   | PPUCTRL    | Write  | PPU Control Register 1         |
// | $2001   | PPUMASK    | Write  | PPU Control Register 2         |
// | $2002   | PPUSTATUS  | Read   | PPU Status Register            |
// | $2003   | OAMADDR    | Write  | OAM Address Port               |
// | $2004   | OAMDATA    | R/W    | OAM Data Port                  |
// | $2005   | PPUSCROLL  | Write×2| Scroll Position Register       |
// | $2006   | PPUADDR    | Write×2| PPU Address Register           |
// | $2007   | PPUDATA    | R/W    | PPU Data Port                  |

use crate::bus::MemoryMappedDevice;

// Test-only constants for PPU register addresses
#[cfg(test)]
mod test_constants {
    /// PPU Control Register ($2000) - Write only
    pub const PPUCTRL: u16 = 0x2000;
    /// PPU Mask Register ($2001) - Write only
    pub const PPUMASK: u16 = 0x2001;
    /// PPU Status Register ($2002) - Read only
    pub const PPUSTATUS: u16 = 0x2002;
    /// OAM Address Port ($2003) - Write only
    pub const OAMADDR: u16 = 0x2003;
    /// OAM Data Port ($2004) - Read/Write
    pub const OAMDATA: u16 = 0x2004;
    /// Scroll Position Register ($2005) - Write×2
    pub const PPUSCROLL: u16 = 0x2005;
    /// PPU Address Register ($2006) - Write×2
    pub const PPUADDR: u16 = 0x2006;
    /// PPU Data Port ($2007) - Read/Write
    pub const PPUDATA: u16 = 0x2007;
}

/// PPU register address mask for mirroring
///
/// PPU registers are 8 bytes ($2000-$2007) but mirrored throughout $2000-$3FFF.
/// Use this mask to get the actual register address: `addr & 0x2007` or `addr & 0x0007`
const PPU_REGISTER_MASK: u16 = 0x0007;

/// PPU structure representing the Picture Processing Unit state
///
/// This is a Phase 2 stub implementation. Registers accept writes and return
/// sensible default values on reads. Full PPU rendering will be implemented in Phase 4.
pub struct Ppu {
    // ========================================
    // PPU Registers ($2000-$2007)
    // ========================================
    /// $2000: PPUCTRL - Control register 1
    ppuctrl: u8,

    /// $2001: PPUMASK - Control register 2
    ppumask: u8,

    /// $2002: PPUSTATUS - Status register
    /// Bit 7: VBlank flag (cleared on read)
    /// Bit 6: Sprite 0 hit
    /// Bit 5: Sprite overflow
    ppustatus: u8,

    /// $2003: OAMADDR - OAM address
    oam_addr: u8,

    /// $2004: OAMDATA - OAM data buffer
    /// In the full implementation, this will access the OAM memory
    oam_data: u8,

    // ========================================
    // Internal State
    // ========================================
    /// Write latch (w register) used by PPUSCROLL and PPUADDR
    ///
    /// These registers require two consecutive writes:
    /// - false (0): Next write is the first write
    /// - true (1): Next write is the second write
    ///
    /// Reading PPUSTATUS resets this latch to false.
    write_latch: bool,

    /// Temporary storage for PPUADDR
    ///
    /// PPUADDR is written in two parts:
    /// - First write: High byte
    /// - Second write: Low byte
    ppu_addr_temp: u8,

    /// Current PPU address (v register)
    ///
    /// This is the actual address used when reading/writing PPUDATA.
    /// In the full implementation, this will be a 15-bit value.
    ppu_addr: u16,

    /// Read buffer for PPUDATA
    ///
    /// Reads from PPUDATA are buffered (delayed by one read) for addresses $0000-$3EFF.
    /// This simulates the PPU's internal read buffer.
    read_buffer: u8,
}

impl Ppu {
    /// Create a new PPU instance with default state
    ///
    /// Initializes all registers to their power-on state.
    ///
    /// # Returns
    ///
    /// A new PPU instance in its initial state
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let ppu = Ppu::new();
    /// ```
    pub fn new() -> Self {
        Ppu {
            // Registers
            ppuctrl: 0x00,
            ppumask: 0x00,
            ppustatus: 0x00,
            oam_addr: 0x00,
            oam_data: 0x00,

            // Internal state
            write_latch: false,
            ppu_addr_temp: 0x00,
            ppu_addr: 0x0000,
            read_buffer: 0x00,
        }
    }

    /// Reset PPU to power-on state
    ///
    /// Resets all registers and internal state to their default values.
    /// This simulates a power cycle or reset signal.
    pub fn reset(&mut self) {
        self.ppuctrl = 0x00;
        self.ppumask = 0x00;
        self.ppustatus = 0x00;
        self.oam_addr = 0x00;
        self.oam_data = 0x00;
        self.write_latch = false;
        self.ppu_addr_temp = 0x00;
        self.ppu_addr = 0x0000;
        self.read_buffer = 0x00;
    }

    /// Read from a PPU register
    ///
    /// # Arguments
    ///
    /// * `register` - The register number (0-7)
    ///
    /// # Returns
    ///
    /// The value read from the register
    ///
    /// # Register Behaviors
    ///
    /// - PPUSTATUS ($2002): Returns status, clears VBlank flag and address latch
    /// - OAMDATA ($2004): Returns OAM data (stub: returns 0)
    /// - PPUDATA ($2007): Returns buffered PPU data (stub: returns 0)
    /// - Write-only registers: Return 0
    fn read_register(&mut self, register: u16) -> u8 {
        match register {
            0 => {
                // $2000: PPUCTRL - Write only, return 0
                0
            }
            1 => {
                // $2001: PPUMASK - Write only, return 0
                0
            }
            2 => {
                // $2002: PPUSTATUS - Read only
                // Reading PPUSTATUS has side effects:
                // 1. Clears bit 7 (VBlank flag) after reading
                // 2. Resets the address latch used by PPUSCROLL and PPUADDR
                let status = self.ppustatus;

                // Clear VBlank flag (bit 7)
                self.ppustatus &= 0x7F;

                // Reset address latch
                self.write_latch = false;

                status
            }
            3 => {
                // $2003: OAMADDR - Write only, return 0
                0
            }
            4 => {
                // $2004: OAMDATA - Read/Write
                // Stub: return 0
                // Full implementation will read from OAM memory
                0
            }
            5 => {
                // $2005: PPUSCROLL - Write only, return 0
                0
            }
            6 => {
                // $2006: PPUADDR - Write only, return 0
                0
            }
            7 => {
                // $2007: PPUDATA - Read/Write
                // Stub: return buffered value (0)
                // Full implementation will read from PPU memory
                let value = self.read_buffer;
                self.read_buffer = 0; // Stub: would read from PPU memory here

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.ppu_addr = self.ppu_addr.wrapping_add(increment);

                value
            }
            _ => {
                // Should not reach here due to masking, but return 0 as fallback
                0
            }
        }
    }

    /// Write to a PPU register
    ///
    /// # Arguments
    ///
    /// * `register` - The register number (0-7)
    /// * `data` - The value to write
    ///
    /// # Register Behaviors
    ///
    /// - PPUCTRL ($2000): Stores control flags
    /// - PPUMASK ($2001): Stores mask flags
    /// - OAMADDR ($2003): Sets OAM address
    /// - OAMDATA ($2004): Writes to OAM (stub: does nothing)
    /// - PPUSCROLL ($2005): Sets scroll position (requires 2 writes)
    /// - PPUADDR ($2006): Sets PPU address (requires 2 writes)
    /// - PPUDATA ($2007): Writes to PPU memory (stub: does nothing)
    /// - Read-only registers: Writes are ignored
    fn write_register(&mut self, register: u16, data: u8) {
        match register {
            0 => {
                // $2000: PPUCTRL - Write only
                self.ppuctrl = data;
            }
            1 => {
                // $2001: PPUMASK - Write only
                self.ppumask = data;
            }
            2 => {
                // $2002: PPUSTATUS - Read only, ignore writes
            }
            3 => {
                // $2003: OAMADDR - Write only
                self.oam_addr = data;
            }
            4 => {
                // $2004: OAMDATA - Read/Write
                // Stub: store value but don't write to OAM
                // Full implementation will write to OAM memory
                self.oam_data = data;

                // Increment OAM address
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                // $2005: PPUSCROLL - Write×2
                // First write: X scroll
                // Second write: Y scroll
                // Stub: accept writes but don't use the values
                // Full implementation will update internal scroll registers
                self.write_latch = !self.write_latch;
            }
            6 => {
                // $2006: PPUADDR - Write×2
                // First write: High byte of address
                // Second write: Low byte of address
                if !self.write_latch {
                    // First write: high byte
                    self.ppu_addr_temp = data;
                    self.write_latch = true;
                } else {
                    // Second write: low byte
                    self.ppu_addr = ((self.ppu_addr_temp as u16) << 8) | (data as u16);
                    self.write_latch = false;
                }
            }
            7 => {
                // $2007: PPUDATA - Read/Write
                // Stub: accept write but don't write to PPU memory
                // Full implementation will write to PPU memory

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.ppu_addr = self.ppu_addr.wrapping_add(increment);
            }
            _ => {
                // Should not reach here due to masking, but ignore as fallback
            }
        }
    }
}

impl MemoryMappedDevice for Ppu {
    /// Read a byte from PPU registers
    ///
    /// The address is automatically masked to handle mirroring.
    /// PPU registers ($2000-$2007) are mirrored throughout $2000-$3FFF.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from (will be masked to 0-7)
    ///
    /// # Returns
    ///
    /// The byte value from the specified register
    fn read(&mut self, addr: u16) -> u8 {
        let register = addr & PPU_REGISTER_MASK;
        self.read_register(register)
    }

    /// Write a byte to PPU registers
    ///
    /// The address is automatically masked to handle mirroring.
    /// PPU registers ($2000-$2007) are mirrored throughout $2000-$3FFF.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to (will be masked to 0-7)
    /// * `data` - The byte value to write
    fn write(&mut self, addr: u16, data: u8) {
        let register = addr & PPU_REGISTER_MASK;
        self.write_register(register, data);
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::test_constants::*;
    use super::*;

    // ========================================
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
        assert_eq!(ppu.ppu_addr, 0x2000);
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
        ppu.ppu_addr = 0x2000;
        ppu.ppuctrl = 0x00; // Increment by 1

        ppu.read(PPUDATA);
        assert_eq!(ppu.ppu_addr, 0x2001);

        ppu.read(PPUDATA);
        assert_eq!(ppu.ppu_addr, 0x2002);
    }

    #[test]
    fn test_read_ppudata_increments_by_32() {
        let mut ppu = Ppu::new();
        ppu.ppu_addr = 0x2000;
        ppu.ppuctrl = 0x04; // Increment by 32

        ppu.read(PPUDATA);
        assert_eq!(ppu.ppu_addr, 0x2020);
    }

    #[test]
    fn test_write_ppudata_increments_addr() {
        let mut ppu = Ppu::new();
        ppu.ppu_addr = 0x2000;
        ppu.ppuctrl = 0x00; // Increment by 1

        ppu.write(PPUDATA, 0x42);
        assert_eq!(ppu.ppu_addr, 0x2001);
    }

    #[test]
    fn test_write_ppudata_increments_by_32() {
        let mut ppu = Ppu::new();
        ppu.ppu_addr = 0x2000;
        ppu.ppuctrl = 0x04; // Increment by 32

        ppu.write(PPUDATA, 0x42);
        assert_eq!(ppu.ppu_addr, 0x2020);
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
        assert_eq!(ppu.ppu_addr, 0x3F00);
    }

    #[test]
    fn test_ppuaddr_multiple_sequences() {
        let mut ppu = Ppu::new();

        // First sequence
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        assert_eq!(ppu.ppu_addr, 0x2000);

        // Second sequence
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x10);
        assert_eq!(ppu.ppu_addr, 0x3F10);
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
        assert_eq!(ppu.ppu_addr, 0x3F00);
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
        assert_eq!(ppu.ppu_addr, 0x2001);

        ppu.write(PPUDATA, 0x43);
        assert_eq!(ppu.ppu_addr, 0x2002);
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
}
