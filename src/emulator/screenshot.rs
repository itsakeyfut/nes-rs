// Screenshot functionality
//
// Captures the current frame buffer and saves it as a PNG file.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur during screenshot operations
#[derive(Debug)]
pub enum ScreenshotError {
    /// I/O error
    Io(io::Error),

    /// PNG encoding error
    PngEncoding(png::EncodingError),
}

impl std::fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenshotError::Io(e) => write!(f, "I/O error: {}", e),
            ScreenshotError::PngEncoding(e) => write!(f, "PNG encoding error: {}", e),
        }
    }
}

impl std::error::Error for ScreenshotError {}

impl From<io::Error> for ScreenshotError {
    fn from(e: io::Error) -> Self {
        ScreenshotError::Io(e)
    }
}

impl From<png::EncodingError> for ScreenshotError {
    fn from(e: png::EncodingError) -> Self {
        ScreenshotError::PngEncoding(e)
    }
}

/// Save a screenshot of the current frame
///
/// Converts the PPU frame buffer (palette indices) to RGB and saves as PNG.
///
/// # Arguments
///
/// * `frame_buffer` - The PPU frame buffer (256x240 palette indices)
/// * `rom_path` - Optional path to the currently loaded ROM (for naming)
///
/// # Returns
///
/// Result containing the path to the saved screenshot or an error
///
/// # Example
///
/// ```no_run
/// use nes_rs::emulator::save_screenshot;
/// use nes_rs::ppu::Ppu;
///
/// let ppu = Ppu::new();
/// let screenshot_path = save_screenshot(ppu.frame(), None).expect("Failed to save screenshot");
/// println!("Screenshot saved to: {}", screenshot_path.display());
/// ```
pub fn save_screenshot(
    frame_buffer: &[u8],
    rom_path: Option<&Path>,
) -> Result<PathBuf, ScreenshotError> {
    // Create screenshots directory
    let screenshots_dir = get_screenshot_directory(rom_path);
    fs::create_dir_all(&screenshots_dir)?;

    // Generate filename with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}.png", timestamp);
    let file_path = screenshots_dir.join(filename);

    // Convert palette indices to RGB
    let rgb_data = palette_indices_to_rgb(frame_buffer);

    // Save as PNG
    save_png(&file_path, &rgb_data, 256, 240)?;

    Ok(file_path)
}

/// Get the screenshot directory for the current ROM
///
/// Creates a directory structure like: screenshots/<rom_name>/
fn get_screenshot_directory(rom_path: Option<&Path>) -> PathBuf {
    let base_dir = PathBuf::from("screenshots");

    if let Some(rom_path) = rom_path {
        if let Some(rom_name) = rom_path.file_stem() {
            return base_dir.join(rom_name);
        }
    }

    base_dir.join("default")
}

/// Convert palette indices to RGB data
///
/// Takes the PPU frame buffer (palette indices 0-63) and converts to RGB888.
///
/// # Arguments
///
/// * `palette_indices` - Frame buffer with palette indices (256x240)
///
/// # Returns
///
/// RGB data (256x240x3 bytes)
fn palette_indices_to_rgb(palette_indices: &[u8]) -> Vec<u8> {
    use crate::display::NES_PALETTE;

    let mut rgb_data = Vec::with_capacity(palette_indices.len() * 3);

    for &index in palette_indices {
        let color = NES_PALETTE[index as usize % NES_PALETTE.len()];
        rgb_data.push(((color >> 16) & 0xFF) as u8); // R
        rgb_data.push(((color >> 8) & 0xFF) as u8); // G
        rgb_data.push((color & 0xFF) as u8); // B
    }

    rgb_data
}

/// Save RGB data as a PNG file
///
/// # Arguments
///
/// * `path` - Path to save the PNG file
/// * `data` - RGB data (width × height × 3 bytes)
/// * `width` - Image width
/// * `height` - Image height
///
/// # Returns
///
/// Result indicating success or error
fn save_png(path: &Path, data: &[u8], width: u32, height: u32) -> Result<(), ScreenshotError> {
    let file = fs::File::create(path)?;
    let w = io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_indices_to_rgb() {
        let indices = vec![0, 1, 2, 3];
        let rgb = palette_indices_to_rgb(&indices);

        // Should produce 12 bytes (4 pixels × 3 bytes)
        assert_eq!(rgb.len(), 12);

        // Each pixel should have 3 bytes (RGB)
        assert_eq!(rgb.len() % 3, 0);
    }

    #[test]
    fn test_get_screenshot_directory() {
        let dir = get_screenshot_directory(None);
        assert!(dir.ends_with("screenshots/default"));

        let rom_path = PathBuf::from("test/game.nes");
        let dir = get_screenshot_directory(Some(&rom_path));
        assert!(dir.ends_with("screenshots/game"));
    }
}
