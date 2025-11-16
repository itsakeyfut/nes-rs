//! Initialization, register, and basic APU tests

use crate::apu::Apu;
use crate::bus::MemoryMappedDevice;

// ========================================
// Initialization Tests
// ========================================

#[test]
fn test_apu_initialization() {
    let apu = Apu::new();
    // Pulse channels should be initialized
    assert!(!apu.pulse1.enabled);
    assert!(!apu.pulse2.enabled);
    // Verify sweep units were created with correct channel numbers
    assert_eq!(apu.pulse1.sweep.channel, 1);
    assert_eq!(apu.pulse2.sweep.channel, 2);
    // Triangle channel should be initialized
    assert!(!apu.triangle.enabled);
    assert_eq!(apu.triangle.linear_counter.counter, 0);
    assert_eq!(apu.triangle.length_counter.counter, 0);
    // Noise channel should be initialized
    assert!(!apu.noise.enabled);
    assert_eq!(apu.noise.length_counter.counter, 0);
    // Other channels (stub registers)
    assert_eq!(apu.dmc_flags_rate, 0x00);
    assert_eq!(apu.status_control, 0x00);
    assert_eq!(apu.frame_counter, 0x00);
}

#[test]
fn test_apu_default() {
    let apu = Apu::default();
    assert_eq!(apu.status_control, 0x00);
}

#[test]
fn test_apu_reset() {
    let mut apu = Apu::new();
    apu.write(0x4015, 0x01);
    apu.write(0x4000, 0x80);
    apu.write(0x4015, 0x0F);

    // Verify something changed
    assert_eq!(apu.status_control, 0x0F);

    apu.reset();

    // After reset, everything should be back to defaults
    assert!(!apu.pulse1.enabled);
    assert_eq!(apu.status_control, 0x00);
}

// ========================================
// Pulse 1 Register Tests ($4000-$4003)
// ========================================

#[test]
fn test_write_pulse1_registers() {
    let mut apu = Apu::new();

    // Enable Pulse 1 first
    apu.write(0x4015, 0x01);

    // Write to pulse 1 registers
    apu.write(0x4000, 0xBF); // Duty=2 (75%), envelope loop, constant volume, volume=15
    apu.write(0x4001, 0x08); // Sweep disabled, period=1, shift=0
    apu.write(0x4002, 0xA9); // Timer low byte
    apu.write(0x4003, 0x0F); // Length counter index=0, timer high=7

    // Verify duty cycle was set (bits 7-6)
    assert_eq!(apu.pulse1.duty, 2); // 0xBF >> 6 = 2 (75% duty)

    // Verify envelope settings
    assert!(apu.pulse1.envelope.constant_volume); // Bit 4
    assert!(apu.pulse1.envelope.loop_flag); // Bit 5
    assert_eq!(apu.pulse1.envelope.period, 15); // Bits 3-0

    // Verify timer period (11-bit value from registers 2 and 3)
    assert_eq!(apu.pulse1.timer.period, 0x7A9); // (0x0F & 0x07) << 8 | 0xA9 = 0x7A9

    // Verify channel is enabled
    assert!(apu.pulse1.enabled);
}

#[test]
fn test_read_pulse1_registers_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x4000, 0xBF);

    // Pulse 1 registers are write-only
    assert_eq!(apu.read(0x4000), 0x00);
    assert_eq!(apu.read(0x4001), 0x00);
    assert_eq!(apu.read(0x4002), 0x00);
    assert_eq!(apu.read(0x4003), 0x00);
}

// ========================================
// Pulse 2 Register Tests ($4004-$4007)
// ========================================

#[test]
fn test_write_pulse2_registers() {
    let mut apu = Apu::new();

    // Enable Pulse 2 first
    apu.write(0x4015, 0x02);

    apu.write(0x4004, 0x80); // Duty=2 (50%), no loop, no constant volume
    apu.write(0x4005, 0x10); // Sweep settings
    apu.write(0x4006, 0x55); // Timer low
    apu.write(0x4007, 0x20); // Length counter index=4, timer high=0

    // Verify duty cycle
    assert_eq!(apu.pulse2.duty, 2); // 0x80 >> 6 = 2

    // Verify timer period
    assert_eq!(apu.pulse2.timer.period, 0x055); // (0x20 & 0x07) << 8 | 0x55 = 0x055

    // Verify channel is enabled
    assert!(apu.pulse2.enabled);
}

