//! Pulse channel functionality tests

use crate::apu::Apu;
use crate::bus::MemoryMappedDevice;

// ========================================
// Pulse Channel Functionality Tests
// ========================================

#[test]
fn test_pulse_duty_cycle_patterns() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Test each duty cycle pattern
    for duty in 0..4 {
        apu.write(0x4000, (duty << 6) | 0x30); // Set duty cycle, constant volume
        apu.write(0x4003, 0x08); // Load length counter

        assert_eq!(apu.pulse1.duty, duty);
    }
}

#[test]
fn test_pulse_envelope_constant_volume() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Configure constant volume mode, volume = 10
    apu.write(0x4000, 0x1A); // Constant volume (bit 4), volume = 10
    apu.write(0x4003, 0x08); // Load length counter (restarts envelope)

    // Volume should be 10 (constant)
    assert_eq!(apu.pulse1.envelope.volume(), 10);

    // Clock envelope - should not change in constant volume mode
    apu.clock_quarter_frame();
    assert_eq!(apu.pulse1.envelope.volume(), 10);
}

#[test]
fn test_pulse_envelope_decay() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Configure decay mode (not constant volume), period = 1
    apu.write(0x4000, 0x01); // Decay mode, period = 1
    apu.write(0x4003, 0x08); // Load length counter (restarts envelope)

    // Envelope start flag should be set
    assert!(apu.pulse1.envelope.start);

    // Clock envelope once - this reloads decay level to 15
    apu.clock_quarter_frame();

    // After first clock with start flag, decay level should be 15
    assert_eq!(apu.pulse1.envelope.decay_level, 15);
    assert!(!apu.pulse1.envelope.start); // Start flag cleared

    // Clock envelope twice more (once to decrement divider, once to reload and decrement decay)
    apu.clock_quarter_frame(); // Divider: 1 -> 0
    apu.clock_quarter_frame(); // Divider reloads, decay: 15 -> 14

    // Decay level should have decreased
    assert_eq!(apu.pulse1.envelope.decay_level, 14);
}

#[test]
fn test_pulse_length_counter() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Configure without halt flag
    apu.write(0x4000, 0x00); // No halt
    apu.write(0x4003, 0x08); // Load length counter, index = 1

    // Length counter should be loaded from table
    assert!(apu.pulse1.length_counter.counter > 0);
    let initial_count = apu.pulse1.length_counter.counter;

    // Clock length counter
    apu.clock_half_frame();

    // Counter should have decreased
    assert_eq!(apu.pulse1.length_counter.counter, initial_count - 1);
}

#[test]
fn test_pulse_length_counter_halt() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Configure with halt flag
    apu.write(0x4000, 0x20); // Halt flag set (bit 5)
    apu.write(0x4003, 0x08); // Load length counter

    let initial_count = apu.pulse1.length_counter.counter;

    // Clock length counter
    apu.clock_half_frame();

    // Counter should NOT have decreased due to halt
    assert_eq!(apu.pulse1.length_counter.counter, initial_count);
}

#[test]
fn test_pulse_sweep_calculation() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Set initial timer period
    apu.write(0x4002, 0x00); // Low byte = 0
    apu.write(0x4003, 0x08); // High = 1, so period = 0x100

    // Configure sweep: enabled, period=0, negate=0, shift=1
    // This should double the period when sweep clocks
    apu.write(0x4001, 0x81); // Enabled, period=0, shift=1

    // Target period should be current + (current >> shift)
    // 0x100 + (0x100 >> 1) = 0x100 + 0x80 = 0x180
    let target = apu.pulse1.sweep.calculate_target_period(0x100);
    assert_eq!(target, 0x180);
}

#[test]
fn test_pulse_sweep_muting() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Set timer period < 8 (should mute)
    apu.write(0x4002, 0x05);
    apu.write(0x4003, 0x08); // Period = 5

    // Configure constant volume so we can check output
    apu.write(0x4000, 0x3F); // Constant volume = 15

    // Output should be 0 due to period < 8
    assert_eq!(apu.pulse1_output(), 0);
}

#[test]
fn test_pulse_output_generation() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);

    // Configure: 50% duty, constant volume = 8, period = 100
    apu.write(0x4000, 0x98); // Duty=2 (50%), constant vol=8
    apu.write(0x4002, 0x64); // Period low = 100
    apu.write(0x4003, 0x08); // Load length counter

    // Output should be either 0 or 8 depending on duty position
    let output = apu.pulse1_output();
    assert!(output == 0 || output == 8);

    // Clock timer to change duty position
    for _ in 0..=100 {
        apu.clock();
    }

    // Output might have changed
    let new_output = apu.pulse1_output();
    assert!(new_output == 0 || new_output == 8);
}

#[test]
fn test_pulse_disable_clears_length_counter() {
    let mut apu = Apu::new();

    // Enable and configure pulse 1
    apu.write(0x4015, 0x01);
    apu.write(0x4000, 0x30);
    apu.write(0x4003, 0x08); // Load length counter

    assert!(apu.pulse1.length_counter.counter > 0);

    // Disable pulse 1
    apu.write(0x4015, 0x00);

    // Length counter should be cleared
    assert_eq!(apu.pulse1.length_counter.counter, 0);
    assert!(!apu.pulse1.is_active());
}

#[test]
fn test_both_pulse_channels_work() {
    let mut apu = Apu::new();

    // Enable both pulse channels
    apu.write(0x4015, 0x03);

    // Configure pulse 1
    apu.write(0x4000, 0x3F); // Constant volume = 15
    apu.write(0x4003, 0x08);

    // Configure pulse 2
    apu.write(0x4004, 0x38); // Constant volume = 8
    apu.write(0x4007, 0x08);

    // Both should produce output
    assert!(apu.pulse1_output() <= 15);
    assert!(apu.pulse2_output() <= 8);

    // Mixed output should be sum (saturating)
    let mixed = apu.output();
    assert!(mixed <= 30);
}

#[test]
fn test_sweep_units_differ_for_pulse_1_and_2() {
    // Pulse 1 uses one's complement for negate
    // Pulse 2 uses two's complement for negate

    let mut apu = Apu::new();

    // Enable both channels
    apu.write(0x4015, 0x03);

    // Set same period for both
    apu.write(0x4002, 0x00);
    apu.write(0x4003, 0x08); // Period = 0x100
    apu.write(0x4006, 0x00);
    apu.write(0x4007, 0x08); // Period = 0x100

    // Configure same sweep with negate for both
    apu.write(0x4001, 0x89); // Enabled, negate, shift=1
    apu.write(0x4005, 0x89); // Enabled, negate, shift=1

    // Calculate target periods
    let target1 = apu.pulse1.sweep.calculate_target_period(0x100);
    let target2 = apu.pulse2.sweep.calculate_target_period(0x100);

    // They should differ by 1 due to one's vs two's complement
    // Pulse 1: 0x100 - 0x80 - 1 = 0x7F
    // Pulse 2: 0x100 - 0x80 = 0x80
    assert_eq!(target1, 0x7F);
    assert_eq!(target2, 0x80);
}
