// Example: PPU and Display Integration
//
// This example demonstrates how to integrate the PPU rendering
// with the display system to show NES graphics on screen.

use nes_rs::display::{copy_ppu_to_display, DisplayWindow, FrameBuffer, WindowConfig};
use nes_rs::Ppu;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Emulator - PPU Display Integration Example");
    println!("================================================");
    println!();

    // Create PPU instance
    let mut ppu = Ppu::new();

    // Configure PPU to render background and sprites
    // (In a real emulator, this would be done by CPU writes to PPU registers)
    // For now, we'll just render a frame with whatever is in PPU memory

    println!("Rendering PPU frame...");
    ppu.render_frame();

    // Copy PPU frame buffer to display buffer
    let mut display_buffer = FrameBuffer::new();
    copy_ppu_to_display(ppu.frame(), &mut display_buffer);

    // Since PPU memory is empty, create a test pattern for demonstration
    display_buffer.test_pattern();

    println!("Creating display window...");
    println!();

    // Configure window (3x scale, 60 FPS, VSync enabled)
    let config = WindowConfig::new()
        .with_scale(3)
        .with_fps(60)
        .with_vsync(true);

    println!("Press the close button or Ctrl+C to exit.");
    println!();

    // In a real emulator, you would:
    // 1. Run emulation loop in a separate thread
    // 2. Update the display buffer each frame
    // 3. Synchronize with the display refresh rate
    //
    // For this example, we display the PPU frame buffer (with test pattern)

    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Set control flow based on VSync setting
    if config.vsync {
        event_loop.set_control_flow(ControlFlow::Wait);
    } else {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    // Create display window
    let mut display = DisplayWindow::new(config);

    // Copy the PPU-derived buffer into the window's framebuffer
    display.frame_buffer_mut().copy_from(&display_buffer);

    // Run the event loop
    event_loop.run_app(&mut display)?;

    Ok(())
}