#[test]
fn test_read_pulse2_registers_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x4004, 0x80);

    // Pulse 2 registers are write-only
    assert_eq!(apu.read(0x4004), 0x00);
    assert_eq!(apu.read(0x4005), 0x00);
    assert_eq!(apu.read(0x4006), 0x00);
    assert_eq!(apu.read(0x4007), 0x00);
}

// ========================================
// Triangle Register Tests ($4008-$400B)
// ========================================

#[test]
fn test_write_triangle_registers() {
    let mut apu = Apu::new();

    // Enable triangle channel first
    apu.write(0x4015, 0x04);

    apu.write(0x4008, 0x81); // Control flag set, reload value = 1
    apu.write(0x4009, 0x00); // Unused
    apu.write(0x400A, 0xDD); // Timer low
    apu.write(0x400B, 0x18); // Length counter index=3, timer high=0

    // Verify linear counter settings
    assert!(apu.triangle.linear_counter.control_flag);
    assert_eq!(apu.triangle.linear_counter.reload_value, 0x01);

    // Verify timer period
    assert_eq!(apu.triangle.timer.period, 0x0DD);

    // Verify channel is enabled
    assert!(apu.triangle.enabled);
}

#[test]
fn test_read_triangle_registers_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x4008, 0x81);

    // Triangle registers are write-only
    assert_eq!(apu.read(0x4008), 0x00);
    assert_eq!(apu.read(0x4009), 0x00);
    assert_eq!(apu.read(0x400A), 0x00);
    assert_eq!(apu.read(0x400B), 0x00);
}

// ========================================
// Noise Register Tests ($400C-$400F)
// ========================================

#[test]
fn test_write_noise_registers() {
    let mut apu = Apu::new();

    // Enable noise channel first
    apu.write(0x4015, 0x08);

    apu.write(0x400C, 0x30); // Envelope with loop and constant volume=0
    apu.write(0x400D, 0x00); // Unused
    apu.write(0x400E, 0x87); // Mode 1 (bit 7 set), period index 7
    apu.write(0x400F, 0x10); // Length counter index=2

    // Verify envelope settings
    assert!(apu.noise.envelope.loop_flag); // Bit 5 of 0x30
    assert!(apu.noise.envelope.constant_volume); // Bit 4 of 0x30
    assert_eq!(apu.noise.envelope.period, 0); // Bits 3-0 of 0x30

    // Verify mode flag
    assert!(apu.noise.mode); // Bit 7 of 0x87

    // Verify timer period (from period table index 7)
    assert_eq!(apu.noise.timer.period, 160);

    // Verify channel is enabled
    assert!(apu.noise.enabled);
}

#[test]
fn test_read_noise_registers_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x400C, 0x30);

    // Noise registers are write-only
    assert_eq!(apu.read(0x400C), 0x00);
    assert_eq!(apu.read(0x400D), 0x00);
    assert_eq!(apu.read(0x400E), 0x00);
    assert_eq!(apu.read(0x400F), 0x00);
}

// ========================================
// DMC Register Tests ($4010-$4013)
// ========================================

#[test]
fn test_write_dmc_registers() {
    let mut apu = Apu::new();
    apu.write(0x4010, 0x0F);
    apu.write(0x4011, 0x40);
    apu.write(0x4012, 0xC0);
    apu.write(0x4013, 0xFF);

    assert_eq!(apu.dmc_flags_rate, 0x0F);
    assert_eq!(apu.dmc_direct_load, 0x40);
    assert_eq!(apu.dmc_sample_address, 0xC0);
    assert_eq!(apu.dmc_sample_length, 0xFF);
}

#[test]
fn test_read_dmc_registers_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x4010, 0x0F);

    // DMC registers are write-only
    assert_eq!(apu.read(0x4010), 0x00);
    assert_eq!(apu.read(0x4011), 0x00);
    assert_eq!(apu.read(0x4012), 0x00);
    assert_eq!(apu.read(0x4013), 0x00);
}

