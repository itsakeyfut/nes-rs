// Display module - Handles window creation and frame rendering
//
// This module provides:
// - NES color palette (52 unique colors)
// - Frame buffer (256×240 pixels)
// - Window creation with scaling support (2x, 3x, 4x)
// - Frame rendering using winit + pixels
// - VSync and frame timing (60 FPS)

pub mod framebuffer;
pub mod integration;
pub mod palette;
pub mod window;

pub use framebuffer::{FrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};
pub use integration::copy_ppu_to_display;
pub use palette::{palette_to_rgb, palette_to_rgba, NES_PALETTE};
pub use window::{run_display, run_emulator, DisplayWindow, EmulatorDisplayWindow, WindowConfig};
