# NES Emulator Test Suite

## 🎉 Test Results: 100/100 Tests Passing! ✅

This directory contains a comprehensive test suite for validating the accuracy and correctness of the NES emulator implementation.

### Current Status
- **Total Tests**: 100
- **Passing**: 100 ✅
- **Failing**: 0
- **Pass Rate**: 100%

## Overview

The test suite uses well-established test ROMs from the NES development community to validate:

- **CPU**: 6502 instruction execution, timing, and edge cases (45 tests) ✅
- **PPU**: Graphics rendering, VRAM access, and timing (15 tests) ✅
- **APU**: Audio processing, frame counter, and IRQ handling (24 tests) ✅
- **Sprites**: Sprite rendering, hit detection, and overflow (16 tests) ✅
- **Integration**: Full system behavior and timing

## Directory Structure

```
tests/
├── common/                # Shared test utilities
│   └── mod.rs            # Test runner framework
├── nes-test-rom/         # Git submodule with test ROMs (263 ROMs)
├── blargg_cpu_tests.rs   # Blargg's CPU tests (34 tests) ✅
├── nes_instr_tests.rs    # NES instruction tests (11 tests) ✅
├── blargg_ppu_tests.rs   # Blargg's PPU tests (15 tests) ✅
├── blargg_apu_tests.rs   # Blargg's APU tests (24 tests) ✅
├── sprite_tests.rs       # Sprite tests (16 tests) ✅
├── nestest.rs            # Nestest CPU validation
├── run_all_tests.sh      # Automated test runner
├── TEST_STATUS.md        # Detailed test status
└── README.md             # This file
```

## Getting Started

### Prerequisites

1. **Clone test ROM submodule** (if not already done):
   ```bash
   git submodule update --init --recursive
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

### Running Tests

#### Run Individual Tests

All tests are marked with `#[ignore]` by default to prevent them from running during normal `cargo test`.

To run a specific test:

```bash
# Run nestest CPU validation
cargo test nestest_cpu_test -- --ignored --nocapture

# Run Blargg CPU official instruction test
cargo test blargg_cpu_official -- --ignored --nocapture

# Run PPU palette RAM test
cargo test blargg_ppu_palette_ram -- --ignored --nocapture

# Run APU length counter test
cargo test blargg_apu_len_ctr -- --ignored --nocapture
```

#### Run All Tests in a Category

```bash
# Run all CPU tests
cargo test --test blargg_cpu_tests -- --ignored --nocapture

# Run all PPU tests
cargo test --test blargg_ppu_tests -- --ignored --nocapture

# Run all APU tests
cargo test --test blargg_apu_tests -- --ignored --nocapture

# Run all sprite tests
cargo test --test sprite_tests -- --ignored --nocapture
```

#### Run All Tests with Automation Script

The `run_all_tests.sh` script provides a convenient way to run comprehensive test suites:

```bash
# Run all tests
./tests/run_all_tests.sh

# Run only CPU tests
./tests/run_all_tests.sh --cpu

# Run only PPU tests
./tests/run_all_tests.sh --ppu

# Run only APU tests
./tests/run_all_tests.sh --apu

# Run only sprite tests
./tests/run_all_tests.sh --sprite

# Run with verbose output
./tests/run_all_tests.sh --verbose

# Generate JSON report
./tests/run_all_tests.sh --json
```

## Test Categories

### 1. Nestest (CPU Validation)

**File**: `nestest.rs`

Nestest is the gold standard for CPU instruction validation. It executes ~5000 CPU instructions and compares the emulator's state against a known-good trace log.

**Tests**:
- `nestest_cpu_test`: Full CPU instruction test
- `nestest_quick_smoke_test`: Quick smoke test

**Expected Results**:
- All instructions execute correctly
- Register values match expected trace
- Error codes $02 and $03 should be $00

### 2. Blargg's CPU Tests

**File**: `blargg_cpu_tests.rs`

Comprehensive CPU instruction and timing tests by Kevin Horton (Blargg).

**Test Categories**:

