// PPU constants

/// PPU register address mask for mirroring
///
/// PPU registers are 8 bytes ($2000-$2007) but mirrored throughout $2000-$3FFF.
/// Use this mask to get the actual register address: `addr & 0x2007` or `addr & 0x0007`
pub(super) const PPU_REGISTER_MASK: u16 = 0x0007;

/// Size of nametable in bytes (1KB)
pub(super) const NAMETABLE_SIZE: usize = 1024;

/// Size of palette RAM in bytes
pub(super) const PALETTE_SIZE: usize = 32;

/// Screen width in pixels
pub(super) const SCREEN_WIDTH: usize = 256;

/// Screen height in pixels
pub(super) const SCREEN_HEIGHT: usize = 240;

/// Nametable width in tiles (32 tiles)
pub(super) const NAMETABLE_WIDTH: usize = 32;

/// Nametable height in tiles (30 tiles)
pub(super) const NAMETABLE_HEIGHT: usize = 30;

/// Tile size in pixels (8x8)
pub(super) const TILE_SIZE: usize = 8;

// ========================================
// PPU Timing Constants (NTSC)
// ========================================

/// Number of PPU cycles per scanline
pub(super) const CYCLES_PER_SCANLINE: u16 = 341;

/// Number of scanlines per frame (NTSC)
pub(super) const SCANLINES_PER_FRAME: u16 = 262;

/// Total PPU cycles per frame (NTSC)
/// 341 cycles/scanline × 262 scanlines = 89,342 cycles
#[allow(dead_code)]
pub(super) const CYCLES_PER_FRAME: u32 =
    (CYCLES_PER_SCANLINE as u32) * (SCANLINES_PER_FRAME as u32);

/// Pre-render scanline number
/// This is scanline 261 (or -1 in some documentation)
pub(super) const PRERENDER_SCANLINE: u16 = 261;

/// First visible scanline
pub(super) const FIRST_VISIBLE_SCANLINE: u16 = 0;

/// Last visible scanline
pub(super) const LAST_VISIBLE_SCANLINE: u16 = 239;

/// Post-render scanline
pub(super) const POSTRENDER_SCANLINE: u16 = 240;

/// First VBlank scanline
pub(super) const FIRST_VBLANK_SCANLINE: u16 = 241;

/// Last VBlank scanline
pub(super) const LAST_VBLANK_SCANLINE: u16 = 260;
