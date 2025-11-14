// Frame Buffer - Stores pixel data for NES display output
//
// The NES has a resolution of 256×240 pixels. Each pixel is represented
// by a palette index (0-63) which maps to an RGB color.

use super::palette::palette_to_rgba;

/// NES screen width in pixels
pub const SCREEN_WIDTH: usize = 256;

/// NES screen height in pixels
pub const SCREEN_HEIGHT: usize = 240;

/// Total number of pixels in the frame buffer
pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Frame buffer for storing pixel data
///
/// Stores palette indices for each pixel (256×240 = 61,440 pixels).
/// The frame buffer can be converted to RGBA format for display.
pub struct FrameBuffer {
    /// Pixel data stored as palette indices (0-63)
    pixels: [u8; SCREEN_SIZE],
}

impl FrameBuffer {
    /// Create a new frame buffer initialized to black (palette index 0x0F)
    pub fn new() -> Self {
        Self {
            pixels: [0x0F; SCREEN_SIZE],
        }
    }

    /// Set a pixel at the given coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-255)
    /// * `y` - Y coordinate (0-239)
    /// * `palette_index` - Palette index (0-63)
    ///
    /// # Panics
    /// Panics if coordinates are out of bounds
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
        assert!(x < SCREEN_WIDTH, "X coordinate {} out of bounds", x);
        assert!(y < SCREEN_HEIGHT, "Y coordinate {} out of bounds", y);

        self.pixels[y * SCREEN_WIDTH + x] = palette_index & 0x3F;
    }

    /// Get a pixel at the given coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-255)
    /// * `y` - Y coordinate (0-239)
    ///
    /// # Returns
    /// Palette index (0-63)
    ///
    /// # Panics
    /// Panics if coordinates are out of bounds
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        assert!(x < SCREEN_WIDTH, "X coordinate {} out of bounds", x);
        assert!(y < SCREEN_HEIGHT, "Y coordinate {} out of bounds", y);

        self.pixels[y * SCREEN_WIDTH + x]
    }

    /// Clear the frame buffer to a specific palette index
    ///
    /// # Arguments
    /// * `palette_index` - Palette index to fill (0-63)
    pub fn clear(&mut self, palette_index: u8) {
        self.pixels.fill(palette_index & 0x3F);
    }

    /// Get the raw pixel data as palette indices
    pub fn as_slice(&self) -> &[u8] {
        &self.pixels
    }

    /// Get mutable access to the raw pixel data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Copy pixel data from another frame buffer
    ///
    /// # Arguments
    /// * `other` - Source frame buffer to copy from
    pub fn copy_from(&mut self, other: &FrameBuffer) {
        self.pixels.copy_from_slice(&other.pixels);
    }

    /// Convert the frame buffer to RGBA format for display
    ///
    /// # Arguments
    /// * `output` - Output buffer to write RGBA data (must be at least SCREEN_SIZE * 4 bytes)
    ///
    /// # Panics
    /// Panics if output buffer is too small
    pub fn to_rgba(&self, output: &mut [u8]) {
        assert!(
            output.len() >= SCREEN_SIZE * 4,
            "Output buffer too small for RGBA conversion"
        );

        for (i, &palette_index) in self.pixels.iter().enumerate() {
            let rgba = palette_to_rgba(palette_index);
            let offset = i * 4;
            output[offset] = rgba[0]; // R
            output[offset + 1] = rgba[1]; // G
            output[offset + 2] = rgba[2]; // B
            output[offset + 3] = rgba[3]; // A
        }
    }

    /// Create a test pattern for debugging
    ///
    /// This generates a colorful test pattern showing all palette colors
    pub fn test_pattern(&mut self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                // Create a pattern that cycles through palette colors
                let palette_index = ((x / 16) + (y / 16) * 16) as u8 % 64;
                self.set_pixel(x, y, palette_index);
            }
        }
    }

    /// Create a gradient test pattern
    pub fn gradient_pattern(&mut self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                // Create horizontal gradient
                let palette_index = (x * 64 / SCREEN_WIDTH) as u8;
                self.set_pixel(x, y, palette_index);
            }
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = FrameBuffer::new();
        assert_eq!(fb.as_slice().len(), SCREEN_SIZE);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(100, 100, 0x20);
        assert_eq!(fb.get_pixel(100, 100), 0x20);
    }

    #[test]
    fn test_clear() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(0, 0, 0xFF);
        fb.clear(0x10);
        assert_eq!(fb.get_pixel(0, 0), 0x10);
        assert_eq!(fb.get_pixel(255, 239), 0x10);
    }

    #[test]
    fn test_to_rgba() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(0, 0, 0x00); // Should be dark gray

        let mut rgba = vec![0u8; SCREEN_SIZE * 4];
        fb.to_rgba(&mut rgba);

        // First pixel should be dark gray (0x666666)
        assert_eq!(rgba[0], 0x66); // R
        assert_eq!(rgba[1], 0x66); // G
        assert_eq!(rgba[2], 0x66); // B
        assert_eq!(rgba[3], 0xFF); // A
    }

    #[test]
    #[should_panic]
    fn test_set_pixel_out_of_bounds_x() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(256, 0, 0x00);
    }

    #[test]
    #[should_panic]
    fn test_set_pixel_out_of_bounds_y() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(0, 240, 0x00);
    }
}
