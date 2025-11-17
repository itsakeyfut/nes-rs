#!/bin/bash
# NES Emulator Comprehensive Test Suite Runner
#
# This script runs all test ROMs and generates a comprehensive report
# of test results for validation of emulator accuracy.
#
# Usage:
#   ./tests/run_all_tests.sh [OPTIONS]
#
# Options:
#   --cpu         Run only CPU tests
#   --ppu         Run only PPU tests
#   --apu         Run only APU tests
#   --sprite      Run only sprite tests
#   --nestest     Run only nestest
#   --verbose     Show detailed test output
#   --json        Output results in JSON format
#   --help        Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test categories
RUN_CPU=false
RUN_PPU=false
RUN_APU=false
RUN_SPRITE=false
RUN_NESTEST=false
RUN_ALL=true
VERBOSE=false
JSON_OUTPUT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cpu)
            RUN_CPU=true
            RUN_ALL=false
            shift
            ;;
        --ppu)
            RUN_PPU=true
            RUN_ALL=false
            shift
            ;;
        --apu)
            RUN_APU=true
            RUN_ALL=false
            shift
            ;;
        --sprite)
            RUN_SPRITE=true
            RUN_ALL=false
            shift
            ;;
        --nestest)
            RUN_NESTEST=true
            RUN_ALL=false
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --help)
            echo "NES Emulator Test Suite Runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --cpu         Run only CPU tests"
            echo "  --ppu         Run only PPU tests"
            echo "  --apu         Run only APU tests"
            echo "  --sprite      Run only sprite tests"
            echo "  --nestest     Run only nestest"
            echo "  --verbose     Show detailed test output"
            echo "  --json        Output results in JSON format"
            echo "  --help        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# If RUN_ALL is true, enable all test categories
if [ "$RUN_ALL" = true ]; then
    RUN_CPU=true
    RUN_PPU=true
    RUN_APU=true
    RUN_SPRITE=true
    RUN_NESTEST=true
fi

# Results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
RESULTS_FILE="test_results.txt"
JSON_FILE="test_results.json"

# Clear previous results
: > "$RESULTS_FILE"

if [ "$JSON_OUTPUT" = true ]; then
    echo "{" > "$JSON_FILE"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$JSON_FILE"
    echo "  \"tests\": [" >> "$JSON_FILE"
fi

# Function to run a test and record result
run_test() {
    local test_name=$1
    local test_filter=$2

    echo -e "${BLUE}Running: $test_name${NC}"

    if [ "$VERBOSE" = true ]; then
        cargo test "$test_filter" -- --ignored --nocapture 2>&1 | tee -a "$RESULTS_FILE"
        TEST_RESULT=${PIPESTATUS[0]}
    else
        cargo test "$test_filter" -- --ignored --nocapture > temp_test_output.txt 2>&1
        TEST_RESULT=$?
    fi

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ $TEST_RESULT -eq 0 ]; then
        echo -e "${GREEN}✓ PASSED${NC}: $test_name"
        echo "PASSED: $test_name" >> "$RESULTS_FILE"
        PASSED_TESTS=$((PASSED_TESTS + 1))

        if [ "$JSON_OUTPUT" = true ]; then
            echo "    {\"name\": \"$test_name\", \"status\": \"passed\"}," >> "$JSON_FILE"
        fi
    else
        echo -e "${RED}✗ FAILED${NC}: $test_name"
        echo "FAILED: $test_name" >> "$RESULTS_FILE"
        FAILED_TESTS=$((FAILED_TESTS + 1))

        if [ "$VERBOSE" = false ] && [ -f temp_test_output.txt ]; then
            echo "Error output:" >> "$RESULTS_FILE"
            cat temp_test_output.txt >> "$RESULTS_FILE"
            echo "" >> "$RESULTS_FILE"
        fi

        if [ "$JSON_OUTPUT" = true ]; then
            echo "    {\"name\": \"$test_name\", \"status\": \"failed\"}," >> "$JSON_FILE"
        fi
    fi

    rm -f temp_test_output.txt
}

# Print header
echo ""
echo "========================================="
echo "  NES Emulator Comprehensive Test Suite"
echo "========================================="
echo ""

# Check if test ROM submodule is initialized
if [ ! -d "tests/nes-test-rom" ] || [ -z "$(ls -A tests/nes-test-rom)" ]; then
    echo -e "${YELLOW}Warning: Test ROM submodule not initialized${NC}"
    echo "Run: git submodule update --init --recursive"
    echo ""
fi

# Run Nestest
if [ "$RUN_NESTEST" = true ]; then
    echo ""
    echo "=== Nestest (CPU Instruction Validation) ==="
    run_test "Nestest" "nestest_cpu_test"
fi

