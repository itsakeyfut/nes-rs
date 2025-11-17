// NES Instruction Test Suite
//
// Alternative comprehensive instruction test suite that validates
// CPU instruction behavior with different approach than instr_test_v5.
//
// This test suite provides additional coverage for edge cases and
// instruction implementation details.

mod common;

use common::run_blargg_style_test;

/// Run a NES instruction test ROM and check the result
///
/// This is a thin wrapper around the common test runner that uses
/// the standard timeout value for instruction tests.
fn run_nes_instr_test(rom_path: &str) -> Result<(bool, String), String> {
    run_blargg_style_test(rom_path, 100_000_000)
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
