// Blargg's CPU Test Suite
//
// These tests validate CPU instruction execution, timing, and edge cases
// using Kevin Horton's (Blargg) comprehensive CPU test ROMs.
//
// Test ROMs test various CPU behaviors:
// - Official instruction set
// - Timing accuracy
// - Edge cases and quirks
// - Interrupts
// - Reset behavior

mod common;

use common::{load_prg_rom, load_rom};
use nes_rs::bus::Bus;
use nes_rs::cpu::Cpu;
use std::path::Path;

/// Run a Blargg test ROM and check the result
///
/// Blargg's tests write ASCII result messages to $6004+
/// and set $6000 to a non-zero value when complete.
/// The test passes if the message starts with "Passed"
fn run_blargg_test(rom_path: &str) -> Result<(bool, String), String> {
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
// Blargg CPU Test 5 - Official Instruction Set
// ============================================================================

#[test]
#[ignore] // Run with: cargo test blargg_cpu_official -- --ignored --nocapture
fn blargg_cpu_official() {
    let result = run_blargg_test("tests/nes-test-rom/blargg_nes_cpu_test5/official.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// Instruction Test v5 - Individual Instruction Tests
// ============================================================================

#[test]
#[ignore]
fn instr_test_v5_all() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/all_instrs.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_basics() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/01-basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_implied() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/02-implied.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_immediate() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/03-immediate.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_zero_page() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/04-zero_page.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_zp_xy() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/05-zp_xy.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_absolute() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/06-absolute.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_abs_xy() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/07-abs_xy.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_ind_x() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/08-ind_x.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_ind_y() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/09-ind_y.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_branches() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/10-branches.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_stack() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/11-stack.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_jmp_jsr() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/12-jmp_jsr.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_rts() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/13-rts.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_rti() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/14-rti.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_brk() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/15-brk.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_test_v5_special() {
    let result = run_blargg_test("tests/nes-test-rom/instr_test-v5/rom_singles/16-special.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// CPU Timing Tests
// ============================================================================

#[test]
#[ignore]
fn cpu_timing_test() {
    let result = run_blargg_test("tests/nes-test-rom/cpu_timing_test6/cpu_timing_test.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// CPU Interrupts Test
// ============================================================================

#[test]
#[ignore]
fn cpu_interrupts_v2() {
    let tests = [
        "1-cli_latency.nes",
        "2-nmi_and_brk.nes",
        "3-nmi_and_irq.nes",
        "4-irq_and_dma.nes",
        "5-branch_delays_irq.nes",
    ];

    for test in &tests {
        let path = format!("tests/nes-test-rom/cpu_interrupts_v2/rom_singles/{}", test);
        println!("\nRunning: {}", test);

        let result = run_blargg_test(&path);

        match result {
            Ok((passed, message)) => {
                println!("{}", message);
                assert!(passed, "Test failed: {}", message);
            }
            Err(e) => {
                panic!("Test error in {}: {}", test, e);
            }
        }
    }
}

// ============================================================================
// CPU Reset Test
// ============================================================================

#[test]
#[ignore]
fn cpu_reset() {
    let result = run_blargg_test("tests/nes-test-rom/cpu_reset/registers.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// CPU Dummy Reads Test
// ============================================================================

#[test]
#[ignore]
fn cpu_dummy_reads() {
    let result = run_blargg_test("tests/nes-test-rom/cpu_dummy_reads/cpu_dummy_reads.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// CPU Exec Space Test
// ============================================================================

#[test]
#[ignore]
fn cpu_exec_space() {
    let result = run_blargg_test("tests/nes-test-rom/cpu_exec_space/test_cpu_exec_space_ppuio.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// Instruction Misc Tests
// ============================================================================

#[test]
#[ignore]
fn instr_misc_all() {
    let result = run_blargg_test("tests/nes-test-rom/instr_misc/instr_misc.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_misc_abs_x_wrap() {
    let result = run_blargg_test("tests/nes-test-rom/instr_misc/rom_singles/01-abs_x_wrap.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nAbsolute X Wrap:");
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
fn instr_misc_branch_wrap() {
    let result = run_blargg_test("tests/nes-test-rom/instr_misc/rom_singles/02-branch_wrap.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nBranch Wrap:");
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
fn instr_misc_dummy_reads() {
    let result = run_blargg_test("tests/nes-test-rom/instr_misc/rom_singles/03-dummy_reads.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nDummy Reads:");
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
fn instr_misc_dummy_reads_apu() {
    let result =
        run_blargg_test("tests/nes-test-rom/instr_misc/rom_singles/04-dummy_reads_apu.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nDummy Reads APU:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// Instruction Timing Tests
// ============================================================================

#[test]
#[ignore]
fn instr_timing_all() {
    let result = run_blargg_test("tests/nes-test-rom/instr_timing/instr_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\n{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn instr_timing_instr() {
    let result = run_blargg_test("tests/nes-test-rom/instr_timing/rom_singles/1-instr_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nInstruction Timing:");
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
fn instr_timing_branch() {
    let result = run_blargg_test("tests/nes-test-rom/instr_timing/rom_singles/2-branch_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nBranch Timing:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// Branch Timing Tests
// ============================================================================

#[test]
#[ignore]
fn branch_timing_basics() {
    let result = run_blargg_test("tests/nes-test-rom/branch_timing_tests/1.Branch_Basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nBranch Basics:");
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
fn branch_timing_backward() {
    let result = run_blargg_test("tests/nes-test-rom/branch_timing_tests/2.Backward_Branch.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nBackward Branch:");
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
fn branch_timing_forward() {
    let result = run_blargg_test("tests/nes-test-rom/branch_timing_tests/3.Forward_Branch.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nForward Branch:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}
