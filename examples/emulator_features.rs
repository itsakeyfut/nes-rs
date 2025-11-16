// Example: Emulator Features
//
// This example demonstrates all the quality-of-life features added to the emulator:
// - Save states (quick save/load and multiple slots)
// - Screenshots
// - Speed control (fast forward, slow motion, pause)
// - Reset functionality
// - Configuration management
// - Recent ROMs list

use nes_rs::emulator::{Emulator, SpeedMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Emulator - Feature Demonstration");
    println!("====================================");
    println!();

    // Create a new emulator instance
    let mut emulator = Emulator::new();
    println!("✓ Created emulator instance");

    // Load configuration
    let config = emulator.config();
    println!("✓ Loaded configuration:");
    println!("  - Video scale: {}x", config.video.scale);
    println!("  - VSync: {}", config.video.vsync);
    println!("  - Audio volume: {}", config.audio.volume);
    println!("  - Save slots: {}", config.save_state.slots);
    println!();

    // Demonstrate loading a ROM (if provided via command line)
    if let Some(rom_path) = std::env::args().nth(1) {
        println!("Loading ROM: {}", rom_path);
        match emulator.load_rom(&rom_path) {
            Ok(()) => println!("✓ ROM loaded successfully"),
            Err(e) => println!("✗ Failed to load ROM: {}", e),
        }
        println!();
    } else {
        println!("No ROM provided. Usage: cargo run --example emulator_features <rom_path>");
        println!("Continuing with demonstration of other features...");
        println!();
    }

    // Demonstrate reset functionality
    println!("Demonstration: Reset");
    println!("--------------------");
    emulator.reset();
    println!("✓ Emulator reset to initial state");
    println!();

    // Demonstrate speed control
    println!("Demonstration: Speed Control");
    println!("----------------------------");
    emulator.set_speed_mode(SpeedMode::Normal);
    println!("✓ Set speed mode to Normal (1x)");

    emulator.set_speed_mode(SpeedMode::FastForward2x);
    println!("✓ Set speed mode to Fast Forward 2x");

    emulator.set_speed_mode(SpeedMode::FastForward4x);
    println!("✓ Set speed mode to Fast Forward 4x");

    emulator.set_speed_mode(SpeedMode::SlowMotion);
    println!("✓ Set speed mode to Slow Motion (0.5x)");

    emulator.set_speed_mode(SpeedMode::Normal);
    println!("✓ Restored speed mode to Normal");
    println!();

    // Demonstrate pause/resume
    println!("Demonstration: Pause/Resume");
    println!("---------------------------");
    emulator.pause();
    println!("✓ Emulator paused (is_paused: {})", emulator.is_paused());

    emulator.resume();
    println!("✓ Emulator resumed (is_paused: {})", emulator.is_paused());

    emulator.toggle_pause();
    println!("✓ Toggled pause (is_paused: {})", emulator.is_paused());

    emulator.toggle_pause();
    println!(
        "✓ Toggled pause again (is_paused: {})",
        emulator.is_paused()
    );
    println!();

    // Demonstrate save states
    println!("Demonstration: Save States");
    println!("-------------------------");

    // Quick save (slot 0)
    match emulator.quick_save() {
        Ok(()) => println!("✓ Quick save successful (slot 0)"),
        Err(e) => println!("✗ Quick save failed: {}", e),
    }

    // Save to specific slot
    match emulator.save_state(1) {
        Ok(()) => println!("✓ Saved state to slot 1"),
        Err(e) => println!("✗ Save state failed: {}", e),
    }

    match emulator.save_state(2) {
        Ok(()) => println!("✓ Saved state to slot 2"),
        Err(e) => println!("✗ Save state failed: {}", e),
    }

    // Quick load (slot 0)
    match emulator.quick_load() {
        Ok(()) => println!("✓ Quick load successful (slot 0)"),
        Err(e) => println!("✗ Quick load failed: {}", e),
    }

    // Load from specific slot
    match emulator.load_state(1) {
        Ok(()) => println!("✓ Loaded state from slot 1"),
        Err(e) => println!("✗ Load state failed: {}", e),
    }
    println!();

    // Demonstrate screenshots
    println!("Demonstration: Screenshots");
    println!("-------------------------");
    match emulator.screenshot() {
        Ok(path) => println!("✓ Screenshot saved to: {}", path.display()),
        Err(e) => println!("✗ Screenshot failed: {}", e),
    }
    println!();

    // Demonstrate configuration save
    println!("Demonstration: Configuration");
    println!("---------------------------");
    let config_mut = emulator.config_mut();
    config_mut.video.scale = 4;
    config_mut.audio.volume = 0.75;

    match config_mut.save() {
        Ok(()) => println!("✓ Configuration saved"),
        Err(e) => println!("✗ Configuration save failed: {}", e),
    }
    println!();

    // Demonstrate recent ROMs list
    println!("Demonstration: Recent ROMs List");
    println!("------------------------------");
    use nes_rs::emulator::RecentRomsList;
    let recent_roms = RecentRomsList::load_or_default();

    if recent_roms.is_empty() {
        println!("Recent ROMs list is empty");
    } else {
        println!("Recent ROMs ({} total):", recent_roms.len());
        for (i, entry) in recent_roms.entries().iter().enumerate() {
            println!(
                "  {}. {} ({})",
                i + 1,
                entry.display_name,
                entry.last_accessed
            );
        }
    }
    println!();

    // Summary
    println!("Summary");
    println!("=======");
    println!("All emulator features demonstrated successfully!");
    println!();
    println!("Available hotkeys (default configuration):");
    println!("  F5  - Quick save");
    println!("  F7  - Quick load");
    println!("  F8  - Reset");
    println!("  F9  - Screenshot");
    println!("  Tab - Fast forward");
    println!("  P   - Pause/Resume");
    println!();
    println!("Save states are stored in: ./saves/<rom_name>/slot_*.state");
    println!("Screenshots are stored in: ./screenshots/<rom_name>/screenshot_*.png");
    println!("Configuration is stored in: ./emulator_config.toml");
    println!("Recent ROMs list is stored in: ./recent_roms.toml");

    Ok(())
}
