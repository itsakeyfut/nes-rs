// NES Color Palette - RGB conversions for all 64 palette entries
//
// The NES has a master palette of 64 colors (indexed 0x00-0x3F).
// This includes 52 unique colors plus some duplicates and unused entries.
//
// Color indices $0D, $1D, $2D, $3D are problematic blacks that can cause
// display issues on real hardware, so they're typically avoided.
//
// Indices $0E-$0F, $1E-$1F, $2E-$2F, $3E-$3F are unused and typically render as black.

/// NES master palette in RGB format (64 colors)
///
/// Each color is represented as a 32-bit value: 0xRRGGBB
/// The palette uses a standard RGB conversion that approximates the NTSC NES output.
pub const NES_PALETTE: [u32; 64] = [
    // $00-$0F
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00,
    0x333500, 0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000,
    // $10-$1F
    0xADADAD, 0x155FD9, 0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00,
    0x6B6D00, 0x388700, 0x0C9300, 0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000,
    // $20-$2F
    0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF, 0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22,
    0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE, 0x4F4F4F, 0x000000, 0x000000,
    // $30-$3F
    0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA, 0xFECCC5, 0xF7D8A5,
    0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000, 0x000000,
];

/// Convert a NES palette index to RGB color
///
/// # Arguments
/// * `index` - Palette index (0x00-0x3F)
///
/// # Returns
/// 32-bit RGB color value (0xRRGGBB)
#[inline]
pub fn palette_to_rgb(index: u8) -> u32 {
    NES_PALETTE[(index & 0x3F) as usize]
}

/// Convert RGB color to RGBA format expected by pixels crate
///
/// # Arguments
/// * `rgb` - 32-bit RGB color (0xRRGGBB)
///
/// # Returns
/// Array of [R, G, B, A] bytes
#[inline]
pub fn rgb_to_rgba(rgb: u32) -> [u8; 4] {
    [
        ((rgb >> 16) & 0xFF) as u8, // Red
        ((rgb >> 8) & 0xFF) as u8,  // Green
        (rgb & 0xFF) as u8,         // Blue
        0xFF,                       // Alpha (fully opaque)
    ]
}

/// Convert NES palette index directly to RGBA bytes
///
/// # Arguments
/// * `index` - Palette index (0x00-0x3F)
///
/// # Returns
/// Array of [R, G, B, A] bytes
#[inline]
pub fn palette_to_rgba(index: u8) -> [u8; 4] {
    rgb_to_rgba(palette_to_rgb(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_size() {
        assert_eq!(NES_PALETTE.len(), 64);
    }

    #[test]
    fn test_palette_to_rgb() {
        // Test first color (dark gray)
        assert_eq!(palette_to_rgb(0x00), 0x666666);

        // Test a known color (white)
        assert_eq!(palette_to_rgb(0x30), 0xFFFEFF);

        // Test with mask (index >= 64 should wrap)
        assert_eq!(palette_to_rgb(0x40), palette_to_rgb(0x00));
    }

    #[test]
    fn test_rgb_to_rgba() {
        let rgba = rgb_to_rgba(0x123456);
        assert_eq!(rgba, [0x12, 0x34, 0x56, 0xFF]);
    }

    #[test]
    fn test_palette_to_rgba() {
        let rgba = palette_to_rgba(0x00);
        assert_eq!(rgba, [0x66, 0x66, 0x66, 0xFF]);
    }
}
