// NES Emulator - Main Entry Point
//
// This is a demonstration of the display system with a test pattern.
// Eventually, this will integrate with the full emulator (CPU, PPU, etc.)

use nes_rs::display::{run_display, WindowConfig};
use nes_rs::input::InputConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Emulator (nes-rs) v0.1.0");
    println!("==============================");
    println!();

    // Load or create input configuration
    let config_path = "input_config.toml";
    let input_config = InputConfig::load_or_default(config_path);
    println!("Input configuration loaded from '{}'", config_path);
    println!();

    println!("Display System Test");
    println!("-------------------");
    println!();

    // Create window configuration
    // Default: 3x scale, 60 FPS, VSync enabled
    let config = WindowConfig::new()
        .with_scale(3) // 768x720 window (256x240 * 3)
        .with_fps(60) // 60 FPS (NTSC)
        .with_vsync(true); // Enable VSync for smooth display

    // Run the display window with test pattern
    println!("Press the close button or Ctrl+C to exit.");
    println!();

    run_display(config)?;

    println!("Display window closed.");
    Ok(())
}
