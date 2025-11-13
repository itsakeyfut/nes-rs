// PPU module - Picture Processing Unit implementation
// This module contains the PPU (2C02) emulation
//
// # PPU Registers (Phase 4 - Full Implementation)
//
// The PPU has 8 registers mapped at $2000-$2007 in CPU memory space.
// These registers are mirrored throughout $2008-$3FFF (repeating every 8 bytes).
//
// This implementation includes full PPU register behavior, including:
// - Proper internal scroll registers (v, t, x, w)
// - PPU memory (VRAM) with nametables and palette RAM
// - PPUDATA read buffering for non-palette addresses
// - Correct mirroring behavior
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
use crate::cartridge::Mirroring;

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

/// Size of nametable in bytes (1KB)
const NAMETABLE_SIZE: usize = 1024;

/// Size of palette RAM in bytes
const PALETTE_SIZE: usize = 32;

/// PPU structure representing the Picture Processing Unit state
///
/// This is a full implementation of PPU registers with proper behavior.
/// Includes PPU memory (VRAM), internal scroll registers, and all register behaviors.
pub struct Ppu {
    // ========================================
    // PPU Registers ($2000-$2007)
    // ========================================
    /// $2000: PPUCTRL - Control register 1
    ///
    /// Bit layout:
    /// - 7: Generate NMI at start of VBlank (0: off, 1: on)
    /// - 6: PPU master/slave select
    /// - 5: Sprite size (0: 8x8, 1: 8x16)
    /// - 4: Background pattern table address (0: $0000, 1: $1000)
    /// - 3: Sprite pattern table address (0: $0000, 1: $1000)
    /// - 2: VRAM address increment (0: +1, 1: +32)
    /// - 1-0: Base nametable address (0: $2000, 1: $2400, 2: $2800, 3: $2C00)
    ppuctrl: u8,

    /// $2001: PPUMASK - Control register 2
    ///
    /// Bit layout:
    /// - 7: Emphasize blue
    /// - 6: Emphasize green
    /// - 5: Emphasize red
    /// - 4: Show sprites (0: hide, 1: show)
    /// - 3: Show background (0: hide, 1: show)
    /// - 2: Show sprites in leftmost 8 pixels
    /// - 1: Show background in leftmost 8 pixels
    /// - 0: Grayscale (0: color, 1: grayscale)
    ppumask: u8,

    /// $2002: PPUSTATUS - Status register
    ///
    /// Bit layout:
    /// - 7: VBlank flag (cleared on read)
    /// - 6: Sprite 0 hit
    /// - 5: Sprite overflow
    /// - 4-0: Open bus (returns stale PPU bus value)
    ppustatus: u8,

    /// $2003: OAMADDR - OAM address
    oam_addr: u8,

    // ========================================
    // Internal Scroll Registers
    // ========================================
    /// v: Current VRAM address (15 bits)
    ///
    /// This is the actual address used when reading/writing PPUDATA.
    /// Also serves as the current scroll position during rendering.
    v: u16,

    /// t: Temporary VRAM address (15 bits)
    ///
    /// Also serves as temporary storage during address/scroll writes.
    /// Can be thought of as the "top-left" onscreen address.
    t: u16,

    /// x: Fine X scroll (3 bits)
    ///
    /// The fine X offset within the current tile (0-7 pixels).
    fine_x: u8,

    /// w: Write toggle (1 bit)
    ///
    /// Used by PPUSCROLL and PPUADDR to track which write is next.
    ///
    /// - false (0): First write
    /// - true (1): Second write
    ///
    /// Reading PPUSTATUS resets this to false.
    write_latch: bool,

    /// Read buffer for PPUDATA
    ///
    /// Reads from PPUDATA are buffered (delayed by one read) for addresses $0000-$3EFF.
    /// Palette reads ($3F00-$3FFF) are not buffered.
    read_buffer: u8,

    // ========================================
    // PPU Memory (VRAM)
    // ========================================
    /// Nametables: 2KB of internal VRAM
    ///
    /// The NES has 2KB of internal VRAM, which can be configured as:
    /// - Horizontal mirroring: $2000=$2400, $2800=$2C00
    /// - Vertical mirroring: $2000=$2800, $2400=$2C00
    /// - Four-screen: Requires external cartridge RAM (not implemented here)
    /// - Single-screen: All point to same nametable
    nametables: [u8; NAMETABLE_SIZE * 2],