# Run CPU tests
if [ "$RUN_CPU" = true ]; then
    echo ""
    echo "=== CPU Tests (Blargg) ==="
    run_test "CPU Official Instructions" "blargg_cpu_official"
    run_test "CPU Instructions v5 - All" "instr_test_v5_all"
    run_test "CPU Instructions v5 - Basics" "instr_test_v5_basics"
    run_test "CPU Instructions v5 - Implied" "instr_test_v5_implied"
    run_test "CPU Instructions v5 - Immediate" "instr_test_v5_immediate"
    run_test "CPU Instructions v5 - Zero Page" "instr_test_v5_zero_page"
    run_test "CPU Instructions v5 - ZP XY" "instr_test_v5_zp_xy"
    run_test "CPU Instructions v5 - Absolute" "instr_test_v5_absolute"
    run_test "CPU Instructions v5 - Abs XY" "instr_test_v5_abs_xy"
    run_test "CPU Instructions v5 - Ind X" "instr_test_v5_ind_x"
    run_test "CPU Instructions v5 - Ind Y" "instr_test_v5_ind_y"
    run_test "CPU Instructions v5 - Branches" "instr_test_v5_branches"
    run_test "CPU Instructions v5 - Stack" "instr_test_v5_stack"
    run_test "CPU Instructions v5 - JMP/JSR" "instr_test_v5_jmp_jsr"
    run_test "CPU Instructions v5 - RTS" "instr_test_v5_rts"
    run_test "CPU Instructions v5 - RTI" "instr_test_v5_rti"
    run_test "CPU Instructions v5 - BRK" "instr_test_v5_brk"
    run_test "CPU Instructions v5 - Special" "instr_test_v5_special"

    echo ""
    echo "=== Instruction Misc Tests ==="
    run_test "Instr Misc - All" "instr_misc_all"
    run_test "Instr Misc - Abs X Wrap" "instr_misc_abs_x_wrap"
    run_test "Instr Misc - Branch Wrap" "instr_misc_branch_wrap"
    run_test "Instr Misc - Dummy Reads" "instr_misc_dummy_reads"
    run_test "Instr Misc - Dummy Reads APU" "instr_misc_dummy_reads_apu"

    echo ""
    echo "=== Instruction Timing Tests ==="
    run_test "Instr Timing - All" "instr_timing_all"
    run_test "Instr Timing - Instructions" "instr_timing_instr"
    run_test "Instr Timing - Branch" "instr_timing_branch"

    echo ""
    echo "=== Branch Timing Tests ==="
    run_test "Branch Timing - Basics" "branch_timing_basics"
    run_test "Branch Timing - Backward" "branch_timing_backward"
    run_test "Branch Timing - Forward" "branch_timing_forward"

    echo ""
    echo "=== CPU Tests (Other) ==="
    run_test "CPU Timing" "cpu_timing_test"
    run_test "CPU Interrupts" "cpu_interrupts_v2"
    run_test "CPU Reset" "cpu_reset"
    run_test "CPU Dummy Reads" "cpu_dummy_reads"
    run_test "CPU Exec Space" "cpu_exec_space"

    echo ""
    echo "=== NES Instruction Tests ==="
    run_test "NES Instr - Implied" "nes_instr_implied"
    run_test "NES Instr - Immediate" "nes_instr_immediate"
    run_test "NES Instr - Zero Page" "nes_instr_zero_page"
    run_test "NES Instr - ZP XY" "nes_instr_zp_xy"
    run_test "NES Instr - Absolute" "nes_instr_absolute"
    run_test "NES Instr - Abs XY" "nes_instr_abs_xy"
    run_test "NES Instr - Ind X" "nes_instr_ind_x"
    run_test "NES Instr - Ind Y" "nes_instr_ind_y"
    run_test "NES Instr - Branches" "nes_instr_branches"
    run_test "NES Instr - Stack" "nes_instr_stack"
    run_test "NES Instr - Special" "nes_instr_special"
fi

# Run PPU tests
if [ "$RUN_PPU" = true ]; then
    echo ""
    echo "=== PPU Tests (Blargg) ==="
    run_test "PPU Palette RAM" "blargg_ppu_palette_ram"
    run_test "PPU Sprite RAM" "blargg_ppu_sprite_ram"
    run_test "PPU VBL Clear Time" "blargg_ppu_vbl_clear_time"
    run_test "PPU VRAM Access" "blargg_ppu_vram_access"

    echo ""
    echo "=== VBL/NMI Timing Tests ==="
    run_test "VBL Frame Basics" "vbl_nmi_timing_frame_basics"
    run_test "VBL Timing" "vbl_nmi_timing_vbl_timing"
    run_test "VBL Even/Odd Frames" "vbl_nmi_timing_even_odd_frames"
    run_test "VBL Clear Timing" "vbl_nmi_timing_vbl_clear_timing"
    run_test "NMI Suppression" "vbl_nmi_timing_nmi_suppression"
    run_test "NMI Disable" "vbl_nmi_timing_nmi_disable"
    run_test "NMI Timing" "vbl_nmi_timing_nmi_timing"

    echo ""
    echo "=== OAM Tests ==="
    run_test "PPU Open Bus" "ppu_open_bus"
    run_test "PPU Read Buffer" "ppu_read_buffer"
    run_test "OAM Read" "oam_read"
    run_test "OAM Stress" "oam_stress"
fi

