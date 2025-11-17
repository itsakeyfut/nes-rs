// NES Emulator - Game Runner
//
// This example demonstrates running a NES ROM.

use nes_rs::emulator::Emulator;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Emulator (nes-rs) v0.1.0");
    println!("==============================");
    println!();

    // Get ROM path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_path>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} game.nes", args[0]);
        eprintln!("  {} \"assets/JPA/Dragon Quest.nes\"", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];

    // Create and initialize emulator
    println!("Initializing emulator...");
    let mut emulator = Emulator::new();

    // Load ROM
    println!("Loading ROM: {}", rom_path);
    match emulator.load_rom(rom_path) {
        Ok(()) => println!("✓ ROM loaded successfully"),
        Err(e) => {
            eprintln!("✗ Failed to load ROM: {}", e);
            std::process::exit(1);
        }
    }
    println!();

    println!("Emulator initialized successfully!");
    println!();
    println!("Note: Full display integration is in progress.");
    println!("The emulator can load ROMs and execute CPU instructions.");
    println!("To test CPU functionality, use the nestest integration test:");
    println!("  cargo test nestest_cpu_test -- --ignored --nocapture");

    Ok(())
}