    /// Palette RAM: 32 bytes
    ///
    /// Layout:
    /// - $3F00-$3F0F: Background palettes (4 palettes × 4 colors)
    /// - $3F10-$3F1F: Sprite palettes (4 palettes × 4 colors)
    ///
    /// Note: $3F10, $3F14, $3F18, $3F1C are mirrors of $3F00, $3F04, $3F08, $3F0C
    palette_ram: [u8; PALETTE_SIZE],

    /// Mirroring mode (from cartridge)
    mirroring: Mirroring,

    // ========================================
    // OAM Memory (Object Attribute Memory)
    // ========================================
    /// OAM (Object Attribute Memory) - 256 bytes
    ///
    /// Stores sprite data for 64 sprites (4 bytes per sprite):
    /// - Byte 0: Y position
    /// - Byte 1: Tile index
    /// - Byte 2: Attributes (palette, priority, flip)
    /// - Byte 3: X position
    oam: [u8; 256],
}

impl Ppu {
    /// Create a new PPU instance with default state
    ///
    /// Initializes all registers to their power-on state with horizontal mirroring.
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

            // Internal scroll registers
            v: 0x0000,
            t: 0x0000,
            fine_x: 0,
            write_latch: false,
            read_buffer: 0x00,

            // PPU memory
            nametables: [0; NAMETABLE_SIZE * 2],
            palette_ram: [0; PALETTE_SIZE],
            mirroring: Mirroring::Horizontal,