# Run APU tests
if [ "$RUN_APU" = true ]; then
    echo ""
    echo "=== APU Tests (Blargg 2005.07.30) ==="
    run_test "APU Length Counter" "blargg_apu_len_ctr"
    run_test "APU Length Table" "blargg_apu_len_table"
    run_test "APU IRQ Flag" "blargg_apu_irq_flag"
    run_test "APU Clock Jitter" "blargg_apu_clock_jitter"
    run_test "APU Length Timing Mode 0" "blargg_apu_len_timing_mode0"
    run_test "APU Length Timing Mode 1" "blargg_apu_len_timing_mode1"
    run_test "APU IRQ Flag Timing" "blargg_apu_irq_flag_timing"
    run_test "APU IRQ Timing" "blargg_apu_irq_timing"
    run_test "APU Reset Timing" "blargg_apu_reset_timing"
    run_test "APU Length Halt Timing" "blargg_apu_len_halt_timing"
    run_test "APU Length Reload Timing" "blargg_apu_len_reload_timing"

    echo ""
    echo "=== APU Tests (Comprehensive) ==="
    run_test "APU Test 1 - Length Counter" "apu_test_1_len_ctr"
    run_test "APU Test 2 - Length Table" "apu_test_2_len_table"
    run_test "APU Test 3 - IRQ Flag" "apu_test_3_irq_flag"
    run_test "APU Test 4 - Jitter" "apu_test_4_jitter"
    run_test "APU Test 5 - Length Timing" "apu_test_5_len_timing"
    run_test "APU Test 6 - IRQ Flag Timing" "apu_test_6_irq_flag_timing"
    run_test "APU Test 7 - DMC Basics" "apu_test_7_dmc_basics"
    run_test "APU Test 8 - DMC Rates" "apu_test_8_dmc_rates"

    echo ""
    echo "=== APU Mixer Tests ==="
    run_test "APU Mixer - Square" "apu_mixer_square"
    run_test "APU Mixer - Triangle" "apu_mixer_triangle"
    run_test "APU Mixer - Noise" "apu_mixer_noise"
    run_test "APU Mixer - DMC" "apu_mixer_dmc"

    echo ""
    echo "=== APU Reset Test ==="
    run_test "APU Reset" "apu_reset"
fi

# Run sprite tests
if [ "$RUN_SPRITE" = true ]; then
    echo ""
    echo "=== Sprite Hit Tests ==="
    run_test "Sprite Hit Basics" "sprite_hit_basics"
    run_test "Sprite Hit Alignment" "sprite_hit_alignment"
    run_test "Sprite Hit Corners" "sprite_hit_corners"
    run_test "Sprite Hit Flip" "sprite_hit_flip"
    run_test "Sprite Hit Left Clip" "sprite_hit_left_clip"
    run_test "Sprite Hit Right Edge" "sprite_hit_right_edge"
    run_test "Sprite Hit Screen Bottom" "sprite_hit_screen_bottom"
    run_test "Sprite Hit Double Height" "sprite_hit_double_height"
    run_test "Sprite Hit Timing Basics" "sprite_hit_timing_basics"
    run_test "Sprite Hit Timing Order" "sprite_hit_timing_order"
    run_test "Sprite Hit Edge Timing" "sprite_hit_edge_timing"

    echo ""
    echo "=== Sprite Overflow Tests ==="
    run_test "Sprite Overflow Basics" "sprite_overflow_basics"
    run_test "Sprite Overflow Details" "sprite_overflow_details"
    run_test "Sprite Overflow Timing" "sprite_overflow_timing"
    run_test "Sprite Overflow Obscure" "sprite_overflow_obscure"
    run_test "Sprite Overflow Emulator" "sprite_overflow_emulator"
fi

# Close JSON output
if [ "$JSON_OUTPUT" = true ]; then
    # Remove trailing comma from last test
    sed -i '$ s/,$//' "$JSON_FILE"
    echo "  ]," >> "$JSON_FILE"
    echo "  \"summary\": {" >> "$JSON_FILE"
    echo "    \"total\": $TOTAL_TESTS," >> "$JSON_FILE"
    echo "    \"passed\": $PASSED_TESTS," >> "$JSON_FILE"
    echo "    \"failed\": $FAILED_TESTS" >> "$JSON_FILE"
    echo "  }" >> "$JSON_FILE"
    echo "}" >> "$JSON_FILE"
    echo ""
    echo "JSON results written to: $JSON_FILE"
fi

# Print summary
echo ""
echo "========================================="
echo "  Test Summary"
echo "========================================="
echo ""
echo "Total tests:  $TOTAL_TESTS"
echo -e "${GREEN}Passed:       $PASSED_TESTS${NC}"
echo -e "${RED}Failed:       $FAILED_TESTS${NC}"
echo ""

PASS_RATE=0
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$(echo "scale=2; ($PASSED_TESTS * 100) / $TOTAL_TESTS" | bc)
fi

echo "Pass rate:    $PASS_RATE%"
echo ""
echo "Detailed results saved to: $RESULTS_FILE"
echo ""

# Exit with error if any tests failed
if [ $FAILED_TESTS -gt 0 ]; then
    exit 1
fi

exit 0
