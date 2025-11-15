//! Triangle channel functionality tests

use crate::apu::Apu;
use crate::bus::MemoryMappedDevice;

// ========================================
// Triangle Channel Functionality Tests
// ========================================

#[test]
fn test_triangle_linear_counter_setup() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure linear counter with reload value = 42
    apu.write(0x4008, 0x2A); // Control flag = 0, reload value = 42

    assert_eq!(apu.triangle.linear_counter.reload_value, 42);
    assert!(!apu.triangle.linear_counter.control_flag);
}

#[test]
fn test_triangle_linear_counter_control_flag() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with control flag set
    apu.write(0x4008, 0x80); // Control flag = 1, reload value = 0

    assert!(apu.triangle.linear_counter.control_flag);
    assert!(apu.triangle.length_counter.halt); // Control flag doubles as length counter halt
}

#[test]
fn test_triangle_linear_counter_reload() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure linear counter with reload value = 10
    apu.write(0x4008, 0x0A);
    apu.write(0x400B, 0x08); // This sets the reload flag

    // Reload flag should be set
    assert!(apu.triangle.linear_counter.reload_flag);

    // Clock linear counter - should reload to 10
    apu.clock_quarter_frame();

    assert_eq!(apu.triangle.linear_counter.counter, 10);
    // Reload flag should be cleared because control flag is 0
    assert!(!apu.triangle.linear_counter.reload_flag);
}

#[test]
fn test_triangle_linear_counter_countdown() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure linear counter with reload value = 5
    apu.write(0x4008, 0x05);
    apu.write(0x400B, 0x08); // Set reload flag

    // Clock once to reload
    apu.clock_quarter_frame();
    assert_eq!(apu.triangle.linear_counter.counter, 5);

    // Clock again - should decrement
    apu.clock_quarter_frame();
    assert_eq!(apu.triangle.linear_counter.counter, 4);

    // Clock again - should decrement
    apu.clock_quarter_frame();
    assert_eq!(apu.triangle.linear_counter.counter, 3);
}

#[test]
fn test_triangle_linear_counter_with_control_flag() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with control flag set (reload flag never clears)
    apu.write(0x4008, 0x85); // Control = 1, reload value = 5
    apu.write(0x400B, 0x08); // Set reload flag

    // Clock - should reload to 5
    apu.clock_quarter_frame();
    assert_eq!(apu.triangle.linear_counter.counter, 5);

    // Clock again - should still reload to 5 because control flag keeps reload flag set
    apu.clock_quarter_frame();
    assert_eq!(apu.triangle.linear_counter.counter, 5);
}

#[test]
fn test_triangle_length_counter() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure without halt flag
    apu.write(0x4008, 0x00); // Control flag = 0
    apu.write(0x400B, 0x08); // Length counter index = 1

    assert!(apu.triangle.length_counter.counter > 0);
    let initial_count = apu.triangle.length_counter.counter;

    // Clock length counter
    apu.clock_half_frame();

    // Counter should have decreased
    assert_eq!(apu.triangle.length_counter.counter, initial_count - 1);
}

#[test]
fn test_triangle_length_counter_halt() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with halt flag (control flag)
    apu.write(0x4008, 0x80); // Control flag = 1 (also sets halt)
    apu.write(0x400B, 0x08); // Length counter index = 1

    let initial_count = apu.triangle.length_counter.counter;

    // Clock length counter
    apu.clock_half_frame();

    // Counter should NOT have decreased due to halt
    assert_eq!(apu.triangle.length_counter.counter, initial_count);
}

#[test]
fn test_triangle_wave_output() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure triangle: control flag set, reload value = 127
    apu.write(0x4008, 0xFF); // Maximum linear counter
    apu.write(0x400A, 0x64); // Timer low = 100
    apu.write(0x400B, 0x08); // Load length counter

    // Clock quarter frame to reload linear counter
    apu.clock_quarter_frame();

    // Triangle should be active
    assert!(apu.triangle.is_active());

    // Output should be in range 0-15
    let output = apu.triangle_output();
    assert!(output <= 15);

    // Initial position should be 0, so output should be 15
    assert_eq!(output, 15);
}

#[test]
fn test_triangle_wave_sequence() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure triangle with control flag set (continuous reload)
    apu.write(0x4008, 0xFF);
    apu.write(0x400A, 0x10); // Timer period = 16 (not 0, to avoid ultrasonic)
    apu.write(0x400B, 0xF8); // Load length counter with max value

    // Need to clock quarter frame to reload linear counter
    apu.clock_quarter_frame();

    // Triangle should be active
    assert!(apu.triangle.is_active());

    // Clock the timer enough times to advance through the sequence
    // Each timer period is 16, so we need to clock 32 * 17 times to see full sequence
    let mut outputs = Vec::new();
    for _ in 0..32 {
        outputs.push(apu.triangle_output());
        // Clock enough to advance sequencer once
        for _ in 0..17 {
            apu.clock();
        }
    }

    // Should follow triangle sequence: 15,14,13,...,0,0,1,...,15
    assert_eq!(outputs[0], 15);
    assert_eq!(outputs[15], 0);
    assert_eq!(outputs[16], 0);
    assert_eq!(outputs[31], 15);
}

#[test]
fn test_triangle_ultrasonic_silencing() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with period < 2 (should be muted)
    apu.write(0x4008, 0xFF); // Linear counter max
    apu.write(0x400A, 0x01); // Timer period = 1
    apu.write(0x400B, 0xF8); // Length counter max

    // Clock quarter frame to reload linear counter
    apu.clock_quarter_frame();

    // Should be active but muted due to ultrasonic silencing
    assert!(apu.triangle.linear_counter.is_active());
    assert!(apu.triangle.length_counter.is_active());
    assert_eq!(apu.triangle_output(), 0); // Muted due to period < 2
}