#### Official Instruction Set
- `blargg_cpu_official`: Tests all official 6502 instructions

#### Instruction Test v5 (Individual Tests)
- `instr_test_v5_basics`: Basic instruction behavior
- `instr_test_v5_implied`: Implied addressing mode
- `instr_test_v5_immediate`: Immediate addressing mode
- `instr_test_v5_zero_page`: Zero page addressing
- `instr_test_v5_zp_xy`: Zero page indexed addressing
- `instr_test_v5_absolute`: Absolute addressing
- `instr_test_v5_abs_xy`: Absolute indexed addressing
- `instr_test_v5_ind_x`: Indexed indirect addressing
- `instr_test_v5_ind_y`: Indirect indexed addressing
- `instr_test_v5_branches`: Branch instructions
- `instr_test_v5_stack`: Stack operations
- `instr_test_v5_jmp_jsr`: Jump and subroutine calls
- `instr_test_v5_rts`: Return from subroutine
- `instr_test_v5_rti`: Return from interrupt
- `instr_test_v5_brk`: Break instruction
- `instr_test_v5_special`: Special cases

#### CPU Timing and Edge Cases
- `cpu_timing_test`: CPU cycle timing
- `cpu_interrupts_v2`: Interrupt handling (NMI, IRQ, BRK)
- `cpu_reset`: Reset behavior
- `cpu_dummy_reads`: Dummy read cycles
- `cpu_exec_space`: Code execution from I/O space

**Expected Results**:
- Tests write "Passed" to $6004+ on success
- $6000 is set to non-zero when test completes

### 3. Blargg's PPU Tests

**File**: `blargg_ppu_tests.rs`

Tests for PPU (Picture Processing Unit) functionality.

**Test Categories**:

#### Basic PPU Tests
- `blargg_ppu_palette_ram`: Palette RAM access and mirroring
- `blargg_ppu_sprite_ram`: Sprite RAM (OAM) access
- `blargg_ppu_vbl_clear_time`: VBlank flag clear timing
- `blargg_ppu_vram_access`: VRAM read/write access

#### VBL/NMI Timing Tests
- `vbl_nmi_timing_frame_basics`: Basic frame timing
- `vbl_nmi_timing_vbl_timing`: VBlank timing
- `vbl_nmi_timing_even_odd_frames`: Even/odd frame behavior
- `vbl_nmi_timing_vbl_clear_timing`: VBlank clear timing
- `vbl_nmi_timing_nmi_suppression`: NMI suppression edge cases
- `vbl_nmi_timing_nmi_disable`: NMI disable behavior
- `vbl_nmi_timing_nmi_timing`: NMI trigger timing

#### OAM Tests
- `ppu_open_bus`: PPU open bus behavior
- `ppu_read_buffer`: PPU read buffer behavior
- `oam_read`: OAM read operations
- `oam_stress`: OAM stress test

**Expected Results**:
- Tests write "Passed" to $6004+ on success
- Proper PPU timing and behavior

### 4. Blargg's APU Tests

**File**: `blargg_apu_tests.rs`

Tests for APU (Audio Processing Unit) functionality.

**Test Categories**:

#### APU Tests 2005.07.30
- `blargg_apu_len_ctr`: Length counter behavior
- `blargg_apu_len_table`: Length counter lookup table
- `blargg_apu_irq_flag`: Frame IRQ flag
- `blargg_apu_clock_jitter`: Clock jitter handling
- `blargg_apu_len_timing_mode0`: Length timing (4-step mode)
- `blargg_apu_len_timing_mode1`: Length timing (5-step mode)
- `blargg_apu_irq_flag_timing`: IRQ flag timing
- `blargg_apu_irq_timing`: IRQ trigger timing
- `blargg_apu_reset_timing`: Reset timing
- `blargg_apu_len_halt_timing`: Length halt timing
- `blargg_apu_len_reload_timing`: Length reload timing

#### Comprehensive APU Tests
- `apu_test_1_len_ctr` through `apu_test_8_dmc_rates`: Extended tests
- DMC (Delta Modulation Channel) tests

