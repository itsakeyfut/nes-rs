// NES Instruction Test Suite
//
// Alternative comprehensive instruction test suite that validates
// CPU instruction behavior with different approach than instr_test_v5.
//
// This test suite provides additional coverage for edge cases and
// instruction implementation details.

mod common;

use common::{load_prg_rom, load_rom};
use nes_rs::bus::Bus;
use nes_rs::cpu::Cpu;
use std::path::Path;

/// Run a NES instruction test ROM and check the result
fn run_nes_instr_test(rom_path: &str) -> Result<(bool, String), String> {
    let path = Path::new(rom_path);
    if !path.exists() {
        return Err(format!("ROM file not found: {}", rom_path));
    }

    // Load ROM
    let prg_rom = load_rom(path)?;

    // Initialize CPU and Bus
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
// NES Instruction Tests - Individual Tests
// ============================================================================

#[test]
#[ignore] // Run with: cargo test nes_instr -- --ignored --nocapture
fn nes_instr_implied() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/01-implied.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nImplied Addressing:");
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
fn nes_instr_immediate() {
    let result =
        run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/02-immediate.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nImmediate Addressing:");
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
fn nes_instr_zero_page() {
    let result =
        run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/03-zero_page.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nZero Page Addressing:");
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
fn nes_instr_zp_xy() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/04-zp_xy.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nZero Page XY Addressing:");
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
fn nes_instr_absolute() {
    let result =
        run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/05-absolute.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAbsolute Addressing:");
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
fn nes_instr_abs_xy() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/06-abs_xy.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAbsolute XY Addressing:");
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
fn nes_instr_ind_x() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/07-ind_x.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nIndexed Indirect (X) Addressing:");
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
fn nes_instr_ind_y() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/08-ind_y.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nIndirect Indexed (Y) Addressing:");
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
fn nes_instr_branches() {
    let result =
        run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/09-branches.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nBranch Instructions:");
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
fn nes_instr_stack() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/10-stack.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nStack Operations:");
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
fn nes_instr_special() {
    let result = run_nes_instr_test("tests/nes-test-rom/nes_instr_test/rom_singles/11-special.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSpecial Instructions:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}
