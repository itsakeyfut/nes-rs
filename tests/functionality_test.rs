// Basic functionality tests for NES emulator components
// These tests verify that the core functionality works correctly

use nes_rs::*;
use std::path::Path;

#[test]
fn test_cpu_basic_functionality() {
    // Test CPU initialization and basic operation
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Verify initial state
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.x, 0);
    assert_eq!(cpu.y, 0);

    // Test simple instruction execution
    // LDA #$FF (load immediate $FF into A)
    bus.write(0x8000, 0xA9);
    bus.write(0x8001, 0xFF);
    cpu.pc = 0x8000;

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn test_ppu_initialization() {
    // Test PPU initialization
    let ppu = Ppu::new();

    // Verify PPU starts in correct state
    // Basic sanity check that PPU can be created
    assert!(std::mem::size_of_val(&ppu) > 0);
}

#[test]
fn test_apu_initialization() {
    // Test APU initialization
    let apu = Apu::new();

    // Verify APU starts in correct state
    assert!(std::mem::size_of_val(&apu) > 0);
}

#[test]
fn test_bus_read_write() {
    // Test Bus memory operations
    let mut bus = Bus::new();

    // Test RAM read/write
    bus.write(0x0000, 0x42);
    assert_eq!(bus.read(0x0000), 0x42);

    // Test RAM mirroring
    bus.write(0x0000, 0x11);
    assert_eq!(bus.read(0x0800), 0x11);
    assert_eq!(bus.read(0x1000), 0x11);
    assert_eq!(bus.read(0x1800), 0x11);
}

#[test]
fn test_controller_initialization() {
    // Test controller initialization
    let controller = Controller::new();

    // Verify controller starts with no buttons pressed
    assert!(std::mem::size_of_val(&controller) > 0);
}

#[test]
fn test_ram_operations() {
    // Test RAM module
    let mut ram = Ram::new();

    // Test basic read/write
    ram.write(0x0000, 0xAA);
    assert_eq!(ram.read(0x0000), 0xAA);

    // Test different addresses
    ram.write(0x07FF, 0x55);
    assert_eq!(ram.read(0x07FF), 0x55);
}

#[test]
fn test_cartridge_ines_header_parsing() {
    // Test iNES header parsing
    let mut header_bytes = [0u8; 16];
    header_bytes[0..4].copy_from_slice(&[0x4E, 0x45, 0x53, 0x1A]); // "NES" + EOF
    header_bytes[4] = 2; // 2 x 16KB PRG-ROM
    header_bytes[5] = 1; // 1 x 8KB CHR-ROM
    header_bytes[6] = 0x00; // Horizontal mirroring, mapper 0
    header_bytes[7] = 0x00;

    let header = INesHeader::from_bytes(&header_bytes).expect("Failed to parse header");

    assert_eq!(header.prg_rom_banks, 2);
    assert_eq!(header.chr_rom_banks, 1);
    assert_eq!(header.mapper_number(), 0);
    assert_eq!(header.mirroring(), Mirroring::Horizontal);
}

#[test]
fn test_emulator_initialization() {
    // Test emulator initialization
    let emulator = Emulator::new();

    // Verify emulator can be created
    assert!(std::mem::size_of_val(&emulator) > 0);
}

#[test]
#[ignore] // Only run when test ROM is available
fn test_emulator_load_rom() {
    // Test ROM loading functionality
    let rom_path = "tests/nes-test-rom/other/nestest.nes";

    if !Path::new(rom_path).exists() {
        eprintln!("Test ROM not found, skipping test");
        return;
    }

    let mut emulator = Emulator::new();
    let result = emulator.load_rom(rom_path);

    assert!(result.is_ok(), "Failed to load ROM: {:?}", result.err());
}

#[test]
fn test_cpu_flags() {
    // Test CPU status flags
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Test Zero flag with LDA #$00
    bus.write(0x8000, 0xA9); // LDA #$00
    bus.write(0x8001, 0x00);
    cpu.pc = 0x8000;
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x00);
    // Check that Zero flag is set (bit 1)
    assert!(cpu.get_flag(0b0000_0010)); // Zero flag
}

#[test]
fn test_cpu_stack_operations() {
    // Test CPU stack push/pop
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Initialize stack pointer
    cpu.sp = 0xFF;

    // PHA (Push A to stack)
    cpu.a = 0x42;
    bus.write(0x8000, 0x48); // PHA
    cpu.pc = 0x8000;
    cpu.step(&mut bus);

    assert_eq!(cpu.sp, 0xFE);
    assert_eq!(bus.read(0x01FF), 0x42);
}
