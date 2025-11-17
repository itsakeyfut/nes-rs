// Window module - Manages display window and rendering
//
// This module provides window creation, scaling, and frame rendering
// using the winit and pixels crates.

use super::framebuffer::{FrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::integration::copy_ppu_to_display;
use crate::emulator::Emulator;
use crate::input::{ControllerIO, InputConfig, Player, UnifiedInputHandler};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
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
        self.target_fps = fps.max(1);
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
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    config: WindowConfig,
    frame_buffer: FrameBuffer,
    last_frame_time: Instant,
    input_handler: UnifiedInputHandler,
    controller_io: ControllerIO,
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
            input_handler: UnifiedInputHandler::new(),
            controller_io: ControllerIO::new(),
        }
    }

    /// Create a new display window with custom input configuration
    ///
    /// # Arguments
    /// * `config` - Window configuration
    /// * `input_config` - Input configuration for keyboard and gamepad mappings
    ///
    /// # Returns
    /// Result containing DisplayWindow or error message if input config is invalid
    pub fn with_input_config(
        config: WindowConfig,
        input_config: &InputConfig,
    ) -> Result<Self, String> {
        let input_handler = UnifiedInputHandler::with_config(input_config)?;

        Ok(Self {
            window: None,
            pixels: None,
            config,
            frame_buffer: FrameBuffer::new(),
            last_frame_time: Instant::now(),
            input_handler,
            controller_io: ControllerIO::new(),
        })
    }

    /// Get a reference to the frame buffer
    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    /// Get a mutable reference to the frame buffer
    pub fn frame_buffer_mut(&mut self) -> &mut FrameBuffer {
        &mut self.frame_buffer
    }

    /// Get a reference to the input handler
    pub fn input_handler(&self) -> &UnifiedInputHandler {
        &self.input_handler
    }

    /// Get a mutable reference to the input handler
    pub fn input_handler_mut(&mut self) -> &mut UnifiedInputHandler {
        &mut self.input_handler
    }

    /// Get a reference to the controller I/O
    pub fn controller_io(&self) -> &ControllerIO {
        &self.controller_io
    }

    /// Get a mutable reference to the controller I/O
    pub fn controller_io_mut(&mut self) -> &mut ControllerIO {
        &mut self.controller_io
    }

    /// Update controller states from current input state (keyboard + gamepad)
    fn update_controllers(&mut self) {
        // Update gamepad states first
        self.input_handler.update_gamepads();

        // Get merged controller states (keyboard + gamepad)
        let controller1 = self.input_handler.get_controller_state(Player::One);
        let controller2 = self.input_handler.get_controller_state(Player::Two);

        self.controller_io.set_controller1(controller1);
        self.controller_io.set_controller2(controller2);
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

        // Wrap window in Arc for shared ownership
        let window = Arc::new(window);
        let window_size = window.inner_size();

        // Create surface texture using Arc<Window> for safe 'static lifetime
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());

        let pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .expect("Failed to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        self.input_handler.handle_key_press(physical_key);
                    }
                    ElementState::Released => {
                        self.input_handler.handle_key_release(physical_key);
                    }
                }
                // Update controller states after keyboard input
                self.update_controllers();
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
        // Update controllers (including gamepad polling)
        self.update_controllers();

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
/// * `input_config` - Optional input configuration for custom mappings
///
/// # Returns
/// Result indicating success or error
pub fn run_display(
    config: WindowConfig,
    input_config: Option<&InputConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Set control flow based on VSync setting
    if config.vsync {
        event_loop.set_control_flow(ControlFlow::Wait);
    } else {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    let mut display = if let Some(input_cfg) = input_config {
        DisplayWindow::with_input_config(config, input_cfg)
            .map_err(|e| format!("Failed to apply input configuration: {}", e))?
    } else {
        DisplayWindow::new(config)
    };

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

/// Emulator display window - integrates emulator with display
pub struct EmulatorDisplayWindow {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    config: WindowConfig,
    frame_buffer: FrameBuffer,
    last_frame_time: Instant,
    input_handler: UnifiedInputHandler,
    controller_io: ControllerIO,
    emulator: Emulator,
}

impl EmulatorDisplayWindow {
    /// Create a new emulator display window
    pub fn new(config: WindowConfig, emulator: Emulator) -> Self {
        Self {
            window: None,
            pixels: None,
            config,
            frame_buffer: FrameBuffer::new(),
            last_frame_time: Instant::now(),
            input_handler: UnifiedInputHandler::new(),
            controller_io: ControllerIO::new(),
            emulator,
        }
    }

    /// Create a new emulator display window with custom input configuration
    pub fn with_input_config(
        config: WindowConfig,
        emulator: Emulator,
        input_config: &InputConfig,
    ) -> Result<Self, String> {
        let input_handler = UnifiedInputHandler::with_config(input_config)?;

        Ok(Self {
            window: None,
            pixels: None,
            config,
            frame_buffer: FrameBuffer::new(),
            last_frame_time: Instant::now(),
            input_handler,
            controller_io: ControllerIO::new(),
            emulator,
        })
    }

    /// Update controller states from current input state
    fn update_controllers(&mut self) {
        self.input_handler.update_gamepads();
        let controller1 = self.input_handler.get_controller_state(Player::One);
        let controller2 = self.input_handler.get_controller_state(Player::Two);
        self.controller_io.set_controller1(controller1);
        self.controller_io.set_controller2(controller2);
    }

    /// Execute one frame of emulation and render it
    fn execute_and_render(&mut self) -> Result<(), pixels::Error> {
        // Run one frame of emulation
        self.emulator.run_frame();

        // Copy PPU frame to display buffer
        let ppu_frame = self.emulator.bus().ppu().frame();
        copy_ppu_to_display(ppu_frame, &mut self.frame_buffer);

        // Render to screen
        if let Some(pixels) = &mut self.pixels {
            let frame = pixels.frame_mut();
            self.frame_buffer.to_rgba(frame);
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

impl ApplicationHandler for EmulatorDisplayWindow {
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

        let window = Arc::new(window);
        let window_size = window.inner_size();

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());

        let pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)
            .expect("Failed to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        self.input_handler.handle_key_press(physical_key);
                    }
                    ElementState::Released => {
                        self.input_handler.handle_key_release(physical_key);
                    }
                }
                self.update_controllers();
            }
            WindowEvent::RedrawRequested => {
                // Execute emulation and render frame
                if self.should_render_frame() {
                    if let Err(err) = self.execute_and_render() {
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
        self.update_controllers();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Run the emulator with display
///
/// # Arguments
/// * `config` - Window configuration
/// * `emulator` - Initialized emulator with ROM loaded
/// * `input_config` - Optional input configuration for custom mappings
///
/// # Returns
/// Result indicating success or error
pub fn run_emulator(
    config: WindowConfig,
    emulator: Emulator,
    input_config: Option<&InputConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    if config.vsync {
        event_loop.set_control_flow(ControlFlow::Wait);
    } else {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    let mut display = if let Some(input_cfg) = input_config {
        EmulatorDisplayWindow::with_input_config(config, emulator, input_cfg)
            .map_err(|e| format!("Failed to apply input configuration: {}", e))?
    } else {
        EmulatorDisplayWindow::new(config, emulator)
    };

    println!("Starting emulator...");
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
