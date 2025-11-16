// Common test utilities for ROM-based integration tests
//
// This module provides shared functionality for running and validating
// test ROMs across different test suites (CPU, PPU, APU, etc.)

#![allow(dead_code)]

use nes_rs::bus::Bus;
use nes_rs::cpu::Cpu;
use std::fs;
use std::path::Path;

/// Maximum number of frames to run a test ROM before timing out
pub const MAX_TEST_FRAMES: u32 = 600; // ~10 seconds at 60 FPS

/// Maximum number of CPU cycles to run before timing out
pub const MAX_TEST_CYCLES: u64 = 100_000_000; // 100 million cycles

/// Result of running a test ROM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed
    Passed,
    /// Test failed with an error code
    Failed(u8),
    /// Test timed out
    Timeout,
    /// Test result unknown (could not determine pass/fail)
    Unknown,
}

/// Test ROM runner configuration
pub struct TestConfig {
    /// Maximum number of cycles to run
    pub max_cycles: u64,
    /// Starting PC address (None = use reset vector)
    pub start_pc: Option<u16>,
    /// Starting cycle count
    pub start_cycles: u64,
    /// Enable trace logging
    pub trace: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            max_cycles: MAX_TEST_CYCLES,
            start_pc: None,
            start_cycles: 0,
            trace: false,
        }
    }
}

/// Load a ROM file and return the PRG-ROM data
///
/// # Arguments
///
/// * `path` - Path to the .nes ROM file
///
/// # Returns
///
/// The PRG-ROM data (excluding iNES header)
pub fn load_rom(path: &Path) -> Result<Vec<u8>, String> {
    let rom_data =
        fs::read(path).map_err(|e| format!("Failed to load ROM from {}: {}", path.display(), e))?;

    if rom_data.len() < 16 {
        return Err("ROM file too small (missing iNES header)".to_string());
    }

    // Parse iNES header
    let prg_rom_banks = rom_data[4] as usize;
    let prg_rom_size = prg_rom_banks * 16384;

    if rom_data.len() < 16 + prg_rom_size {
        return Err("ROM file too small for declared PRG-ROM size".to_string());
    }

    // Extract PRG-ROM (skip 16-byte header)
    Ok(rom_data[16..16 + prg_rom_size].to_vec())
}

/// Load PRG-ROM into the bus at $8000 and mirror at $C000
///
/// # Arguments
///
/// * `bus` - The bus to load the ROM into
/// * `prg_rom` - The PRG-ROM data
pub fn load_prg_rom(bus: &mut Bus, prg_rom: &[u8]) {
    // Load at $8000 and mirror at $C000
    for (i, &byte) in prg_rom.iter().enumerate() {
        let offset = i as u16;
        if offset < 0x4000 {
            // First 16KB at $8000
            bus.write(0x8000 + offset, byte);
            // Mirror at $C000
            bus.write(0xC000 + offset, byte);
        }
    }
}

/// Check if a test ROM has completed by examining result registers
///
/// Most test ROMs write result codes to specific memory locations:
/// - $6000: Test status (0 = running, 1+ = complete)
/// - $6001-$6003: Error code or result message
///
/// # Arguments
///
/// * `bus` - The bus to check
///
/// # Returns
///
/// TestResult indicating the test status
pub fn check_test_result(bus: &mut Bus) -> TestResult {
    // Check common test result locations
    // Location $6000: Test status (0 = running, non-zero = done)
    let status = bus.read(0x6000);

    if status == 0 {
        // Test still running
        return TestResult::Unknown;
    }

    // Location $6001: Result code (0 = passed, non-zero = error code)
    let result = bus.read(0x6001);

    if result == 0 {
        TestResult::Passed
    } else {
        TestResult::Failed(result)
    }
}

/// Read null-terminated string from memory
///
/// # Arguments
///
/// * `bus` - The bus to read from
/// * `addr` - Starting address
/// * `max_len` - Maximum length to read
///
/// # Returns
///
/// The string read from memory
pub fn read_string(bus: &mut Bus, addr: u16, max_len: usize) -> String {
    let mut result = String::new();
    let mut current_addr = addr;

    for _ in 0..max_len {
        let byte = bus.read(current_addr);
        if byte == 0 {
            break;
        }
        if (0x20..=0x7E).contains(&byte) {
            result.push(byte as char);
        }
        current_addr = current_addr.wrapping_add(1);
    }

    result
}

/// Run a test ROM and return the result
///
/// # Arguments
///
/// * `rom_path` - Path to the ROM file
/// * `config` - Test configuration
///
/// # Returns
///
/// Result containing the test result or error message
pub fn run_test_rom(rom_path: &Path, config: &TestConfig) -> Result<TestResult, String> {
    // Load ROM
    let prg_rom = load_rom(rom_path)?;

    // Initialize CPU and Bus
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Load PRG-ROM into bus
    load_prg_rom(&mut bus, &prg_rom);

    // Set PC
    if let Some(pc) = config.start_pc {
        cpu.pc = pc;
    } else {
        // Use reset vector
        let pc_low = bus.read(0xFFFC);
        let pc_high = bus.read(0xFFFD);
        cpu.pc = u16::from_le_bytes([pc_low, pc_high]);
    }

    cpu.cycles = config.start_cycles;

    // Run test
    let mut total_cycles = 0u64;

    while total_cycles < config.max_cycles {
        // Execute one instruction
        cpu.step(&mut bus);
        total_cycles = cpu.cycles;

        // Check for test completion
        let result = check_test_result(&mut bus);
        match result {
            TestResult::Passed | TestResult::Failed(_) => {
                return Ok(result);
            }
            TestResult::Unknown => {
                // Continue running
            }
            TestResult::Timeout => {
                return Ok(TestResult::Timeout);
            }
        }
    }

    // Timeout
    Ok(TestResult::Timeout)
}

/// Format test result for display
pub fn format_result(result: &TestResult) -> String {
    match result {
        TestResult::Passed => "✓ PASSED".to_string(),
        TestResult::Failed(code) => format!("✗ FAILED (error code: ${:02X})", code),
        TestResult::Timeout => "✗ TIMEOUT".to_string(),
        TestResult::Unknown => "? UNKNOWN".to_string(),
    }
}
