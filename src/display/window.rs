// Window module - Manages display window and rendering
//
// This module provides window creation, scaling, and frame rendering
// using the winit and pixels crates.

use super::framebuffer::{FrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Window configuration
#[derive(Debug, Clone, Copy)]
pub struct WindowConfig {
    /// Scale factor (1x, 2x, 3x, 4x, etc.)
    pub scale: u32,
    /// Target frame rate in Hz (typically 60 for NTSC NES)
    pub target_fps: u32,
    /// Whether to enable VSync
    pub vsync: bool,
}

impl WindowConfig {
    /// Create a new window configuration with default values
    ///
    /// Default: 3x scale, 60 FPS, VSync enabled
    pub fn new() -> Self {
        Self {
            scale: 3,
            target_fps: 60,
            vsync: true,
        }
    }

    /// Set the scale factor
    pub fn with_scale(mut self, scale: u32) -> Self {
        self.scale = scale.clamp(1, 8); // Clamp between 1x and 8x
        self
    }

    /// Set the target frame rate
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    /// Set VSync enabled or disabled
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Get the window width in pixels
    pub fn window_width(&self) -> u32 {
        SCREEN_WIDTH as u32 * self.scale
    }

    /// Get the window height in pixels
    pub fn window_height(&self) -> u32 {
        SCREEN_HEIGHT as u32 * self.scale
    }

    /// Get the frame duration for the target FPS
    pub fn frame_duration(&self) -> Duration {
        Duration::from_micros(1_000_000 / self.target_fps as u64)
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Display window for rendering NES output
pub struct DisplayWindow {
    window: Option<Window>,
    pixels: Option<Box<Pixels<'static>>>,
    config: WindowConfig,
    frame_buffer: FrameBuffer,
    last_frame_time: Instant,
}

impl DisplayWindow {
    /// Create a new display window (window will be created when event loop starts)
    pub fn new(config: WindowConfig) -> Self {
        Self {
            window: None,
            pixels: None,
            config,
            frame_buffer: FrameBuffer::new(),
            last_frame_time: Instant::now(),
        }
    }

    /// Get a reference to the frame buffer
    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    /// Get a mutable reference to the frame buffer
    pub fn frame_buffer_mut(&mut self) -> &mut FrameBuffer {
        &mut self.frame_buffer
    }

    /// Render the current frame buffer to the window
    fn render(&mut self) -> Result<(), pixels::Error> {
        if let Some(pixels) = &mut self.pixels {
            // Get the pixel buffer
            let frame = pixels.frame_mut();

            // Convert frame buffer to RGBA
            self.frame_buffer.to_rgba(frame);

            // Render to screen
            pixels.render()?;
        }
        Ok(())
    }

    /// Check if enough time has passed for the next frame
    fn should_render_frame(&mut self) -> bool {
        let elapsed = self.last_frame_time.elapsed();
        let frame_duration = self.config.frame_duration();

        if elapsed >= frame_duration {
            self.last_frame_time = Instant::now();
            true
        } else {
            false
        }
    }
}

impl ApplicationHandler for DisplayWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title(format!(
                "NES Emulator - {}x{}",
                self.config.window_width(),
                self.config.window_height()
            ))
            .with_inner_size(LogicalSize::new(
                self.config.window_width(),
                self.config.window_height(),
            ))
            .with_resizable(false);

        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        // Store window first
        self.window = Some(window);

        // Now create pixel buffer using a reference to the stored window
        // SAFETY: We know the window exists and will outlive the pixels
        // because they're both stored in the same struct and pixels is dropped first.
        // We use unsafe to extend the lifetime to 'static, which is safe because
        // the window lives as long as self.
        let window_ref = self.window.as_ref().unwrap();
        let window_size = window_ref.inner_size();

        // Create a raw pointer and transmute to 'static lifetime
        // This is safe because:
        // 1. Window and Pixels are stored in the same struct
        // 2. Pixels is dropped before Window (field order)
        // 3. Neither is moved after creation
        let window_ptr = window_ref as *const Window;
        let window_static: &'static Window = unsafe { &*window_ptr };
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window_static);

        let pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .expect("Failed to create pixel buffer");

        self.pixels = Some(Box::new(pixels));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Render frame if enough time has passed
                if self.should_render_frame() {
                    if let Err(err) = self.render() {
                        eprintln!("Render error: {}", err);
                        event_loop.exit();
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request a redraw
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Create and run the display window
///
/// # Arguments
/// * `config` - Window configuration
///
/// # Returns
/// Result indicating success or error
pub fn run_display(config: WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Set control flow based on VSync setting
    if config.vsync {
        event_loop.set_control_flow(ControlFlow::Wait);
    } else {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    let mut display = DisplayWindow::new(config);

    // Create a test pattern for demonstration
    display.frame_buffer_mut().test_pattern();

    println!("Starting display window...");
    println!("  Resolution: {}x{}", SCREEN_WIDTH, SCREEN_HEIGHT);
    println!(
        "  Window size: {}x{}",
        config.window_width(),
        config.window_height()
    );
    println!("  Scale: {}x", config.scale);
    println!("  Target FPS: {}", config.target_fps);
    println!("  VSync: {}", config.vsync);

    event_loop.run_app(&mut display)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_defaults() {
        let config = WindowConfig::new();
        assert_eq!(config.scale, 3);
        assert_eq!(config.target_fps, 60);
        assert!(config.vsync);
    }

    #[test]
    fn test_window_config_builder() {
        let config = WindowConfig::new()
            .with_scale(2)
            .with_fps(30)
            .with_vsync(false);

        assert_eq!(config.scale, 2);
        assert_eq!(config.target_fps, 30);
        assert!(!config.vsync);
    }

    #[test]
    fn test_window_dimensions() {
        let config = WindowConfig::new().with_scale(2);
        assert_eq!(config.window_width(), 512);
        assert_eq!(config.window_height(), 480);
    }

    #[test]
    fn test_frame_duration() {
        let config = WindowConfig::new().with_fps(60);
        let duration = config.frame_duration();
        assert_eq!(duration.as_micros(), 16666); // ~16.67ms for 60 FPS
    }

    #[test]
    fn test_scale_clamping() {
        let config = WindowConfig::new().with_scale(100);
        assert_eq!(config.scale, 8); // Should be clamped to max 8x

        let config = WindowConfig::new().with_scale(0);
        assert_eq!(config.scale, 1); // Should be clamped to min 1x
    }
}
