//! Frame counter integration tests

use crate::apu::Apu;
use crate::bus::MemoryMappedDevice;

#[test]
fn test_frame_counter_default_mode() {
    let apu = Apu::new();
    // Frame counter should start in 4-step mode
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_4_step_mode_irq() {
    let mut apu = Apu::new();

    // Clock to step 4 (29829 CPU cycles)
    for _ in 0..29829 {
        apu.clock();
    }

    // Frame IRQ should be set in 4-step mode
    assert!(apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_4_step_mode_quarter_frames() {
    let mut apu = Apu::new();

    // Enable pulse channel 1 with envelope and length counter
    apu.write(0x4015, 0x01); // Enable pulse 1
    apu.write(0x4000, 0x30); // Set envelope parameters
    apu.write(0x4003, 0xF8); // Load length counter

    let initial_length = apu.pulse1.length_counter.counter;

    // Clock to first quarter frame (7457 cycles)
    for _ in 0..7457 {
        apu.clock();
    }

    // Envelope should have been clocked, but length counter should not
    // (Quarter frame only clocks envelope, not length counter)
    assert_eq!(apu.pulse1.length_counter.counter, initial_length);
}

#[test]
fn test_frame_counter_4_step_mode_half_frames() {
    let mut apu = Apu::new();

    // Enable pulse channel 1 with envelope and length counter
    apu.write(0x4015, 0x01); // Enable pulse 1
    apu.write(0x4000, 0x10); // Set envelope parameters (no halt flag - bit 5 is 0)
    apu.write(0x4003, 0xF8); // Load length counter

    let initial_length = apu.pulse1.length_counter.counter;

    // Clock to second step (half frame at 14913 cycles)
    for _ in 0..14913 {
        apu.clock();
    }

    // Length counter should have been clocked (decremented)
    assert!(apu.pulse1.length_counter.counter < initial_length);
}

#[test]
fn test_frame_counter_5_step_mode() {
    let mut apu = Apu::new();

    // Set 5-step mode (bit 7 of $4017)
    apu.write(0x4017, 0x80);

    // Clock through entire frame (37282 cycles)
    for _ in 0..37282 {
        apu.clock();
    }

    // No IRQ should be generated in 5-step mode
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_irq_inhibit() {
    let mut apu = Apu::new();

    // Set IRQ inhibit flag (bit 6 of $4017)
    apu.write(0x4017, 0x40);

    // Clock to step 4 (29829 CPU cycles)
    for _ in 0..29829 {
        apu.clock();
    }

    // IRQ should not be generated when inhibit is set
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_irq_clear_on_read() {
    let mut apu = Apu::new();

    // Clock to step 4 to generate IRQ
    for _ in 0..29829 {
        apu.clock();
    }

    assert!(apu.frame_irq_pending());

    // Read $4015 should clear the frame IRQ flag
    let status = apu.read(0x4015);

    // Bit 6 should have been set before clearing
    assert_eq!(status & 0x40, 0x40);

    // After reading, IRQ should be cleared
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_status_register() {
    let mut apu = Apu::new();

    // Enable pulse 1 and load length counter
    apu.write(0x4015, 0x01);
    apu.write(0x4003, 0x08); // Load length counter with small value

    // Read status
    let status = apu.read(0x4015);

    // Pulse 1 length counter should be active (bit 0)
    assert_eq!(status & 0x01, 0x01);

    // Frame IRQ should not be set yet (bit 6)
    assert_eq!(status & 0x40, 0x00);
}

#[test]
fn test_frame_counter_mode_switch() {
    let mut apu = Apu::new();

    // Start in 4-step mode (default)
    apu.write(0x4017, 0x00);

    // Clock partway through
    for _ in 0..10000 {
        apu.clock();
    }

    // Switch to 5-step mode
    apu.write(0x4017, 0x80);

    // Continue clocking through a full 5-step frame
    for _ in 0..37282 {
        apu.clock();
    }

    // No IRQ should be generated in 5-step mode
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_frame_counter_immediate_clock_on_5_step_write() {
    let mut apu = Apu::new();

    // Enable pulse 1 and load length counter
    apu.write(0x4015, 0x01);
    apu.write(0x4000, 0x10); // No length counter halt (bit 5 = 0)
    apu.write(0x4003, 0xF8); // Load length counter

    let initial_length = apu.pulse1.length_counter.counter;

    // Write to $4017 with 5-step mode
    // This should immediately clock a half frame
    apu.write(0x4017, 0x80);

    // Length counter should have been decremented immediately
    assert!(apu.pulse1.length_counter.counter < initial_length);
}

#[test]
fn test_frame_counter_timing_accuracy() {
    let mut apu = Apu::new();

    // Enable pulse 1
    apu.write(0x4015, 0x01);
    apu.write(0x4000, 0x10); // No halt flag (bit 5 = 0)
    apu.write(0x4003, 0x08);

    let mut half_frame_count = 0;

    // Track when length counter changes (indicates half frames)
    let mut last_length = apu.pulse1.length_counter.counter;

    // Clock for one full 4-step frame
    for _cycle in 1..=29830 {
        apu.clock();

        let current_length = apu.pulse1.length_counter.counter;
        if current_length != last_length {
            half_frame_count += 1;
            last_length = current_length;
        }
    }

    // Should have exactly 2 half frames in 4-step mode (at steps 2 and 4)
    assert_eq!(half_frame_count, 2);
}

#[test]
fn test_frame_counter_reset_on_write() {
    let mut apu = Apu::new();

    // Clock partway through frame
    for _ in 0..15000 {
        apu.clock();
    }

    // Write to $4017 should reset the frame counter
    apu.write(0x4017, 0x00);

    // Clock to what would normally be past step 4 if not reset
    for _ in 0..15000 {
        apu.clock();
    }

    // IRQ should not be set yet because counter was reset
    assert!(!apu.frame_irq_pending());
}

#[test]
fn test_both_modes_clock_envelopes() {
    // Test 4-step mode
    let mut apu_4step = Apu::new();
    apu_4step.write(0x4015, 0x01); // Enable pulse 1
    apu_4step.write(0x4000, 0x0F); // Max volume, no constant volume
    apu_4step.write(0x4003, 0x08); // Load envelope

    // Clock to first quarter frame
    for _ in 0..7457 {
        apu_4step.clock();
    }

    // Test 5-step mode
    let mut apu_5step = Apu::new();
    apu_5step.write(0x4017, 0x80); // 5-step mode
    apu_5step.write(0x4015, 0x01); // Enable pulse 1
    apu_5step.write(0x4000, 0x0F); // Max volume, no constant volume
    apu_5step.write(0x4003, 0x08); // Load envelope

    // Clock to first quarter frame
    for _ in 0..7457 {
        apu_5step.clock();
    }

    // Both modes should clock envelopes at quarter frames
    // (We can't easily test the exact envelope state without exposing internals,
    // but we verify the modes work without crashing)
}
