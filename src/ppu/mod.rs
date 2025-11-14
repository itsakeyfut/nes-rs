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
mod tests;
