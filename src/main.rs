// NES Emulator - Main Entry Point
use nes_rs::{Apu, Bus, Cartridge, Controller, Cpu, Ppu};

fn main() {
    println!("NES Emulator (nes-rs) v0.1.0");
    println!("==============================");

    // Initialize all components
    let cpu = Cpu::new();
    let _ppu = Ppu::new();
    let _apu = Apu::new();
    let _bus = Bus::new();
    let _cartridge = Cartridge::new();
    let _controller = Controller::new();

    println!("\nComponents initialized:");
    println!("  [✓] CPU (6502)");
    println!("  [✓] PPU (2C02)");
    println!("  [✓] APU");
    println!("  [✓] Memory Bus");
    println!("  [✓] Cartridge");
    println!("  [✓] Controller");

    println!("\nCPU State:");
    println!("  A: ${:02X}", cpu.a);
    println!("  X: ${:02X}", cpu.x);
    println!("  Y: ${:02X}", cpu.y);
    println!("  SP: ${:02X}", cpu.sp);
    println!("  PC: ${:04X}", cpu.pc);
    println!("  Status: ${:02X}", cpu.status);

    println!("\nProject structure setup complete!");
    println!("Ready for Phase 1: CPU Implementation");
}