            // OAM memory
            oam: [0; 256],
        }
    }

    /// Reset PPU to power-on state
    ///
    /// Resets all registers and internal state to their default values.
    /// This simulates a power cycle or reset signal.
    /// Note: Mirroring mode is not reset as it comes from the cartridge.
    pub fn reset(&mut self) {
        self.ppuctrl = 0x00;
        self.ppumask = 0x00;
        self.ppustatus = 0x00;
        self.oam_addr = 0x00;
        self.v = 0x0000;
        self.t = 0x0000;
        self.fine_x = 0;
        self.write_latch = false;
        self.read_buffer = 0x00;
        self.nametables = [0; NAMETABLE_SIZE * 2];
        self.palette_ram = [0; PALETTE_SIZE];
        self.oam = [0; 256];
    }

    /// Set the mirroring mode
    ///
    /// This should be called when loading a cartridge to set the appropriate
    /// nametable mirroring mode.
    ///
    /// # Arguments
    ///
    /// * `mirroring` - The mirroring mode to use
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    /// use nes_rs::cartridge::Mirroring;
    ///
    /// let mut ppu = Ppu::new();
    /// ppu.set_mirroring(Mirroring::Vertical);
    /// ```
    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    /// Write directly to OAM memory (used by OAM DMA)
    ///
    /// This method is used by the OAM DMA transfer ($4014) to write directly
    /// to OAM memory without going through the OAMDATA register.
    ///
    /// # Arguments
    ///
    /// * `addr` - OAM address (0-255)
    /// * `data` - Byte value to write
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    /// ppu.write_oam(0, 0x50); // Write Y position of first sprite
    /// ```
    pub fn write_oam(&mut self, addr: u8, data: u8) {
        self.oam[addr as usize] = data;
    }

    /// Read directly from OAM memory (for testing)
    ///
    /// # Arguments
    ///
    /// * `addr` - OAM address (0-255)
    ///
    /// # Returns
    ///
    /// The byte value at the specified OAM address
    pub fn read_oam(&self, addr: u8) -> u8 {
        self.oam[addr as usize]
    }

    /// Mirror nametable address based on mirroring mode
    ///
    /// The PPU has 2KB of internal VRAM for nametables, but the address space
    /// allows for 4 nametables ($2000-$2FFF). This function maps a nametable
    /// address to the appropriate physical memory location based on the mirroring mode.
    ///
    /// # Arguments
    ///
    /// * `addr` - Nametable address ($2000-$2FFF)
    ///
    /// # Returns
    ///
    /// Physical VRAM address (0-2047)
    fn mirror_nametable_addr(&self, addr: u16) -> usize {
        // Normalize address to 0-0xFFF range (remove $2000 base)
        let addr = (addr & 0x0FFF) as usize;

        // Determine which nametable (0-3)
        let table = addr / NAMETABLE_SIZE;
        let offset = addr % NAMETABLE_SIZE;

        let mirrored_table = match self.mirroring {
            Mirroring::Horizontal => {
                // Horizontal: 0->0, 1->0, 2->1, 3->1
                // $2000=$2400, $2800=$2C00
                match table {
                    0 | 1 => 0,
                    2 | 3 => 1,
                    _ => unreachable!(),
                }
            }
            Mirroring::Vertical => {
                // Vertical: 0->0, 1->1, 2->0, 3->1
                // $2000=$2800, $2400=$2C00
                match table {
                    0 | 2 => 0,
                    1 | 3 => 1,
                    _ => unreachable!(),
                }
            }
            Mirroring::SingleScreen => {
                // All nametables point to the same physical table
                0
            }
            Mirroring::FourScreen => {
                // Four-screen would require 4KB of VRAM
                // For now, treat as horizontal mirroring
                // TODO: Implement four-screen VRAM when cartridge support is added
                match table {
                    0 | 1 => 0,
                    2 | 3 => 1,
                    _ => unreachable!(),
                }
            }
        };

        mirrored_table * NAMETABLE_SIZE + offset
    }

    /// Mirror palette address
    ///
    /// Palette RAM has special mirroring:
    /// - $3F10, $3F14, $3F18, $3F1C mirror $3F00, $3F04, $3F08, $3F0C
    /// - This is because sprite palette entry 0 is actually the background color
    ///
    /// # Arguments
    ///
    /// * `addr` - Palette address ($3F00-$3FFF)
    ///
    /// # Returns
    ///
    /// Physical palette RAM address (0-31)
    fn mirror_palette_addr(&self, addr: u16) -> usize {
        // Palette RAM is at $3F00-$3F1F, mirrored every 32 bytes
        let addr = (addr & 0x001F) as usize;

        // Special mirroring: $3F10, $3F14, $3F18, $3F1C -> $3F00, $3F04, $3F08, $3F0C
        if addr >= 16 && addr.is_multiple_of(4) {
            addr - 16
        } else {
            addr
        }
    }

    /// Read from PPU memory (VRAM)
    ///
    /// Handles reading from pattern tables (via cartridge), nametables, and palette RAM.
    /// This is the internal memory read function used by PPUDATA.
    ///
    /// # Arguments
    ///
    /// * `addr` - PPU memory address ($0000-$3FFF)
    ///
    /// # Returns
    ///
    /// The byte value at the specified address
    fn read_ppu_memory(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF; // Mirror to 14-bit address space

        match addr {
            // Pattern tables: $0000-$1FFF
            // TODO: Read from cartridge CHR-ROM/RAM
            0x0000..=0x1FFF => {
                // For now, return 0
                // In full implementation, this will read from cartridge CHR memory
                0
            }

            // Nametables: $2000-$2FFF
            0x2000..=0x2FFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr);
                self.nametables[mirrored_addr]
            }

            // Nametable mirrors: $3000-$3EFF -> $2000-$2EFF
            0x3000..=0x3EFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr - 0x1000);
                self.nametables[mirrored_addr]
            }

            // Palette RAM: $3F00-$3FFF
            0x3F00..=0x3FFF => {
                let mirrored_addr = self.mirror_palette_addr(addr);
                self.palette_ram[mirrored_addr]
            }

            _ => unreachable!(),
        }
    }

    /// Write to PPU memory (VRAM)
    ///
    /// Handles writing to pattern tables (via cartridge), nametables, and palette RAM.
    /// This is the internal memory write function used by PPUDATA.
    ///
    /// # Arguments
    ///
    /// * `addr` - PPU memory address ($0000-$3FFF)
    /// * `data` - Byte value to write
    fn write_ppu_memory(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF; // Mirror to 14-bit address space

        match addr {
            // Pattern tables: $0000-$1FFF
            // TODO: Write to cartridge CHR-RAM (if present)
            0x0000..=0x1FFF => {
                // For now, ignore writes
                // In full implementation, this will write to cartridge CHR-RAM if present
            }

            // Nametables: $2000-$2FFF
            0x2000..=0x2FFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr);
                self.nametables[mirrored_addr] = data;
            }

            // Nametable mirrors: $3000-$3EFF -> $2000-$2EFF
            0x3000..=0x3EFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr - 0x1000);
                self.nametables[mirrored_addr] = data;
            }

            // Palette RAM: $3F00-$3FFF
            0x3F00..=0x3FFF => {
                let mirrored_addr = self.mirror_palette_addr(addr);
                self.palette_ram[mirrored_addr] = data;
            }

            _ => unreachable!(),
        }
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
    /// - OAMDATA ($2004): Returns OAM data at current OAM address
    /// - PPUDATA ($2007): Returns buffered PPU data (palette reads are immediate)
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

                // Reset address latch (w register)
                self.write_latch = false;

                status
            }
            3 => {
                // $2003: OAMADDR - Write only, return 0
                0
            }
            4 => {
                // $2004: OAMDATA - Read/Write
                // Read from OAM at current OAM address
                self.oam[self.oam_addr as usize]
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
                // Reading from PPUDATA is buffered for addresses $0000-$3EFF
                // Palette reads ($3F00-$3FFF) are immediate but still update the buffer

                let addr = self.v & 0x3FFF;
                let value;

                if addr >= 0x3F00 {
                    // Palette reads are immediate (not buffered)
                    value = self.read_ppu_memory(addr);
                    // But still update the buffer with nametable data "underneath"
                    // This reads from the mirrored nametable address
                    self.read_buffer = self.read_ppu_memory(addr & 0x2FFF);
                } else {
                    // Normal reads are buffered
                    value = self.read_buffer;
                    self.read_buffer = self.read_ppu_memory(addr);
                }

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment) & 0x3FFF;

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
    /// - PPUCTRL ($2000): Stores control flags and updates nametable select in t
    /// - PPUMASK ($2001): Stores mask flags
    /// - OAMADDR ($2003): Sets OAM address
    /// - OAMDATA ($2004): Writes to OAM and increments address
    /// - PPUSCROLL ($2005): Sets scroll position (requires 2 writes, updates t and x)
    /// - PPUADDR ($2006): Sets PPU address (requires 2 writes, updates t then v)
    /// - PPUDATA ($2007): Writes to PPU memory and increments v
    /// - Read-only registers: Writes are ignored
    fn write_register(&mut self, register: u16, data: u8) {
        match register {
            0 => {
                // $2000: PPUCTRL - Write only
                self.ppuctrl = data;

                // Update nametable select bits in t register
                // t: ...GH.. ........ <- d: ......GH
                // (bits 10-11 of t from bits 0-1 of data)
                self.t = (self.t & 0xF3FF) | (((data as u16) & 0x03) << 10);
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
                // Write to OAM at current OAM address
                self.oam[self.oam_addr as usize] = data;

                // Increment OAM address
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                // $2005: PPUSCROLL - Write×2
                // This register uses complex bit manipulation to update the internal
                // scroll registers (t and fine_x)

                if !self.write_latch {
                    // First write: X scroll
                    // t: ....... ...ABCDE <- d: ABCDEFGH
                    // x:              FGH <- d: ABCDEFGH
                    self.t = (self.t & 0xFFE0) | ((data as u16) >> 3);
                    self.fine_x = data & 0x07;
                    self.write_latch = true;
                } else {
                    // Second write: Y scroll
                    // t: FGH..AB CDE..... <- d: ABCDEFGH
                    self.t = (self.t & 0x8FFF) | (((data as u16) & 0x07) << 12);
                    self.t = (self.t & 0xFC1F) | (((data as u16) & 0xF8) << 2);
                    self.write_latch = false;
                }
            }
            6 => {
                // $2006: PPUADDR - Write×2
                // First write: High byte of address
                // Second write: Low byte of address

                if !self.write_latch {
                    // First write: high byte
                    // t: .CDEFGH ........ <- d: ..CDEFGH
                    // t: X...... ........ <- 0
                    self.t = (self.t & 0x80FF) | (((data as u16) & 0x3F) << 8);
                    self.write_latch = true;
                } else {
                    // Second write: low byte
                    // t: ....... ABCDEFGH <- d: ABCDEFGH
                    // v: <...all bits...> <- t: <...all bits...>
                    self.t = (self.t & 0xFF00) | (data as u16);
                    self.v = self.t;
                    self.write_latch = false;
                }
            }
            7 => {
                // $2007: PPUDATA - Read/Write
                // Write to PPU memory at current address (v)
                self.write_ppu_memory(self.v, data);

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment) & 0x3FFF;
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

    // ========================================
    // PPU Address Space Masking Tests
    // ========================================

    #[test]
    fn test_ppuaddr_masks_high_byte() {
        let mut ppu = Ppu::new();

        // Write high byte with bits beyond 0x3F (should be masked to 6 bits)
        ppu.write(PPUADDR, 0xFF); // All bits set
        ppu.write(PPUADDR, 0xFF); // Low byte

        // Should be masked to 0x3FFF (high byte masked to 0x3F)
        assert_eq!(ppu.v, 0x3FFF);
    }

    #[test]
    fn test_ppuaddr_wraps_at_boundary() {
        let mut ppu = Ppu::new();

        // Set address to 0x3FFF
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0xFF);
        assert_eq!(ppu.v, 0x3FFF);

        // Increment by 1 should wrap to 0x0000
        ppu.ppuctrl = 0x00; // Increment by 1
        ppu.write(PPUDATA, 0x42);
        assert_eq!(ppu.v, 0x0000);
    }

    #[test]
    fn test_ppudata_read_masks_address() {
        let mut ppu = Ppu::new();
        ppu.ppuctrl = 0x00; // Increment by 1

        // Set address to near boundary
        ppu.v = 0x3FFF;

        // Read should increment and wrap
        ppu.read(PPUDATA);
        assert_eq!(ppu.v, 0x0000);
    }

    #[test]
    fn test_ppudata_increment_32_wraps() {
        let mut ppu = Ppu::new();
        ppu.ppuctrl = 0x04; // Increment by 32

        // Set address near boundary
        ppu.v = 0x3FF0;

        // Write should increment by 32 and wrap
        ppu.write(PPUDATA, 0x42);
        assert_eq!(ppu.v, 0x0010); // (0x3FF0 + 32) & 0x3FFF = 0x0010
    }

    #[test]
    fn test_ppuaddr_stays_within_14bit_range() {
        let mut ppu = Ppu::new();

        // Test various high bytes that exceed 0x3F
        for high_byte in 0x40..=0xFF {
            ppu.write(PPUADDR, high_byte);
            ppu.write(PPUADDR, 0x00);

            // Address should always be masked to 0x3FFF or less
            assert!(
                ppu.v <= 0x3FFF,
                "Address 0x{:04X} exceeds 14-bit range (high byte was 0x{:02X})",
                ppu.v,
                high_byte
            );
        }
    }

    // ========================================
    // PPU Memory (VRAM) Tests
    // ========================================

    #[test]
    fn test_nametable_read_write() {
        let mut ppu = Ppu::new();

        // Write to nametable
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x42);

        // Read back from nametable
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read (buffered)
        let value = ppu.read(PPUDATA); // Actual value
        assert_eq!(value, 0x42);
    }

    #[test]
    fn test_nametable_horizontal_mirroring() {
        let mut ppu = Ppu::new();
        ppu.set_mirroring(Mirroring::Horizontal);

        // Write to $2000
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x11);

        // Read from $2400 (should be same as $2000 with horizontal mirroring)
        ppu.write(PPUADDR, 0x24);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x11);

        // Write to $2800
        ppu.write(PPUADDR, 0x28);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x22);

        // Read from $2C00 (should be same as $2800)
        ppu.write(PPUADDR, 0x2C);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x22);
    }

    #[test]
    fn test_nametable_vertical_mirroring() {
        let mut ppu = Ppu::new();
        ppu.set_mirroring(Mirroring::Vertical);

        // Write to $2000
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x11);

        // Read from $2800 (should be same as $2000 with vertical mirroring)
        ppu.write(PPUADDR, 0x28);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x11);

        // Write to $2400
        ppu.write(PPUADDR, 0x24);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x22);

        // Read from $2C00 (should be same as $2400)
        ppu.write(PPUADDR, 0x2C);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x22);
    }

    #[test]
    fn test_nametable_mirror_at_3000() {
        let mut ppu = Ppu::new();

        // Write to $2000
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x42);

        // Read from $3000 (mirrors $2000)
        ppu.write(PPUADDR, 0x30);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x42);
    }

    #[test]
    fn test_palette_read_write() {
        let mut ppu = Ppu::new();

        // Write to background palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F); // Black

        // Palette reads are immediate (not buffered)
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x0F);
    }

    #[test]
    fn test_palette_mirroring() {
        let mut ppu = Ppu::new();

        // Write to $3F00 (background palette 0, color 0)
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F);

        // Read from $3F10 (sprite palette 0, color 0 - mirrors $3F00)
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x10);
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x0F);

        // Write to $3F04
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x04);
        ppu.write(PPUDATA, 0x30);

        // Read from $3F14 (mirrors $3F04)
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x14);
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x30);
    }

    #[test]
    fn test_ppudata_read_buffering() {
        let mut ppu = Ppu::new();

        // Write to nametable
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0xAA);
        ppu.write(PPUDATA, 0xBB);

        // First read is buffered (returns stale data)
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        let first_read = ppu.read(PPUDATA);
        assert_eq!(first_read, 0x00); // Stale buffer value

        // Second read returns the first byte
        let second_read = ppu.read(PPUDATA);
        assert_eq!(second_read, 0xAA);

        // Third read returns the second byte
        let third_read = ppu.read(PPUDATA);
        assert_eq!(third_read, 0xBB);
    }

    #[test]
    fn test_ppudata_palette_read_not_buffered() {
        let mut ppu = Ppu::new();

        // Write to palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F);

        // Palette reads are immediate (not buffered)
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x0F); // Immediate, not buffered
    }

    // ========================================
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

    // ========================================
    // PPUSCROLL Internal Register Tests
    // ========================================

    #[test]
    fn test_ppuscroll_updates_t_and_fine_x() {
        let mut ppu = Ppu::new();

        // First write: X scroll
        ppu.write(PPUSCROLL, 0xF8); // Binary: 11111000
        assert_eq!(ppu.t & 0x001F, 0x1F); // Coarse X = 11111 (0x1F)
        assert_eq!(ppu.fine_x, 0); // Fine X = 000

        // Second write: Y scroll
        ppu.write(PPUSCROLL, 0xE5); // Binary: 11100101
                                    // Fine Y (top 3 bits) should be in bits 12-14 of t: 111
        assert_eq!((ppu.t >> 12) & 0x07, 0x05); // Fine Y = 101
                                                // Coarse Y (bottom 5 bits) should be in bits 5-9 of t: 00101
        assert_eq!((ppu.t >> 5) & 0x1F, 0x1C); // Coarse Y = 11100
    }

    #[test]
    fn test_ppuscroll_write_latch() {
        let mut ppu = Ppu::new();

        assert!(!ppu.write_latch);

        // First write
        ppu.write(PPUSCROLL, 0x00);
        assert!(ppu.write_latch);

        // Second write
        ppu.write(PPUSCROLL, 0x00);
        assert!(!ppu.write_latch);

        // Third write (first again)
        ppu.write(PPUSCROLL, 0x00);
        assert!(ppu.write_latch);
    }

    // ========================================
    // PPUADDR Internal Register Tests
    // ========================================

    #[test]
    fn test_ppuaddr_updates_t_then_v() {
        let mut ppu = Ppu::new();

        // First write should update t but not v
        ppu.write(PPUADDR, 0x20);
        assert_eq!(ppu.t & 0xFF00, 0x2000);
        assert_eq!(ppu.v, 0x0000); // v should not change yet

        // Second write should update t and copy to v
        ppu.write(PPUADDR, 0x50);
        assert_eq!(ppu.t, 0x2050);
        assert_eq!(ppu.v, 0x2050); // v should now match t
    }

    #[test]
    fn test_ppuaddr_and_ppuscroll_share_write_latch() {
        let mut ppu = Ppu::new();

        // Start writing to PPUSCROLL
        ppu.write(PPUSCROLL, 0x10);
        assert!(ppu.write_latch);

        // Read PPUSTATUS should reset latch
        ppu.read(PPUSTATUS);
        assert!(!ppu.write_latch);

        // Next write to PPUADDR should be first write
        ppu.write(PPUADDR, 0x20);
        assert!(ppu.write_latch);
    }
}