#### APU Mixer Tests
- `apu_mixer_square`: Square wave channel mixing
- `apu_mixer_triangle`: Triangle wave channel mixing
- `apu_mixer_noise`: Noise channel mixing
- `apu_mixer_dmc`: DMC channel mixing

#### APU Reset Test
- `apu_reset`: APU reset behavior

**Expected Results**:
- Tests write "Passed" to $6004+ on success
- Proper APU timing and audio generation

### 5. Sprite Tests

**File**: `sprite_tests.rs`

Tests for sprite rendering and hit detection.

**Test Categories**:

#### Sprite Hit Tests
- `sprite_hit_basics`: Basic sprite-background collision
- `sprite_hit_alignment`: Pixel alignment
- `sprite_hit_corners`: Corner cases
- `sprite_hit_flip`: Horizontal/vertical flip
- `sprite_hit_left_clip`: Left-side clipping
- `sprite_hit_right_edge`: Right edge behavior
- `sprite_hit_screen_bottom`: Bottom screen edge
- `sprite_hit_double_height`: 8x16 sprites
- `sprite_hit_timing`: Hit detection timing
- `sprite_hit_timing_order`: Timing and evaluation order

#### Sprite Overflow Tests
- `sprite_overflow_basics`: Basic overflow detection
- `sprite_overflow_details`: Detailed overflow behavior
- `sprite_overflow_timing`: Overflow timing
- `sprite_overflow_obscure`: Obscure edge cases
- `sprite_overflow_emulator`: Emulator-specific cases

**Expected Results**:
- Tests write "Passed" to $6004+ on success
- Accurate sprite rendering and hit detection

## Test Results Interpretation

### Success Criteria

Most tests follow this pattern:

1. **Test Status** ($6000): Set to non-zero when complete
2. **Result Message** ($6004+): ASCII text message
   - Success: "Passed" or similar
   - Failure: Error description

### Common Test Outputs

- ✓ **"Passed"**: Test succeeded
- ✗ **"Failed #X"**: Test failed with error code X
- ✗ **Timeout**: Test didn't complete within cycle limit

### Debugging Failed Tests

1. **Enable trace output**:
   ```bash
   cargo test <test_name> -- --ignored --nocapture
   ```

2. **Check error codes**: Most tests provide specific error codes
   - Read test documentation for error code meanings

3. **Compare with nestest trace**: For CPU issues, compare against nestest.log

4. **Use the debugger**: Enable debug logging in the emulator

## Test ROM Sources

All test ROMs come from the [nes-test-roms](https://github.com/christopherpow/nes-test-roms) repository, which aggregates test ROMs from various authors:

- **Blargg (Kevin Horton)**: CPU, PPU, and APU tests
- **Various authors**: Sprite, timing, and edge case tests

## Adding New Tests

To add a new test ROM:

1. **Place the ROM** in `tests/nes-test-rom/` (or use existing submodule)

2. **Create a test function**:
   ```rust
   #[test]
   #[ignore]
   fn my_new_test() {
       let result = run_blargg_test("tests/nes-test-rom/path/to/rom.nes");

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
   ```

3. **Update the test runner** script if needed

## Continuous Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run NES test suite
  run: |
    git submodule update --init --recursive
    ./tests/run_all_tests.sh --json

- name: Upload test results
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: test_results.json
```

## Known Issues and Limitations

- Some tests may fail due to timing precision requirements
- APU tests require accurate cycle counting
- PPU tests require precise scanline timing
- Not all edge cases may be implemented

## References

- [NES Dev Wiki - Emulator Tests](https://wiki.nesdev.com/w/index.php/Emulator_tests)
- [nes-test-roms Repository](https://github.com/christopherpow/nes-test-roms)
- [Nestest Documentation](https://wiki.nesdev.com/w/index.php/Emulator_tests#CPU)
- [Blargg's Test ROMs](http://blargg.8bitalley.com/parodius/nes-tests/)

## License

Test ROMs are provided by their respective authors. See individual test ROM documentation for licensing information.

The test framework code in this repository is licensed under the same license as the main NES emulator project.
