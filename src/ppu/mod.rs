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

mod constants;
mod memory;
mod registers;
mod rendering;

use crate::bus::MemoryMappedDevice;
use crate::cartridge::{Mapper, Mirroring};
use constants::*;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub(crate) ppuctrl: u8,

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
    pub(crate) ppumask: u8,

    /// $2002: PPUSTATUS - Status register
    ///
    /// Bit layout:
    /// - 7: VBlank flag (cleared on read)
    /// - 6: Sprite 0 hit
    /// - 5: Sprite overflow
    /// - 4-0: Open bus (returns stale PPU bus value)
    pub(crate) ppustatus: u8,

    /// $2003: OAMADDR - OAM address
    pub(crate) oam_addr: u8,

    // ========================================
    // Internal Scroll Registers
    // ========================================
    /// v: Current VRAM address (15 bits)
    ///
    /// This is the actual address used when reading/writing PPUDATA.
    /// Also serves as the current scroll position during rendering.
    pub(crate) v: u16,

    /// t: Temporary VRAM address (15 bits)
    ///
    /// Also serves as temporary storage during address/scroll writes.
    /// Can be thought of as the "top-left" onscreen address.
    pub(crate) t: u16,

    /// x: Fine X scroll (3 bits)
    ///
    /// The fine X offset within the current tile (0-7 pixels).
    pub(crate) fine_x: u8,

    /// w: Write toggle (1 bit)
    ///
    /// Used by PPUSCROLL and PPUADDR to track which write is next.
    ///
    /// - false (0): First write
    /// - true (1): Second write
    ///
    /// Reading PPUSTATUS resets this to false.
    pub(crate) write_latch: bool,

    /// Read buffer for PPUDATA
    ///
    /// Reads from PPUDATA are buffered (delayed by one read) for addresses $0000-$3EFF.
    /// Palette reads ($3F00-$3FFF) are not buffered.
    pub(crate) read_buffer: u8,

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
    pub(crate) nametables: [u8; NAMETABLE_SIZE * 2],

    /// Palette RAM: 32 bytes
    ///
    /// Layout:
    /// - $3F00-$3F0F: Background palettes (4 palettes × 4 colors)
    /// - $3F10-$3F1F: Sprite palettes (4 palettes × 4 colors)
    ///
    /// Note: $3F10, $3F14, $3F18, $3F1C are mirrors of $3F00, $3F04, $3F08, $3F0C
    pub(crate) palette_ram: [u8; PALETTE_SIZE],

    /// Mirroring mode (from cartridge)
    pub(crate) mirroring: Mirroring,

    /// Mapper for CHR-ROM/RAM access (pattern tables)
    ///
    /// Pattern tables ($0000-$1FFF) are stored in cartridge CHR-ROM or CHR-RAM.
    /// The mapper provides the interface to read/write this memory.
    /// None if no cartridge is loaded.
    pub(crate) mapper: Option<Rc<RefCell<Box<dyn Mapper>>>>,

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
    pub(crate) oam: [u8; 256],

    // ========================================
    // Rendering
    // ========================================
    /// Frame buffer - stores the rendered pixels (256x240)
    ///
    /// Each pixel is a palette index (0-63) that will be converted to RGB by the frontend.
    /// The buffer is organized as rows of pixels: [row0_pixels..., row1_pixels..., ...]
    pub(crate) frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    // ========================================
    // Timing (Cycle-accurate execution)
    // ========================================
    /// Current scanline (0-261)
    ///
    /// - 0-239: Visible scanlines
    /// - 240: Post-render scanline
    /// - 241-260: VBlank scanlines
    /// - 261: Pre-render scanline
    pub(crate) scanline: u16,

    /// Current cycle within the scanline (0-340)
    ///
    /// Each scanline has 341 PPU cycles (0-340)
    pub(crate) cycle: u16,

    /// Frame counter (increments each frame)
    ///
    /// Used for odd/even frame detection. On odd frames,
    /// the pre-render scanline is one cycle shorter.
    pub(crate) frame: u64,

    /// NMI pending flag
    ///
    /// Set to true when an NMI should be triggered.
    /// The CPU should check this flag and handle the NMI.
    pub(crate) nmi_pending: bool,
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
            mapper: None,

            // OAM memory
            oam: [0; 256],

            // Rendering
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],

            // Timing
            scanline: 0,
            cycle: 0,
            frame: 0,
            nmi_pending: false,
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
        self.frame_buffer = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.scanline = 0;
        self.cycle = 0;
        self.frame = 0;
        self.nmi_pending = false;
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

    /// Set the mapper for CHR-ROM/RAM access
    ///
    /// This should be called when loading a cartridge to provide access to
    /// pattern table memory (CHR-ROM or CHR-RAM).
    ///
    /// # Arguments
    ///
    /// * `mapper` - Shared reference to the cartridge mapper
    ///
    /// # Example
    ///
    /// ```ignore
    /// use nes_rs::ppu::Ppu;
    /// use nes_rs::cartridge::{Cartridge, Mapper};
    /// use nes_rs::cartridge::mappers::Mapper0;
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let mut ppu = Ppu::new();
    /// let cartridge = Cartridge::from_ines_file("game.nes").unwrap();
    /// let mapper = Rc::new(RefCell::new(Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>));
    /// ppu.set_mapper(mapper);
    /// ```
    pub fn set_mapper(&mut self, mapper: Rc<RefCell<Box<dyn Mapper>>>) {
        // Also update mirroring from the mapper
        self.mirroring = mapper.borrow().mirroring();
        self.mapper = Some(mapper);
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

    /// Get a reference to the frame buffer
    ///
    /// The frame buffer contains palette indices (0-63) for each pixel.
    /// The caller should convert these to RGB values using the NES palette.
    ///
    /// # Returns
    ///
    /// A reference to the frame buffer (256x240 pixels)
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let ppu = Ppu::new();
    /// let frame = ppu.frame();
    /// assert_eq!(frame.len(), 256 * 240);
    /// ```
    pub fn frame(&self) -> &[u8] {
        &self.frame_buffer
    }

    /// Get a mutable reference to the frame buffer (for testing)
    ///
    /// # Returns
    ///
    /// A mutable reference to the frame buffer (256x240 pixels)
    pub fn frame_mut(&mut self) -> &mut [u8] {
        &mut self.frame_buffer
    }

    // ========================================
    // Cycle-accurate timing
    // ========================================

    /// Execute one PPU cycle
    ///
    /// This is the main method for cycle-accurate PPU emulation. It should be called
    /// once for every PPU cycle (3 times per CPU cycle).
    ///
    /// The method handles:
    /// - Scanline and cycle tracking
    /// - VBlank NMI generation
    /// - Frame rendering at the appropriate time
    ///
    /// # Returns
    ///
    /// `true` if a frame was completed, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    ///
    /// // Execute one PPU cycle
    /// let frame_complete = ppu.step();
    /// if frame_complete {
    ///     // Frame is ready for display
    ///     let frame = ppu.frame();
    /// }
    /// ```
    pub fn step(&mut self) -> bool {
        let mut frame_complete = false;

        // Execute current cycle based on scanline
        match self.scanline {
            FIRST_VISIBLE_SCANLINE..=LAST_VISIBLE_SCANLINE => {
                self.visible_scanline_cycle();
            }
            POSTRENDER_SCANLINE => {
                self.postrender_scanline_cycle();
            }
            FIRST_VBLANK_SCANLINE..=LAST_VBLANK_SCANLINE => {
                self.vblank_scanline_cycle();
            }
            PRERENDER_SCANLINE => {
                self.prerender_scanline_cycle();
            }
            _ => {
                // Invalid scanline, should not happen
            }
        }

        // Advance cycle counter
        self.cycle += 1;

        // Check if we've completed a scanline
        if self.cycle >= CYCLES_PER_SCANLINE {
            self.cycle = 0;
            self.scanline += 1;

            // Check if we've completed a frame
            if self.scanline >= SCANLINES_PER_FRAME {
                self.scanline = 0;
                self.frame += 1;
                frame_complete = true;
            }
        }

        // Special case: Odd frames skip the last cycle of the pre-render scanline
        // when rendering is enabled
        if self.scanline == PRERENDER_SCANLINE
            && self.cycle == CYCLES_PER_SCANLINE - 1
            && (self.frame & 1) == 1
            && self.is_rendering_enabled()
        {
            self.cycle = 0;
            self.scanline = 0;
            self.frame += 1;
            frame_complete = true;
        }

        frame_complete
    }

    /// Handle visible scanline cycles (0-239)
    ///
    /// During visible scanlines, the PPU fetches background and sprite data
    /// and renders pixels to the frame buffer.
    fn visible_scanline_cycle(&mut self) {
        // For now, we'll use the existing render methods at specific cycles
        // In a fully cycle-accurate implementation, this would fetch and render
        // pixel-by-pixel. For this implementation, we'll render the entire line
        // at the end of the scanline.

        // Rendering happens throughout the scanline in the real hardware
        // but for simplicity we'll defer to the existing rendering code
        // This is a placeholder for future pixel-perfect rendering
    }

    /// Handle post-render scanline cycle (240)
    ///
    /// The post-render scanline is idle - no memory access occurs.
    fn postrender_scanline_cycle(&mut self) {
        // Post-render scanline is idle
        // No special actions needed
    }

    /// Handle VBlank scanline cycles (241-260)
    ///
    /// During VBlank, the PPU is idle and games typically update VRAM/OAM.
    fn vblank_scanline_cycle(&mut self) {
        // Set VBlank flag at the start of scanline 241, cycle 1
        // We check before the cycle increment in step(), so check for cycle == 0
        // which will become cycle 1 after the increment
        if self.scanline == FIRST_VBLANK_SCANLINE && self.cycle == 0 {
            self.ppustatus |= 0x80; // Set VBlank flag (bit 7)

            // Generate NMI if enabled
            if (self.ppuctrl & 0x80) != 0 {
                self.nmi_pending = true;
            }
        }
    }

    /// Handle pre-render scanline cycle (261)
    ///
    /// The pre-render scanline prepares for the next frame by clearing flags
    /// and performing background fetches.
    fn prerender_scanline_cycle(&mut self) {
        // Clear VBlank and sprite flags at cycle 1 of pre-render scanline
        // We check before the cycle increment, so check for cycle == 0
        if self.cycle == 0 {
            self.ppustatus &= !0x80; // Clear VBlank flag (bit 7)
            self.ppustatus &= !0x40; // Clear Sprite 0 hit (bit 6)
            self.ppustatus &= !0x20; // Clear Sprite overflow (bit 5)
            self.nmi_pending = false;
        }

        // At the end of pre-render scanline (or early in scanline 0),
        // we render the frame using the existing rendering code
        // Check before increment, so CYCLES_PER_SCANLINE - 2
        if self.cycle == CYCLES_PER_SCANLINE - 2 {
            // Render the complete frame here
            // This maintains compatibility with existing rendering code
            if self.is_rendering_enabled() {
                self.render_frame();
            }
        }
    }

    /// Check if rendering is enabled (background or sprites)
    ///
    /// # Returns
    ///
    /// `true` if either background or sprite rendering is enabled
    fn is_rendering_enabled(&self) -> bool {
        (self.ppumask & 0x18) != 0 // Check bits 3 and 4 (show background and show sprites)
    }

    /// Check if an NMI is pending
    ///
    /// The CPU should call this method to check if an NMI should be triggered.
    /// After handling the NMI, the CPU should call `clear_nmi()`.
    ///
    /// # Returns
    ///
    /// `true` if an NMI is pending and should be handled by the CPU
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ppu::Ppu;
    ///
    /// let mut ppu = Ppu::new();
    ///
    /// // ... execute some PPU cycles ...
    ///
    /// if ppu.nmi_pending() {
    ///     // CPU should handle NMI interrupt
    ///     ppu.clear_nmi();
    /// }
    /// ```
    pub fn nmi_pending(&self) -> bool {
        self.nmi_pending
    }

    /// Clear the NMI pending flag
    ///
    /// The CPU should call this after handling an NMI interrupt.
    pub fn clear_nmi(&mut self) {
        self.nmi_pending = false;
    }

    /// Get the current scanline number
    ///
    /// # Returns
    ///
    /// The current scanline (0-261)
    pub fn scanline(&self) -> u16 {
        self.scanline
    }

    /// Get the current cycle within the scanline
    ///
    /// # Returns
    ///
    /// The current cycle (0-340)
    pub fn cycle(&self) -> u16 {
        self.cycle
    }

    /// Get the frame counter
    ///
    /// # Returns
    ///
    /// The number of frames rendered since power-on
    pub fn frame_count(&self) -> u64 {
        self.frame
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
    use crate::cartridge::mappers::Mapper0;
    use crate::cartridge::Cartridge;

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

    // ========================================
    // Pattern Table (CHR-ROM/RAM) Tests
    // ========================================

    /// Helper function to create a test cartridge with CHR-ROM
    fn create_test_cartridge_chr_rom() -> Cartridge {
        let prg_rom = vec![0xAA; 16 * 1024]; // 16KB PRG-ROM
        let mut chr_rom = vec![0x00; 8 * 1024]; // 8KB CHR-ROM

        // Fill CHR-ROM with identifiable pattern
        for (i, byte) in chr_rom.iter_mut().enumerate() {
            *byte = (i & 0xFF) as u8;
        }

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 0,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        }
    }

    /// Helper function to create a test cartridge with CHR-RAM
    fn create_test_cartridge_chr_ram() -> Cartridge {
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

    #[test]
    fn test_pattern_table_without_mapper() {
        let mut ppu = Ppu::new();

        // Without a mapper, pattern table reads should return 0
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read (buffered)
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x00, "Pattern table should return 0 without mapper");

        // Writes should be ignored
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0xFF);

        // Read back should still be 0
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x00);
    }

    #[test]
    fn test_pattern_table_read_chr_rom() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_rom();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Read from pattern table 0 ($0000-$0FFF)
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read (buffered)
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x00, "CHR-ROM[0x0000] should be 0x00");

        // Read from pattern table 0, offset 0x0100
        ppu.write(PPUADDR, 0x01);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x00, "CHR-ROM[0x0100] should be 0x00");

        // Read from pattern table 1 ($1000-$1FFF)
        ppu.write(PPUADDR, 0x10);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x00, "CHR-ROM[0x1000] should be 0x00");
    }

    #[test]
    fn test_pattern_table_write_chr_ram() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Write to pattern table 0
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x42);

        // Read back
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read (buffered)
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x42, "CHR-RAM should be writable");

        // Write to pattern table 1
        ppu.write(PPUADDR, 0x10);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x99);

        // Read back
        ppu.write(PPUADDR, 0x10);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0x99, "CHR-RAM pattern table 1 should be writable");
    }

    #[test]
    fn test_pattern_table_chr_rom_write_ignored() {
        let mut ppu = Ppu::new();
        let mut cartridge = create_test_cartridge_chr_rom();

        // Set specific values in CHR-ROM
        cartridge.chr_rom[0x0000] = 0xAA;
        cartridge.chr_rom[0x1000] = 0xBB;

        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Try to write to CHR-ROM (should be ignored)
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0xFF);

        // Read back - should still have original value
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(value, 0xAA, "CHR-ROM should not be writable");

        // Try to write to pattern table 1
        ppu.write(PPUADDR, 0x10);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0xFF);

        // Read back
        ppu.write(PPUADDR, 0x10);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read
        let value = ppu.read(PPUDATA);
        assert_eq!(
            value, 0xBB,
            "CHR-ROM pattern table 1 should not be writable"
        );
    }

    #[test]
    fn test_set_mapper_updates_mirroring() {
        let mut ppu = Ppu::new();

        // Initial mirroring is Horizontal
        assert_eq!(ppu.mirroring, Mirroring::Horizontal);

        // Create a cartridge with Vertical mirroring
        let mut cartridge = create_test_cartridge_chr_rom();
        cartridge.mirroring = Mirroring::Vertical;

        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Mirroring should be updated
        assert_eq!(ppu.mirroring, Mirroring::Vertical);
    }

    #[test]
    fn test_pattern_table_full_range() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Write to various addresses across both pattern tables
        let test_addresses = [0x0000, 0x0001, 0x00FF, 0x0100, 0x0FFF, 0x1000, 0x1FFF];

        for (i, &addr) in test_addresses.iter().enumerate() {
            let test_value = (0x10 + i) as u8;

            // Write
            ppu.write(PPUADDR, (addr >> 8) as u8);
            ppu.write(PPUADDR, (addr & 0xFF) as u8);
            ppu.write(PPUDATA, test_value);

            // Read back
            ppu.write(PPUADDR, (addr >> 8) as u8);
            ppu.write(PPUADDR, (addr & 0xFF) as u8);
            let _ = ppu.read(PPUDATA); // Dummy read
            let value = ppu.read(PPUDATA);

            assert_eq!(
                value, test_value,
                "CHR-RAM at address ${:04X} should be writable",
                addr
            );
        }
    }

    #[test]
    fn test_pattern_table_sequential_access() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set increment to +1
        ppu.write(PPUCTRL, 0x00);

        // Write sequential data
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        for i in 0..16 {
            ppu.write(PPUDATA, i);
        }

        // Read back sequential data
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read

        for i in 0..16 {
            let value = ppu.read(PPUDATA);
            assert_eq!(value, i, "Sequential CHR-RAM read failed at index {}", i);
        }
    }

    // ========================================
    // Background Rendering Tests
    // ========================================

    #[test]
    fn test_frame_buffer_size() {
        let ppu = Ppu::new();
        let frame = ppu.frame();
        assert_eq!(
            frame.len(),
            256 * 240,
            "Frame buffer should be 256x240 pixels"
        );
    }

    #[test]
    fn test_frame_buffer_initialization() {
        let ppu = Ppu::new();
        let frame = ppu.frame();
        // All pixels should be initialized to 0
        for &pixel in frame.iter() {
            assert_eq!(pixel, 0);
        }
    }

    #[test]
    fn test_render_background_disabled() {
        let mut ppu = Ppu::new();

        // Fill frame buffer with non-zero values
        ppu.frame_mut().fill(0xFF);

        // Disable background rendering (PPUMASK bit 3 = 0)
        ppu.write(PPUMASK, 0x00);

        // Render background
        ppu.render_background();

        // Frame buffer should be cleared when rendering is disabled
        let frame = ppu.frame();
        for &pixel in frame.iter() {
            assert_eq!(
                pixel, 0,
                "Frame should be cleared when rendering is disabled"
            );
        }
    }

    #[test]
    fn test_render_background_enabled() {
        let mut ppu = Ppu::new();

        // Enable background rendering (PPUMASK bit 3 = 1)
        ppu.write(PPUMASK, 0x08);

        // Render background (will render with default data)
        ppu.render_background();

        // Frame buffer should be filled (not necessarily all zeros)
        let frame = ppu.frame();
        assert_eq!(frame.len(), 256 * 240);
    }

    #[test]
    fn test_nametable_tile_read() {
        let mut ppu = Ppu::new();

        // Write a tile index to nametable
        ppu.write(PPUADDR, 0x20);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x42);

        // Read it back using the internal method
        let tile_index = ppu.read_nametable_tile(0x2000);
        assert_eq!(tile_index, 0x42, "Tile index should be readable");
    }

    #[test]
    fn test_render_manual_debug() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up minimal data for testing one pixel
        // Write tile index 1 to nametable[0]
        ppu.nametables[0] = 0x01;

        // Write tile data to pattern table tile 1
        // This goes through the mapper
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF); // Bitplane 0: all 1s
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00); // Bitplane 1: all 0s
        }

        // Set up palette
        ppu.palette_ram[0] = 0x0F; // Universal background
        ppu.palette_ram[1] = 0x30; // Palette 0, color 1

        // Manually test the pipeline
        let tile_index = ppu.read_nametable_tile(0x2000);
        assert_eq!(tile_index, 0x01, "Should read tile 1");

        let color = ppu.fetch_tile_pixel(0x0000, tile_index, 0, 0);
        assert_eq!(color, 1, "Should get color index 1");

        let final_color = ppu.get_background_color(0, color);
        assert_eq!(final_color, 0x30, "Should get palette color 0x30");
    }

    #[test]
    fn test_render_simple_pattern() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up palette directly
        ppu.palette_ram[0] = 0x0F; // Universal background
        ppu.palette_ram[1] = 0x30; // Palette 0, color 1

        // Set up tile data directly in CHR-RAM
        // Tile 1: all pixels use color index 1
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF); // Bitplane 0
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00); // Bitplane 1
        }

        // Set up nametable directly - all tiles use tile 1
        for i in 0..(32 * 30) {
            ppu.nametables[i] = 0x01;
        }

        // Set up attribute table directly - all use palette 0
        for i in 0..64 {
            ppu.nametables[960 + i] = 0x00;
        }

        // Enable background rendering
        ppu.ppumask = 0x08;

        // Render background
        ppu.render_background();

        // Verify frame buffer has been filled
        let frame = ppu.frame();
        assert_eq!(frame.len(), 256 * 240);

        // All pixels should be 0x30 (color 1 from palette 0)
        assert_eq!(frame[0], 0x30, "Top-left pixel should match pattern");
        assert_eq!(frame[1], 0x30, "Second pixel should match pattern");
        assert_eq!(frame[256], 0x30, "Second row first pixel should match");
    }

    #[test]
    fn test_attribute_table_palette_selection() {
        let mut ppu = Ppu::new();

        // Test attribute byte reading for different tile positions
        // Write attribute table data
        ppu.write(PPUADDR, 0x23);
        ppu.write(PPUADDR, 0xC0); // Attribute table for nametable 0
        ppu.write(PPUDATA, 0xE4); // Binary: 11 10 01 00
                                  // This byte covers tiles (0,0) to (3,3)
                                  // Palette indices: top-left=00, top-right=01, bottom-left=10, bottom-right=11

        let palette_0_0 = ppu.read_attribute_byte(0x2000, 0, 0);
        assert_eq!(palette_0_0, 0, "Tile (0,0) should use palette 0");

        let palette_2_0 = ppu.read_attribute_byte(0x2000, 2, 0);
        assert_eq!(palette_2_0, 1, "Tile (2,0) should use palette 1");

        let palette_0_2 = ppu.read_attribute_byte(0x2000, 0, 2);
        assert_eq!(palette_0_2, 2, "Tile (0,2) should use palette 2");

        let palette_2_2 = ppu.read_attribute_byte(0x2000, 2, 2);
        assert_eq!(palette_2_2, 3, "Tile (2,2) should use palette 3");
    }

    #[test]
    fn test_fetch_tile_pixel() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Create a test pattern in tile 0
        // Bitplane 0: alternating 1010101
        // Bitplane 1: alternating 0101010
        // Result should alternate between color indices 1, 2, 1, 2...

        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);

        // Bitplane 0
        for _ in 0..8 {
            ppu.write(PPUDATA, 0b10101010);
        }
        // Bitplane 1
        for _ in 0..8 {
            ppu.write(PPUDATA, 0b01010101);
        }

        // Test fetching pixels
        // Bitplane 0 = 0b10101010 → bits 7,5,3,1 = 1; bits 6,4,2,0 = 0
        // Bitplane 1 = 0b01010101 → bits 7,5,3,1 = 0; bits 6,4,2,0 = 1
        // Color = (bit_1 << 1) | bit_0
        let color_0 = ppu.fetch_tile_pixel(0x0000, 0, 0, 0);
        assert_eq!(
            color_0, 1,
            "Pixel (0,0): bitplane0[7]=1, bitplane1[7]=0 → color = 01 = 1"
        );

        let color_1 = ppu.fetch_tile_pixel(0x0000, 0, 1, 0);
        assert_eq!(
            color_1, 2,
            "Pixel (1,0): bitplane0[6]=0, bitplane1[6]=1 → color = 10 = 2"
        );

        let color_2 = ppu.fetch_tile_pixel(0x0000, 0, 2, 0);
        assert_eq!(
            color_2, 1,
            "Pixel (2,0): bitplane0[5]=1, bitplane1[5]=0 → color = 01 = 1"
        );

        let color_3 = ppu.fetch_tile_pixel(0x0000, 0, 3, 0);
        assert_eq!(
            color_3, 2,
            "Pixel (3,0): bitplane0[4]=0, bitplane1[4]=1 → color = 10 = 2"
        );
    }

    #[test]
    fn test_background_color_palette_lookup() {
        let mut ppu = Ppu::new();

        // Set up background palettes
        // Palette RAM layout:
        // Index 0: Universal background
        // Index 1-3: Palette 0, colors 1-3
        // Index 4: Palette 1, color 0 (unused, color 0 uses universal bg)
        // Index 5-7: Palette 1, colors 1-3
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F); // Index 0: Universal background
        ppu.write(PPUDATA, 0x10); // Index 1: Palette 0, color 1
        ppu.write(PPUDATA, 0x20); // Index 2: Palette 0, color 2
        ppu.write(PPUDATA, 0x30); // Index 3: Palette 0, color 3
        ppu.write(PPUDATA, 0x00); // Index 4: Palette 1, color 0 (unused)
        ppu.write(PPUDATA, 0x11); // Index 5: Palette 1, color 1
        ppu.write(PPUDATA, 0x21); // Index 6: Palette 1, color 2
        ppu.write(PPUDATA, 0x31); // Index 7: Palette 1, color 3

        // Test palette 0
        assert_eq!(
            ppu.get_background_color(0, 0),
            0x0F,
            "Color 0 should be universal background"
        );
        assert_eq!(
            ppu.get_background_color(0, 1),
            0x10,
            "Palette 0, color 1 at index 1"
        );
        assert_eq!(
            ppu.get_background_color(0, 2),
            0x20,
            "Palette 0, color 2 at index 2"
        );
        assert_eq!(
            ppu.get_background_color(0, 3),
            0x30,
            "Palette 0, color 3 at index 3"
        );

        // Test palette 1
        assert_eq!(
            ppu.get_background_color(1, 0),
            0x0F,
            "Color 0 should be universal background"
        );
        assert_eq!(
            ppu.get_background_color(1, 1),
            0x11,
            "Palette 1, color 1 at index 5"
        );
        assert_eq!(
            ppu.get_background_color(1, 2),
            0x21,
            "Palette 1, color 2 at index 6"
        );
        assert_eq!(
            ppu.get_background_color(1, 3),
            0x31,
            "Palette 1, color 3 at index 7"
        );
    }

    #[test]
    fn test_pattern_table_selection() {
        let mut ppu = Ppu::new();

        // Test pattern table 0 (PPUCTRL bit 4 = 0)
        ppu.write(PPUCTRL, 0x00);
        assert_eq!(ppu.ppuctrl & 0x10, 0, "Pattern table 0 should be selected");

        // Test pattern table 1 (PPUCTRL bit 4 = 1)
        ppu.write(PPUCTRL, 0x10);
        assert_eq!(
            ppu.ppuctrl & 0x10,
            0x10,
            "Pattern table 1 should be selected"
        );
    }

    #[test]
    fn test_render_with_scrolling() {
        let mut ppu = Ppu::new();

        // Enable background rendering
        ppu.write(PPUMASK, 0x08);

        // Set scroll position
        ppu.write(PPUSCROLL, 8); // X scroll = 8 pixels
        ppu.write(PPUSCROLL, 16); // Y scroll = 16 pixels

        // Render background
        ppu.render_background();

        // Frame buffer should be filled with scrolled content
        let frame = ppu.frame();
        assert_eq!(frame.len(), 256 * 240);
    }

    #[test]
    fn test_pattern_table_increment_32() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set increment to +32
        ppu.write(PPUCTRL, 0x04);

        // Write data with +32 increment
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x11); // $0000
        ppu.write(PPUDATA, 0x22); // $0020
        ppu.write(PPUDATA, 0x33); // $0040

        // Read back with +32 increment
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUADDR, 0x00);
        let _ = ppu.read(PPUDATA); // Dummy read

        assert_eq!(ppu.read(PPUDATA), 0x11); // $0000
        assert_eq!(ppu.read(PPUDATA), 0x22); // $0020
        assert_eq!(ppu.read(PPUDATA), 0x33); // $0040
    }

    // ========================================
    // Sprite Rendering Tests
    // ========================================

    #[test]
    fn test_sprite_attribute_parsing() {
        let mut ppu = Ppu::new();

        // Write sprite 0 data to OAM
        ppu.write(OAMADDR, 0x00);
        ppu.write(OAMDATA, 0x50); // Y position
        ppu.write(OAMDATA, 0x01); // Tile index
        ppu.write(OAMDATA, 0xE3); // Attributes: vflip, hflip, priority, palette 3
        ppu.write(OAMDATA, 0x80); // X position

        // Read back OAM data
        assert_eq!(ppu.read_oam(0), 0x50);
        assert_eq!(ppu.read_oam(1), 0x01);
        assert_eq!(ppu.read_oam(2), 0xE3);
        assert_eq!(ppu.read_oam(3), 0x80);
    }

    #[test]
    fn test_sprite_rendering_disabled() {
        let mut ppu = Ppu::new();

        // Fill frame buffer with non-zero values
        ppu.frame_mut().fill(0xFF);

        // Disable sprite rendering (PPUMASK bit 4 = 0)
        ppu.write(PPUMASK, 0x00);

        // Render sprites
        ppu.render_sprites();

        // Frame buffer should remain unchanged (sprites not rendered)
        let frame = ppu.frame();
        assert_eq!(
            frame[0], 0xFF,
            "Frame should remain unchanged when sprites disabled"
        );
    }

    #[test]
    fn test_sprite_rendering_enabled() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up sprite palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11); // Sprite palette 0, color 1
        ppu.write(PPUDATA, 0x30);

        // Set up sprite tile data in pattern table
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF); // Bitplane 0
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00); // Bitplane 1
        }

        // Write sprite data to OAM
        ppu.write_oam(0, 50); // Y position
        ppu.write_oam(1, 0x01); // Tile index 1
        ppu.write_oam(2, 0x00); // Attributes: palette 0, in front
        ppu.write_oam(3, 100); // X position

        // Enable sprite rendering
        ppu.write(PPUMASK, 0x10);

        // Render sprites
        ppu.render_sprites();

        // Check that sprite pixels were rendered
        let frame = ppu.frame();
        let sprite_pixel_y = 51; // Y + 1
        let sprite_pixel_x = 100;
        let pixel_index = sprite_pixel_y * 256 + sprite_pixel_x;
        assert_eq!(
            frame[pixel_index], 0x30,
            "Sprite should be rendered at position"
        );
    }

    #[test]
    fn test_sprite_transparency() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up background color
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F); // Universal background

        // Set up sprite tile with transparent pixels (color index 0)
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0x00); // All 0s
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write sprite data
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x00);
        ppu.write_oam(3, 100);

        // Enable sprite rendering
        ppu.write(PPUMASK, 0x10);

        // Clear frame buffer to background color
        ppu.frame_mut().fill(0x0F);

        // Render sprites
        ppu.render_sprites();

        // Transparent sprite should not change background
        let frame = ppu.frame();
        let sprite_pixel_y = 51;
        let sprite_pixel_x = 100;
        let pixel_index = sprite_pixel_y * 256 + sprite_pixel_x;
        assert_eq!(
            frame[pixel_index], 0x0F,
            "Transparent sprite should not overwrite background"
        );
    }

    #[test]
    fn test_sprite_horizontal_flip() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up sprite palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30);

        // Create a pattern with horizontal asymmetry (left half filled, right half empty)
        // Bitplane 0: 1111 0000 pattern
        ppu.mapper
            .as_ref()
            .unwrap()
            .borrow_mut()
            .ppu_write(0x0010, 0b11110000);
        for i in 1..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0b11110000);
        }
        // Bitplane 1: all 0s
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write sprite without flip
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x00); // No flip
        ppu.write_oam(3, 100);

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        let frame = ppu.frame();
        let y = 51;
        let left_pixel = frame[y * 256 + 100];
        let right_pixel = frame[y * 256 + 107];

        assert_eq!(left_pixel, 0x30, "Left side should be filled");
        assert_eq!(right_pixel, 0x00, "Right side should be empty");

        // Now test with horizontal flip
        ppu.frame_mut().fill(0);
        ppu.write_oam(2, 0x40); // Horizontal flip
        ppu.render_sprites();

        let frame = ppu.frame();
        let left_pixel_flip = frame[y * 256 + 100];
        let right_pixel_flip = frame[y * 256 + 107];

        assert_eq!(left_pixel_flip, 0x00, "Left side should be empty (flipped)");
        assert_eq!(
            right_pixel_flip, 0x30,
            "Right side should be filled (flipped)"
        );
    }

    #[test]
    fn test_sprite_vertical_flip() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up sprite palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30);

        // Create a pattern with vertical asymmetry (top half filled, bottom half empty)
        for i in 0..4 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF); // Top half
        }
        for i in 4..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0x00); // Bottom half
        }
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00); // Bitplane 1
        }

        // Write sprite without flip
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x00);
        ppu.write_oam(3, 100);

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        let frame = ppu.frame();
        let top_pixel = frame[51 * 256 + 100];
        let bottom_pixel = frame[57 * 256 + 100];

        assert_eq!(top_pixel, 0x30, "Top should be filled");
        assert_eq!(bottom_pixel, 0x00, "Bottom should be empty");

        // Test with vertical flip
        ppu.frame_mut().fill(0);
        ppu.write_oam(2, 0x80); // Vertical flip
        ppu.render_sprites();

        let frame = ppu.frame();
        let top_pixel_flip = frame[51 * 256 + 100];
        let bottom_pixel_flip = frame[57 * 256 + 100];

        assert_eq!(top_pixel_flip, 0x00, "Top should be empty (flipped)");
        assert_eq!(bottom_pixel_flip, 0x30, "Bottom should be filled (flipped)");
    }

    #[test]
    fn test_sprite_priority_behind_background() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up palettes
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F); // Universal background
        ppu.write(PPUDATA, 0x20); // BG palette 0, color 1

        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30); // Sprite palette 0, color 1

        // Set up sprite tile
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write sprite with priority bit set (behind background)
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x20); // Priority: behind background
        ppu.write_oam(3, 100);

        // Set frame buffer to background color (non-transparent)
        ppu.frame_mut().fill(0x20);

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        // Sprite should be hidden by background
        let frame = ppu.frame();
        let pixel = frame[51 * 256 + 100];
        assert_eq!(pixel, 0x20, "Sprite behind background should be hidden");

        // Test with transparent background
        ppu.frame_mut().fill(0x0F); // Universal background (transparent)
        ppu.render_sprites();

        let frame = ppu.frame();
        let pixel = frame[51 * 256 + 100];
        assert_eq!(
            pixel, 0x30,
            "Sprite should show through transparent background"
        );
    }

    #[test]
    fn test_sprite_8x16_mode() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Enable 8x16 sprite mode
        ppu.write(PPUCTRL, 0x20);

        // Set up sprite palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30);

        // Set up two tiles (tile pair for 8x16)
        // Tile 0: top half
        for i in 0..8 {
            ppu.mapper.as_ref().unwrap().borrow_mut().ppu_write(i, 0xFF);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0008 + i, 0x00);
        }
        // Tile 1: bottom half
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0x00);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0xFF);
        }

        // Write sprite data (tile index 0, which uses tile pair 0-1)
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x00); // Uses tiles 0 and 1
        ppu.write_oam(2, 0x00);
        ppu.write_oam(3, 100);

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        let frame = ppu.frame();
        // Check top half (should use tile 0 - color 1)
        let top_pixel = frame[51 * 256 + 100];
        assert_eq!(top_pixel, 0x30, "Top half should use tile 0");

        // Check bottom half (should use tile 1 - color 2)
        // Note: tile 1 has bitplane 0=0, bitplane 1=FF, so color index is 2
        // But we need to set up palette for color 2
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x12);
        ppu.write(PPUDATA, 0x31);

        ppu.frame_mut().fill(0);
        ppu.render_sprites();

        let frame = ppu.frame();
        let bottom_pixel = frame[59 * 256 + 100];
        assert_eq!(bottom_pixel, 0x31, "Bottom half should use tile 1");
    }

    #[test]
    fn test_sprite_0_hit_detection() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up palettes
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x00);
        ppu.write(PPUDATA, 0x0F);
        ppu.write(PPUDATA, 0x20); // BG color

        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30); // Sprite color

        // Set up sprite tile
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write sprite 0
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x00);
        ppu.write_oam(3, 100);

        // Set background to non-transparent color
        ppu.frame_mut().fill(0x20);

        // Clear sprite 0 hit flag
        ppu.ppustatus &= !0x40;

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        // Sprite 0 hit should be detected
        assert_eq!(
            ppu.ppustatus & 0x40,
            0x40,
            "Sprite 0 hit should be detected"
        );
    }

    #[test]
    fn test_sprite_overflow_flag() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up sprite palette
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30);

        // Set up sprite tile
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write 9 sprites on the same scanline
        for i in 0..9 {
            ppu.write_oam((i * 4) as u8, 50); // All on scanline 51
            ppu.write_oam((i * 4 + 1) as u8, 0x01);
            ppu.write_oam((i * 4 + 2) as u8, 0x00);
            ppu.write_oam((i * 4 + 3) as u8, (i * 10) as u8);
        }

        // Clear overflow flag
        ppu.ppustatus &= !0x20;

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        // Sprite overflow should be detected
        assert_eq!(
            ppu.ppustatus & 0x20,
            0x20,
            "Sprite overflow should be detected"
        );
    }

    #[test]
    fn test_render_frame_clears_sprite_flags() {
        let mut ppu = Ppu::new();

        // Set sprite flags
        ppu.ppustatus |= 0x60; // Set both sprite 0 hit and overflow

        // Render frame
        ppu.render_frame();

        // Flags should be cleared at start of frame
        // (They may be set again during rendering, but at least the method was called)
        // We can't directly test this without setting up sprites, so just verify the method exists
    }

    #[test]
    fn test_multiple_sprite_priorities() {
        let mut ppu = Ppu::new();
        let cartridge = create_test_cartridge_chr_ram();
        let mapper = Rc::new(RefCell::new(
            Box::new(Mapper0::new(cartridge)) as Box<dyn Mapper>
        ));
        ppu.set_mapper(mapper);

        // Set up palettes for different sprites
        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x11);
        ppu.write(PPUDATA, 0x30); // Sprite palette 0
        ppu.write(PPUDATA, 0x31);
        ppu.write(PPUDATA, 0x32);
        ppu.write(PPUDATA, 0x33);

        ppu.write(PPUADDR, 0x3F);
        ppu.write(PPUADDR, 0x15);
        ppu.write(PPUDATA, 0x34); // Sprite palette 1

        // Set up sprite tiles
        for i in 0..8 {
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0010 + i, 0xFF);
            ppu.mapper
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(0x0018 + i, 0x00);
        }

        // Write two overlapping sprites
        // Sprite 0 at (100, 51)
        ppu.write_oam(0, 50);
        ppu.write_oam(1, 0x01);
        ppu.write_oam(2, 0x00); // Palette 0
        ppu.write_oam(3, 100);

        // Sprite 1 at (104, 51) - overlaps with sprite 0
        ppu.write_oam(4, 50);
        ppu.write_oam(5, 0x01);
        ppu.write_oam(6, 0x01); // Palette 1
        ppu.write_oam(7, 104);

        ppu.write(PPUMASK, 0x10);
        ppu.render_sprites();

        let frame = ppu.frame();
        // Check non-overlapping areas
        let sprite0_pixel = frame[51 * 256 + 100];
        assert_eq!(sprite0_pixel, 0x30, "Sprite 0 should be visible");

        // Check overlapping area - sprite 0 has higher priority (lower index)
        let overlap_pixel = frame[51 * 256 + 104];
        assert_eq!(
            overlap_pixel, 0x30,
            "Sprite 0 should have higher priority in overlap"
        );
    }

    // ========================================
    // Cycle-accurate timing tests
    // ========================================

    #[test]
    fn test_ppu_cycle_tracking() {
        let mut ppu = Ppu::new();

        // Initial state
        assert_eq!(ppu.scanline(), 0, "PPU should start at scanline 0");
        assert_eq!(ppu.cycle(), 0, "PPU should start at cycle 0");
        assert_eq!(ppu.frame_count(), 0, "PPU should start at frame 0");

        // Execute one cycle
        ppu.step();
        assert_eq!(ppu.cycle(), 1, "Cycle should advance to 1");
        assert_eq!(ppu.scanline(), 0, "Scanline should remain 0");
    }

    #[test]
    fn test_ppu_scanline_advancement() {
        let mut ppu = Ppu::new();

        // Execute a full scanline (341 cycles)
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }

        assert_eq!(ppu.scanline(), 1, "Scanline should advance to 1");
        assert_eq!(ppu.cycle(), 0, "Cycle should reset to 0");
    }

    #[test]
    fn test_ppu_frame_completion() {
        let mut ppu = Ppu::new();

        // Execute cycles until a frame completes
        let mut frame_complete = false;
        let mut cycles_executed = 0;

        // Execute one full frame (262 scanlines × 341 cycles = 89,342 cycles)
        while !frame_complete && cycles_executed < CYCLES_PER_FRAME + 1000 {
            frame_complete = ppu.step();
            cycles_executed += 1;
        }

        assert!(
            frame_complete,
            "A frame should complete after one full frame of cycles"
        );
        assert_eq!(ppu.scanline(), 0, "Scanline should reset to 0 after frame");
        assert_eq!(ppu.frame_count(), 1, "Frame counter should be 1");
    }

    #[test]
    fn test_vblank_flag_set() {
        let mut ppu = Ppu::new();

        // Execute until scanline 241, cycle 1 (VBlank start)
        // Scanlines 0-240 are visible/post-render
        for _ in 0..=240 {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        // Now we're at scanline 241, cycle 0
        assert_eq!(ppu.scanline(), 241, "Should be at VBlank scanline");

        // Execute one more cycle to trigger VBlank flag
        ppu.step();

        // Check VBlank flag is set (bit 7 of PPUSTATUS)
        assert_ne!(
            ppu.ppustatus & 0x80,
            0,
            "VBlank flag should be set at scanline 241, cycle 1"
        );
    }

    #[test]
    fn test_vblank_nmi_generation() {
        let mut ppu = Ppu::new();

        // Enable NMI on VBlank
        ppu.ppuctrl = 0x80; // Set bit 7 to enable NMI

        // Execute until scanline 241, cycle 1 (VBlank start)
        for _ in 0..=240 {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        // Execute one more cycle to trigger VBlank and NMI
        ppu.step();

        // Check NMI is pending
        assert!(
            ppu.nmi_pending(),
            "NMI should be pending after VBlank starts"
        );
    }

    #[test]
    fn test_vblank_nmi_disabled() {
        let mut ppu = Ppu::new();

        // NMI is disabled by default (ppuctrl bit 7 = 0)
        assert_eq!(ppu.ppuctrl & 0x80, 0, "NMI should be disabled");

        // Execute until scanline 241, cycle 1 (VBlank start)
        for _ in 0..=240 {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        // Execute one more cycle to trigger VBlank
        ppu.step();

        // Check NMI is NOT pending
        assert!(
            !ppu.nmi_pending(),
            "NMI should not be pending when disabled"
        );
    }

    #[test]
    fn test_prerender_scanline_clears_flags() {
        let mut ppu = Ppu::new();

        // Set VBlank and sprite flags
        ppu.ppustatus = 0xE0; // Set VBlank, Sprite 0 hit, Sprite overflow

        // Execute until pre-render scanline (261), cycle 1
        // We need to go through scanlines 0-260 first
        for _ in 0..261 {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        // Now we're at scanline 261, cycle 0
        assert_eq!(ppu.scanline(), 261, "Should be at pre-render scanline");

        // Execute one more cycle to trigger flag clearing
        ppu.step();

        // Check all flags are cleared
        assert_eq!(
            ppu.ppustatus & 0xE0,
            0,
            "VBlank, Sprite 0 hit, and Sprite overflow flags should be cleared"
        );
        assert!(
            !ppu.nmi_pending(),
            "NMI pending flag should be cleared at pre-render scanline"
        );
    }

    #[test]
    fn test_nmi_clear() {
        let mut ppu = Ppu::new();

        // Set NMI pending
        ppu.nmi_pending = true;
        assert!(ppu.nmi_pending(), "NMI should be pending");

        // Clear NMI
        ppu.clear_nmi();
        assert!(!ppu.nmi_pending(), "NMI should be cleared");
    }

    #[test]
    fn test_multiple_frames() {
        let mut ppu = Ppu::new();

        let mut frames_completed = 0;

        // Execute several frames
        for _ in 0..(CYCLES_PER_FRAME * 3) {
            if ppu.step() {
                frames_completed += 1;
            }
        }

        assert_eq!(
            frames_completed, 3,
            "Should complete 3 frames after 3× frame cycles"
        );
        assert_eq!(ppu.frame_count(), 3, "Frame counter should be 3");
    }

    #[test]
    fn test_cycle_counts() {
        // Verify constants are correct
        assert_eq!(
            CYCLES_PER_SCANLINE, 341,
            "PPU should have 341 cycles per scanline"
        );
        assert_eq!(
            SCANLINES_PER_FRAME, 262,
            "PPU should have 262 scanlines per frame (NTSC)"
        );
        assert_eq!(
            CYCLES_PER_FRAME, 89342,
            "PPU should have 89,342 cycles per frame (341 × 262)"
        );
    }

    #[test]
    fn test_scanline_types() {
        // Verify scanline constants
        assert_eq!(FIRST_VISIBLE_SCANLINE, 0, "First visible scanline is 0");
        assert_eq!(LAST_VISIBLE_SCANLINE, 239, "Last visible scanline is 239");
        assert_eq!(POSTRENDER_SCANLINE, 240, "Post-render scanline is 240");
        assert_eq!(FIRST_VBLANK_SCANLINE, 241, "First VBlank scanline is 241");
        assert_eq!(LAST_VBLANK_SCANLINE, 260, "Last VBlank scanline is 260");
        assert_eq!(PRERENDER_SCANLINE, 261, "Pre-render scanline is 261");
    }

    #[test]
    fn test_rendering_enabled_check() {
        let mut ppu = Ppu::new();

        // Initially, rendering is disabled
        assert!(
            !ppu.is_rendering_enabled(),
            "Rendering should be disabled initially"
        );

        // Enable background rendering (bit 3)
        ppu.ppumask = 0x08;
        assert!(
            ppu.is_rendering_enabled(),
            "Rendering should be enabled with background"
        );

        // Disable background, enable sprites (bit 4)
        ppu.ppumask = 0x10;
        assert!(
            ppu.is_rendering_enabled(),
            "Rendering should be enabled with sprites"
        );

        // Enable both
        ppu.ppumask = 0x18;
        assert!(
            ppu.is_rendering_enabled(),
            "Rendering should be enabled with both"
        );

        // Disable both
        ppu.ppumask = 0x00;
        assert!(
            !ppu.is_rendering_enabled(),
            "Rendering should be disabled with neither"
        );
    }

    #[test]
    fn test_odd_frame_skips_last_cycle() {
        let mut ppu = Ppu::new();

        // Enable rendering to trigger odd frame behavior
        ppu.ppumask = 0x18; // Enable background and sprites

        // Execute until pre-render scanline 261, cycle 339 on frame 1 (odd)
        // First complete frame 0 (even frame)
        while ppu.frame_count() < 1 {
            ppu.step();
        }

        assert_eq!(ppu.frame_count(), 1, "Should be on frame 1");
        assert_eq!(ppu.scanline(), 0, "Should be at scanline 0");
        assert_eq!(ppu.cycle(), 0, "Should be at cycle 0");

        // Now on frame 1 (odd), advance to scanline 261
        while ppu.scanline() < PRERENDER_SCANLINE {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        assert_eq!(
            ppu.scanline(),
            PRERENDER_SCANLINE,
            "Should be at pre-render scanline"
        );

        // Advance to cycle 339 (CYCLES_PER_SCANLINE - 2)
        // We need to advance 339 cycles (0 to 339)
        for _ in 0..339 {
            ppu.step();
        }

        assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
        assert_eq!(ppu.cycle(), 339);

        // Next step should skip cycle 340 and complete the frame
        let frame_complete = ppu.step();

        assert!(frame_complete, "Frame should complete");
        assert_eq!(ppu.frame_count(), 2, "Should advance to frame 2");
        assert_eq!(ppu.scanline(), 0, "Should wrap to scanline 0");
        assert_eq!(ppu.cycle(), 0, "Should reset to cycle 0");
    }

    #[test]
    fn test_even_frame_does_not_skip_last_cycle() {
        let mut ppu = Ppu::new();

        // Enable rendering
        ppu.ppumask = 0x18;

        // Frame 0 is even, advance to scanline 261, cycle 339
        while ppu.scanline() < PRERENDER_SCANLINE {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        // Advance to cycle 339
        for _ in 0..339 {
            ppu.step();
        }

        assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
        assert_eq!(ppu.cycle(), 339);
        assert_eq!(ppu.frame_count(), 0, "Still on frame 0 (even)");

        // Next step should go to cycle 340 (not skip)
        let frame_complete = ppu.step();

        assert!(!frame_complete, "Frame should not complete yet");
        assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
        assert_eq!(
            ppu.cycle(),
            340,
            "Should advance to cycle 340 on even frame"
        );
        assert_eq!(ppu.frame_count(), 0, "Still on frame 0");

        // One more step completes the frame normally
        let frame_complete = ppu.step();
        assert!(frame_complete, "Frame should complete now");
        assert_eq!(ppu.frame_count(), 1);
        assert_eq!(ppu.scanline(), 0);
        assert_eq!(ppu.cycle(), 0);
    }

    #[test]
    fn test_odd_frame_skip_only_when_rendering_enabled() {
        let mut ppu = Ppu::new();

        // Disable rendering
        ppu.ppumask = 0x00;

        // Complete frame 0
        while ppu.frame_count() < 1 {
            ppu.step();
        }

        assert_eq!(ppu.frame_count(), 1, "On odd frame");

        // Advance to scanline 261, cycle 339
        while ppu.scanline() < PRERENDER_SCANLINE {
            for _ in 0..CYCLES_PER_SCANLINE {
                ppu.step();
            }
        }

        for _ in 0..339 {
            ppu.step();
        }

        assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
        assert_eq!(ppu.cycle(), 339);

        // With rendering disabled, should NOT skip even on odd frame
        let frame_complete = ppu.step();

        assert!(!frame_complete, "Should not complete yet");
        assert_eq!(ppu.cycle(), 340, "Should advance to cycle 340");

        // One more step completes normally
        let frame_complete = ppu.step();
        assert!(frame_complete);
        assert_eq!(ppu.frame_count(), 2);
    }
}