#[test]
fn test_triangle_ultrasonic_threshold() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with period = 2 (just above threshold, should NOT be muted)
    apu.write(0x4008, 0xFF);
    apu.write(0x400A, 0x02); // Timer period = 2
    apu.write(0x400B, 0xF8);

    // Clock quarter frame to reload linear counter
    apu.clock_quarter_frame();

    // Should be active and NOT muted
    assert!(apu.triangle.is_active());
    assert_eq!(apu.triangle_output(), 15); // Not muted
}

#[test]
fn test_triangle_requires_both_counters() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure with linear counter = 0
    apu.write(0x4008, 0x00); // Reload value = 0
    apu.write(0x400A, 0x10);
    apu.write(0x400B, 0xF8); // Length counter loaded

    // Clock quarter frame to reload linear counter to 0
    apu.clock_quarter_frame();

    // Linear counter is 0, so output should be 0
    assert!(!apu.triangle.linear_counter.is_active());
    assert_eq!(apu.triangle_output(), 0);
}

#[test]
fn test_triangle_sequencer_only_clocks_when_both_counters_active() {
    let mut apu = Apu::new();

    // Enable triangle channel
    apu.write(0x4015, 0x04);

    // Configure triangle
    apu.write(0x4008, 0x05); // Linear counter reload = 5
    apu.write(0x400A, 0x00); // Timer period = 0
    apu.write(0x400B, 0xF8); // Length counter max

    // Clock to reload linear counter
    apu.clock_quarter_frame();

    let initial_position = apu.triangle.sequence_position;

    // Linear counter is active, length counter is active, so sequencer should advance
    apu.clock();
    assert_ne!(apu.triangle.sequence_position, initial_position);

    // Now drain linear counter to 0
    for _ in 0..6 {
        apu.clock_quarter_frame();
    }

    // Linear counter should be 0
    assert!(!apu.triangle.linear_counter.is_active());

    // Save current position
    let position_before = apu.triangle.sequence_position;

    // Clock timer - sequencer should NOT advance
    apu.clock();
    assert_eq!(apu.triangle.sequence_position, position_before);
}

#[test]
fn test_triangle_status_register() {
    let mut apu = Apu::new();

    // Initially, no channels active
    assert_eq!(apu.read(0x4015) & 0x04, 0x00);

    // Enable triangle and load length counter
    apu.write(0x4015, 0x04);
    apu.write(0x4008, 0x80);
    apu.write(0x400B, 0x08);

    // Status should show triangle active (bit 2)
    assert_eq!(apu.read(0x4015) & 0x04, 0x04);
}

#[test]
fn test_triangle_disable_clears_length_counter() {
    let mut apu = Apu::new();

    // Enable and configure triangle
    apu.write(0x4015, 0x04);
    apu.write(0x4008, 0x80);
    apu.write(0x400B, 0x08);

    assert!(apu.triangle.length_counter.counter > 0);

    // Disable triangle
    apu.write(0x4015, 0x00);

    // Length counter should be cleared
    assert_eq!(apu.triangle.length_counter.counter, 0);
    assert!(!apu.triangle.is_active());
}

#[test]
fn test_triangle_with_pulse_channels() {
    let mut apu = Apu::new();

    // Enable all channels
    apu.write(0x4015, 0x07);

    // Configure pulse 1
    apu.write(0x4000, 0x3F); // Constant volume = 15
    apu.write(0x4003, 0x08);

    // Configure pulse 2
    apu.write(0x4004, 0x38); // Constant volume = 8
    apu.write(0x4007, 0x08);

    // Configure triangle
    apu.write(0x4008, 0xFF);
    apu.write(0x400A, 0x10);
    apu.write(0x400B, 0xF8);

    // Clock quarter frame to activate triangle
    apu.clock_quarter_frame();

    // All should produce output
    assert!(apu.pulse1_output() <= 15);
    assert!(apu.pulse2_output() <= 8);
    assert!(apu.triangle_output() <= 15);

    // Status register should show all three active
    assert_eq!(apu.read(0x4015) & 0x07, 0x07);
}

#[test]
fn test_triangle_timer_period() {
    let mut apu = Apu::new();

    // Enable triangle
    apu.write(0x4015, 0x04);

    // Set timer period using low and high bytes
    apu.write(0x400A, 0xAB); // Low byte
    apu.write(0x400B, 0x05); // High byte (bits 2-0) = 5

    // Period should be (5 << 8) | 0xAB = 0x5AB
    assert_eq!(apu.triangle.timer.period, 0x5AB);
}

#[test]
fn test_triangle_no_envelope() {
    let mut apu = Apu::new();

    // Enable triangle
    apu.write(0x4015, 0x04);

    // Configure triangle
    apu.write(0x4008, 0xFF);
    apu.write(0x400A, 0x10);
    apu.write(0x400B, 0xF8);

    // Clock to activate
    apu.clock_quarter_frame();

    // Triangle has no envelope - output is always the sequence value (0-15)
    // Never affected by volume settings
    let output = apu.triangle_output();
    assert!(output <= 15);

    // Clock many frames - output value depends only on sequence position, not time
    for _ in 0..100 {
        apu.clock_quarter_frame();
    }

    let output_after = apu.triangle_output();
    assert!(output_after <= 15);
}
