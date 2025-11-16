// Blargg's PPU Test Suite
//
// These tests validate PPU (Picture Processing Unit) functionality including:
// - Palette RAM access
// - Sprite RAM (OAM) access
// - VBlank clear timing
// - VRAM access timing
//
// The PPU is responsible for rendering graphics and sprites.

mod common;

use common::{load_prg_rom, load_rom};
use nes_rs::bus::Bus;
use nes_rs::cpu::Cpu;
use std::path::Path;

/// Run a Blargg PPU test ROM and check the result
///
/// Blargg's PPU tests write ASCII result messages to $6004+
/// and set $6000 to a non-zero value when complete.
fn run_blargg_ppu_test(rom_path: &str) -> Result<(bool, String), String> {
    let path = Path::new(rom_path);
    if !path.exists() {
        return Err(format!("ROM file not found: {}", rom_path));
    }

    // Load ROM
    let prg_rom = load_rom(path)?;

    // Initialize CPU and Bus (which includes PPU)
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Load PRG-ROM
    load_prg_rom(&mut bus, &prg_rom);

    // Use reset vector
    cpu.reset(&mut bus);

    // Run test with timeout
    let max_cycles = 100_000_000u64;

    while cpu.cycles < max_cycles {
        cpu.step(&mut bus);

        // Check test status ($6000)
        // $80 = running, $81 = need reset, $00-$7F = completed with result code
        let status = bus.read(0x6000);

        // Test is complete when status is $00-$7F
        if status < 0x80 {
            // Read result message from $6004
            let mut message = String::new();
            for i in 0..256 {
                let byte = bus.read(0x6004 + i);
                if byte == 0 {
                    break;
                }
                if (0x20..=0x7E).contains(&byte) {
                    message.push(byte as char);
                }
            }

            // Status code 0 means passed, non-zero means failed
            let passed = status == 0 || message.starts_with("Passed");

            // If message is empty, use status code
            if message.is_empty() {
                message = if status == 0 {
                    "Passed".to_string()
                } else {
                    format!("Failed with status code: {}", status)
                };
            }

            return Ok((passed, message));
        }
    }

    Err("Test timed out".to_string())
}

// ============================================================================
// Blargg PPU Tests 2005.09.15b
// ============================================================================

#[test]
#[ignore] // Run with: cargo test blargg_ppu -- --ignored --nocapture
fn blargg_ppu_palette_ram() {
    let result =
        run_blargg_ppu_test("tests/nes-test-rom/blargg_ppu_tests_2005.09.15b/palette_ram.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nPalette RAM Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn blargg_ppu_sprite_ram() {
    let result =
        run_blargg_ppu_test("tests/nes-test-rom/blargg_ppu_tests_2005.09.15b/sprite_ram.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite RAM Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn blargg_ppu_vbl_clear_time() {
    let result =
        run_blargg_ppu_test("tests/nes-test-rom/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nVBL Clear Time Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn blargg_ppu_vram_access() {
    let result =
        run_blargg_ppu_test("tests/nes-test-rom/blargg_ppu_tests_2005.09.15b/vram_access.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nVRAM Access Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// VBL NMI Timing Tests
// ============================================================================

#[test]
#[ignore]
fn vbl_nmi_timing_frame_basics() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/1.frame_basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nFrame Basics Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_vbl_timing() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/2.vbl_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nVBL Timing Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_even_odd_frames() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/3.even_odd_frames.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nEven/Odd Frames Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_vbl_clear_timing() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/4.vbl_clear_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nVBL Clear Timing Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_nmi_suppression() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/5.nmi_suppression.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nNMI Suppression Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_nmi_disable() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/6.nmi_disable.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nNMI Disable Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn vbl_nmi_timing_nmi_timing() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/vbl_nmi_timing/7.nmi_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nNMI Timing Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// PPU Open Bus Test
// ============================================================================

#[test]
#[ignore]
fn ppu_open_bus() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/ppu_open_bus/ppu_open_bus.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nPPU Open Bus Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// PPU Read Buffer Test
// ============================================================================

#[test]
#[ignore]
fn ppu_read_buffer() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/ppu_read_buffer/test_ppu_read_buffer.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nPPU Read Buffer Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// OAM Tests
// ============================================================================

#[test]
#[ignore]
fn oam_read() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/oam_read/oam_read.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nOAM Read Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn oam_stress() {
    let result = run_blargg_ppu_test("tests/nes-test-rom/oam_stress/oam_stress.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nOAM Stress Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}
