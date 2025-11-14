// Example: PPU and Display Integration
//
// This example demonstrates how to integrate the PPU rendering
// with the display system to show NES graphics on screen.

use nes_rs::display::{copy_ppu_to_display, run_display, FrameBuffer, WindowConfig};
use nes_rs::Ppu;

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

    // Create a test pattern for demonstration
    // (Since PPU memory is empty, let's show the test pattern instead)
    display_buffer.test_pattern();

    println!("Creating display window...");
    println!();

    // Configure window (3x scale, 60 FPS, VSync enabled)
    let config = WindowConfig::new().with_scale(3).with_fps(60).with_vsync(true);

    println!("Press the close button or Ctrl+C to exit.");
    println!();

    // In a real emulator, you would:
    // 1. Run emulation loop in a separate thread
    // 2. Update the display buffer each frame
    // 3. Synchronize with the display refresh rate
    //
    // For this example, we just display a static frame

    run_display(config)?;

    Ok(())
}
