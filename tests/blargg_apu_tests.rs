// Blargg's APU Test Suite
//
// These tests validate APU (Audio Processing Unit) functionality including:
// - Length counter
// - Length table
// - IRQ flag
// - Frame IRQ timing
// - Clock jitter
// - Channel timing
//
// The APU is responsible for generating sound and music.

mod common;

use common::{load_prg_rom, load_rom};
use nes_rs::bus::Bus;
use nes_rs::cpu::Cpu;
use std::path::Path;

/// Run a Blargg APU test ROM and check the result
///
/// Blargg's APU tests write ASCII result messages to $6004+
/// and set $6000 to a non-zero value when complete.
fn run_blargg_apu_test(rom_path: &str) -> Result<(bool, String), String> {
    let path = Path::new(rom_path);
    if !path.exists() {
        return Err(format!("ROM file not found: {}", rom_path));
    }

    // Load ROM
    let prg_rom = load_rom(path)?;

    // Initialize CPU and Bus (which includes APU)
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Load PRG-ROM
    load_prg_rom(&mut bus, &prg_rom);

    // Use reset vector
    cpu.reset(&mut bus);

    // Run test with timeout (APU tests may take longer)
    let max_cycles = 200_000_000u64;

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
// Blargg APU Tests 2005.07.30
// ============================================================================

#[test]
#[ignore] // Run with: cargo test blargg_apu -- --ignored --nocapture
fn blargg_apu_len_ctr() {
    let result = run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/01.len_ctr.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n01. Length Counter Test:");
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
fn blargg_apu_len_table() {
    let result = run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/02.len_table.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n02. Length Table Test:");
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
fn blargg_apu_irq_flag() {
    let result = run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/03.irq_flag.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n03. IRQ Flag Test:");
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
fn blargg_apu_clock_jitter() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/04.clock_jitter.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n04. Clock Jitter Test:");
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
fn blargg_apu_len_timing_mode0() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/05.len_timing_mode0.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n05. Length Timing Mode 0 Test:");
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
fn blargg_apu_len_timing_mode1() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/06.len_timing_mode1.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n06. Length Timing Mode 1 Test:");
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
fn blargg_apu_irq_flag_timing() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/07.irq_flag_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n07. IRQ Flag Timing Test:");
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
fn blargg_apu_irq_timing() {
    let result = run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/08.irq_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n08. IRQ Timing Test:");
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
fn blargg_apu_reset_timing() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/09.reset_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n09. Reset Timing Test:");
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
fn blargg_apu_len_halt_timing() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/10.len_halt_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n10. Length Halt Timing Test:");
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
fn blargg_apu_len_reload_timing() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/blargg_apu_2005.07.30/11.len_reload_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n11. Length Reload Timing Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// APU Test (More comprehensive channel tests)
// ============================================================================

#[test]
#[ignore]
fn apu_test_1_len_ctr() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/1-len_ctr.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 1 - Length Counter:");
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
fn apu_test_2_len_table() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/2-len_table.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 2 - Length Table:");
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
fn apu_test_3_irq_flag() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/3-irq_flag.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 3 - IRQ Flag:");
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
fn apu_test_4_jitter() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/4-jitter.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 4 - Clock Jitter:");
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
fn apu_test_5_len_timing() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/5-len_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 5 - Length Timing:");
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
fn apu_test_6_irq_flag_timing() {
    let result =
        run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/6-irq_flag_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 6 - IRQ Flag Timing:");
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
fn apu_test_7_dmc_basics() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/7-dmc_basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 7 - DMC Basics:");
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
fn apu_test_8_dmc_rates() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_test/rom_singles/8-dmc_rates.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Test 8 - DMC Rates:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// APU Reset Test
// ============================================================================

#[test]
#[ignore]
fn apu_reset() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_reset/4015_cleared.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Reset Test:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// APU Mixer Tests
// ============================================================================

#[test]
#[ignore]
fn apu_mixer_square() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_mixer/square.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Mixer - Square Channel:");
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
fn apu_mixer_triangle() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_mixer/triangle.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Mixer - Triangle Channel:");
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
fn apu_mixer_noise() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_mixer/noise.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Mixer - Noise Channel:");
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
fn apu_mixer_dmc() {
    let result = run_blargg_apu_test("tests/nes-test-rom/apu_mixer/dmc.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAPU Mixer - DMC Channel:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}
