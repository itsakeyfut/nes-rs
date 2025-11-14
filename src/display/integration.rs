// Integration helpers for connecting PPU with display system
//
// This module provides utilities to convert PPU frame buffer data
// into the display system's frame buffer format.

use super::framebuffer::FrameBuffer;

/// Copy PPU frame buffer data into a display frame buffer
///
/// The PPU frame buffer contains palette indices (0-63) for each pixel.
/// This function copies those indices directly into the display frame buffer.
///
/// # Arguments
///
/// * `ppu_frame` - Slice of PPU frame data (palette indices, 256×240 = 61,440 bytes)
/// * `display_buffer` - Mutable reference to the display frame buffer
///
/// # Example
///
/// ```rust,no_run
/// use nes_rs::{Ppu, FrameBuffer};
/// use nes_rs::display::integration::copy_ppu_to_display;
///
/// let mut ppu = Ppu::new();
/// let mut display_buffer = FrameBuffer::new();
///
/// // Render a frame with the PPU
/// ppu.render_frame();
///
/// // Copy PPU output to display buffer
/// copy_ppu_to_display(ppu.frame(), &mut display_buffer);
/// ```
pub fn copy_ppu_to_display(ppu_frame: &[u8], display_buffer: &mut FrameBuffer) {
    // Verify that the PPU frame has the correct size
    const EXPECTED_SIZE: usize = 256 * 240;
    assert_eq!(
        ppu_frame.len(),
        EXPECTED_SIZE,
        "PPU frame buffer must be exactly 256×240 pixels"
    );

    // Copy palette indices directly
    display_buffer.as_mut_slice().copy_from_slice(ppu_frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_ppu_to_display() {
        let ppu_frame = vec![0x0F; 256 * 240]; // Black frame
        let mut display_buffer = FrameBuffer::new();

        copy_ppu_to_display(&ppu_frame, &mut display_buffer);

        // Verify all pixels were copied
        assert_eq!(display_buffer.get_pixel(0, 0), 0x0F);
        assert_eq!(display_buffer.get_pixel(255, 239), 0x0F);
    }

    #[test]
    #[should_panic(expected = "PPU frame buffer must be exactly 256×240 pixels")]
    fn test_copy_invalid_size() {
        let ppu_frame = vec![0x0F; 100]; // Wrong size
        let mut display_buffer = FrameBuffer::new();

        copy_ppu_to_display(&ppu_frame, &mut display_buffer);
    }
}