// ========================================
// Control Register Tests ($4015, $4017)
// ========================================

#[test]
fn test_write_status_control() {
    let mut apu = Apu::new();
    apu.write(0x4015, 0x0F); // Enable all channels

    assert_eq!(apu.status_control, 0x0F);
}

#[test]
fn test_read_status_control() {
    let mut apu = Apu::new();

    // Initially no channels active
    assert_eq!(apu.read(0x4015), 0x00);

    // Enable pulse 1 and write length counter
    apu.write(0x4015, 0x01);
    apu.write(0x4000, 0x30); // Constant volume
    apu.write(0x4003, 0x08); // Load length counter

    // Status should show pulse 1 active (bit 0)
    assert_eq!(apu.read(0x4015), 0x01);

    // Enable pulse 2 and write length counter
    apu.write(0x4015, 0x03); // Enable both
    apu.write(0x4007, 0x08); // Load pulse 2 length counter

    // Status should show both pulse channels active (bits 0-1)
    assert_eq!(apu.read(0x4015), 0x03);
}

#[test]
fn test_write_frame_counter() {
    let mut apu = Apu::new();
    apu.write(0x4017, 0x40); // Enable IRQ inhibit

    assert_eq!(apu.frame_counter, 0x40);
}

#[test]
fn test_read_frame_counter_return_zero() {
    let mut apu = Apu::new();
    apu.write(0x4017, 0x40);

    // Frame counter is write-only
    assert_eq!(apu.read(0x4017), 0x00);
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_typical_apu_initialization_sequence() {
    let mut apu = Apu::new();

    // Typical game initialization
    apu.write(0x4015, 0x00); // Disable all channels
    apu.write(0x4017, 0x40); // Set frame counter mode

    assert_eq!(apu.status_control, 0x00);
    assert_eq!(apu.frame_counter, 0x40);
}

#[test]
fn test_configure_pulse_channel() {
    let mut apu = Apu::new();

    // Enable Pulse 1 first
    apu.write(0x4015, 0x01);

    // Configure Pulse 1 for a tone
    apu.write(0x4000, 0xBF); // Duty=2 (75%), loop, constant vol=15
    apu.write(0x4001, 0x08); // Sweep
    apu.write(0x4002, 0xA9); // Timer low
    apu.write(0x4003, 0x00); // Timer high=0, length counter index=0

    // Verify configuration
    assert_eq!(apu.pulse1.duty, 2);
    assert!(apu.pulse1.enabled);
    assert_eq!(apu.pulse1.envelope.volume(), 15); // Constant volume mode
    assert!(apu.pulse1.is_active());
}

#[test]
fn test_all_channels_can_be_written() {
    let mut apu = Apu::new();

    // Write to all channel registers
    apu.write(0x4000, 0x01); // Pulse 1
    apu.write(0x4004, 0x02); // Pulse 2
    apu.write(0x4008, 0x03); // Triangle
    apu.write(0x400C, 0x04); // Noise
    apu.write(0x4010, 0x05); // DMC

    // Verify pulse channels (implemented)
    assert_eq!(apu.pulse1.duty, 0); // 0x01 >> 6 = 0
    assert_eq!(apu.pulse2.duty, 0); // 0x02 >> 6 = 0

    // Verify triangle channel (implemented)
    assert_eq!(apu.triangle.linear_counter.reload_value, 0x03);

    // Verify noise channel (implemented)
    assert_eq!(apu.noise.envelope.period, 0x04);

    // Verify DMC (stub registers)
    assert_eq!(apu.dmc_flags_rate, 0x05);
}

#[test]
fn test_write_does_not_crash() {
    let mut apu = Apu::new();

    // Write to all APU registers
    for addr in 0x4000..=0x4017 {
        apu.write(addr, 0xFF);
    }

    // Should not crash
}

#[test]
fn test_read_does_not_crash() {
    let mut apu = Apu::new();

    // Read from all APU registers
    for addr in 0x4000..=0x4017 {
        let _ = apu.read(addr);
    }

    // Should not crash
}
