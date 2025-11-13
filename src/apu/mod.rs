// APU module - Audio Processing Unit implementation
//
// This module contains the APU emulation for the NES (Ricoh 2A03).
//
// # APU Registers (Phase 2 - Stub Implementation)
//
// The APU has multiple registers mapped at $4000-$4017 in CPU memory space.
//
// This is a stub implementation for Phase 2. Full APU functionality will be
// implemented in Phase 7.
//
// ## Register Map
//
// ### Pulse 1 ($4000-$4003)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $4000   | Duty cycle, envelope                  |
// | $4001   | Sweep unit                            |
// | $4002   | Timer low byte                        |
// | $4003   | Length counter, timer high bits       |
//
// ### Pulse 2 ($4004-$4007)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $4004   | Duty cycle, envelope                  |
// | $4005   | Sweep unit                            |
// | $4006   | Timer low byte                        |
// | $4007   | Length counter, timer high bits       |
//
// ### Triangle ($4008-$400B)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $4008   | Linear counter                        |
// | $4009   | Unused                                |
// | $400A   | Timer low byte                        |
// | $400B   | Length counter, timer high bits       |
//
// ### Noise ($400C-$400F)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $400C   | Envelope                              |
// | $400D   | Unused                                |
// | $400E   | Mode, period                          |
// | $400F   | Length counter                        |
//
// ### DMC ($4010-$4013)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $4010   | Flags, rate                           |
// | $4011   | Direct load                           |
// | $4012   | Sample address                        |
// | $4013   | Sample length                         |
//
// ### Control ($4015, $4017)
// | Address | Description                           |
// |---------|---------------------------------------|
// | $4015   | Status/Control (R/W)                  |
// | $4017   | Frame counter (W)                     |

use crate::bus::MemoryMappedDevice;

/// APU structure representing the Audio Processing Unit state
///
/// This is a Phase 2 stub implementation. Registers accept writes and return
/// sensible default values on reads. Full APU audio generation will be
/// implemented in Phase 7.
pub struct Apu {
    // ========================================
    // Pulse 1 Registers ($4000-$4003)
    // ========================================
    /// $4000: Pulse 1 - Duty cycle and envelope
    pulse1_duty_envelope: u8,

    /// $4001: Pulse 1 - Sweep unit
    pulse1_sweep: u8,

    /// $4002: Pulse 1 - Timer low byte
    pulse1_timer_low: u8,

    /// $4003: Pulse 1 - Length counter and timer high bits
    pulse1_length_timer_high: u8,

    // ========================================
    // Pulse 2 Registers ($4004-$4007)
    // ========================================
    /// $4004: Pulse 2 - Duty cycle and envelope
    pulse2_duty_envelope: u8,

    /// $4005: Pulse 2 - Sweep unit
    pulse2_sweep: u8,

    /// $4006: Pulse 2 - Timer low byte
    pulse2_timer_low: u8,

    /// $4007: Pulse 2 - Length counter and timer high bits
    pulse2_length_timer_high: u8,

    // ========================================
    // Triangle Registers ($4008-$400B)
    // ========================================
    /// $4008: Triangle - Linear counter
    triangle_linear_counter: u8,

    /// $4009: Triangle - Unused
    triangle_unused: u8,

    /// $400A: Triangle - Timer low byte
    triangle_timer_low: u8,

    /// $400B: Triangle - Length counter and timer high bits
    triangle_length_timer_high: u8,

    // ========================================
    // Noise Registers ($400C-$400F)
    // ========================================
    /// $400C: Noise - Envelope
    noise_envelope: u8,

    /// $400D: Noise - Unused
    noise_unused: u8,

    /// $400E: Noise - Mode and period
    noise_mode_period: u8,

    /// $400F: Noise - Length counter
    noise_length_counter: u8,

    // ========================================
    // DMC Registers ($4010-$4013)
    // ========================================
    /// $4010: DMC - Flags and rate
    dmc_flags_rate: u8,

    /// $4011: DMC - Direct load
    dmc_direct_load: u8,

    /// $4012: DMC - Sample address
    dmc_sample_address: u8,

    /// $4013: DMC - Sample length
    dmc_sample_length: u8,

    // ========================================
    // Control Registers ($4015, $4017)
    // ========================================
    /// $4015: Status/Control - Enable/disable channels
    ///
    /// Read: Status of each channel (length counter > 0)
    /// Write: Enable/disable channels
    status_control: u8,

    /// $4017: Frame Counter - Controls the frame sequencer
    frame_counter: u8,
}

impl Apu {
    /// Create a new APU instance with default state
    ///
    /// Initializes all registers to their power-on state.
    ///
    /// # Returns
    ///
    /// A new APU instance in its initial state
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::apu::Apu;
    ///
    /// let apu = Apu::new();
    /// ```
    pub fn new() -> Self {
        Apu {
            // Pulse 1
            pulse1_duty_envelope: 0x00,
            pulse1_sweep: 0x00,
            pulse1_timer_low: 0x00,
            pulse1_length_timer_high: 0x00,

            // Pulse 2
            pulse2_duty_envelope: 0x00,
            pulse2_sweep: 0x00,
            pulse2_timer_low: 0x00,
            pulse2_length_timer_high: 0x00,

            // Triangle
            triangle_linear_counter: 0x00,
            triangle_unused: 0x00,
            triangle_timer_low: 0x00,
            triangle_length_timer_high: 0x00,

            // Noise
            noise_envelope: 0x00,
            noise_unused: 0x00,
            noise_mode_period: 0x00,
            noise_length_counter: 0x00,

            // DMC
            dmc_flags_rate: 0x00,
            dmc_direct_load: 0x00,
            dmc_sample_address: 0x00,
            dmc_sample_length: 0x00,

            // Control
            status_control: 0x00,
            frame_counter: 0x00,
        }
    }

    /// Reset APU to power-on state
    ///
    /// Resets all registers to their default values.
    /// This simulates a power cycle or reset signal.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Read from an APU register
    ///
    /// # Arguments
    ///
    /// * `addr` - The register address ($4000-$4017)
    ///
    /// # Returns
    ///
    /// The value read from the register
    ///
    /// # Register Behaviors
    ///
    /// - $4015: Returns channel status (stub: returns 0)
    /// - All other registers: Write-only, return 0
    fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            // Pulse 1 ($4000-$4003) - Write only
            0x4000..=0x4003 => 0,

            // Pulse 2 ($4004-$4007) - Write only
            0x4004..=0x4007 => 0,

            // Triangle ($4008-$400B) - Write only
            0x4008..=0x400B => 0,

            // Noise ($400C-$400F) - Write only
            0x400C..=0x400F => 0,

            // DMC ($4010-$4013) - Write only (except $4011 which is not readable)
            0x4010..=0x4013 => 0,

            // $4014: OAM DMA - Not part of APU, handled by bus
            0x4014 => 0,

            // $4015: Status/Control - Read returns channel status
            // Bit 0: Pulse 1 length counter > 0
            // Bit 1: Pulse 2 length counter > 0
            // Bit 2: Triangle length counter > 0
            // Bit 3: Noise length counter > 0
            // Bit 4: DMC active
            // Bit 5: Unused
            // Bit 6: Frame interrupt flag
            // Bit 7: DMC interrupt flag
            // Stub: return 0 (all channels disabled)
            0x4015 => 0,

            // $4016: Controller 1 - Not part of APU, handled separately
            0x4016 => 0,

            // $4017: Frame Counter / Controller 2 - Write only for frame counter
            0x4017 => 0,

            _ => 0,
        }
    }

    /// Write to an APU register
    ///
    /// # Arguments
    ///
    /// * `addr` - The register address ($4000-$4017)
    /// * `data` - The value to write
    ///
    /// # Register Behaviors
    ///
    /// All registers accept writes and store the values.
    /// Audio generation is not implemented in this stub.
    fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            // Pulse 1 ($4000-$4003)
            0x4000 => self.pulse1_duty_envelope = data,
            0x4001 => self.pulse1_sweep = data,
            0x4002 => self.pulse1_timer_low = data,
            0x4003 => self.pulse1_length_timer_high = data,

            // Pulse 2 ($4004-$4007)
            0x4004 => self.pulse2_duty_envelope = data,
            0x4005 => self.pulse2_sweep = data,
            0x4006 => self.pulse2_timer_low = data,
            0x4007 => self.pulse2_length_timer_high = data,

            // Triangle ($4008-$400B)
            0x4008 => self.triangle_linear_counter = data,
            0x4009 => self.triangle_unused = data,
            0x400A => self.triangle_timer_low = data,
            0x400B => self.triangle_length_timer_high = data,

            // Noise ($400C-$400F)
            0x400C => self.noise_envelope = data,
            0x400D => self.noise_unused = data,
            0x400E => self.noise_mode_period = data,
            0x400F => self.noise_length_counter = data,

            // DMC ($4010-$4013)
            0x4010 => self.dmc_flags_rate = data,
            0x4011 => self.dmc_direct_load = data,
            0x4012 => self.dmc_sample_address = data,
            0x4013 => self.dmc_sample_length = data,

            // $4014: OAM DMA - Not part of APU, handled by bus
            0x4014 => {}

            // $4015: Status/Control - Enable/disable channels
            // Bit 0: Enable Pulse 1
            // Bit 1: Enable Pulse 2
            // Bit 2: Enable Triangle
            // Bit 3: Enable Noise
            // Bit 4: Enable DMC
            0x4015 => self.status_control = data,

            // $4016: Controller 1 - Not part of APU, handled separately
            0x4016 => {}

            // $4017: Frame Counter
            // Bit 6: IRQ inhibit flag
            // Bit 7: Sequencer mode (0 = 4-step, 1 = 5-step)
            0x4017 => self.frame_counter = data,

            _ => {}
        }
    }
}

impl MemoryMappedDevice for Apu {
    /// Read a byte from APU registers
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from ($4000-$4017)
    ///
    /// # Returns
    ///
    /// The byte value from the specified register
    fn read(&mut self, addr: u16) -> u8 {
        self.read_register(addr)
    }

    /// Write a byte to APU registers
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to ($4000-$4017)
    /// * `data` - The byte value to write
    fn write(&mut self, addr: u16, data: u8) {
        self.write_register(addr, data);
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Initialization Tests
    // ========================================

    #[test]
    fn test_apu_initialization() {
        let apu = Apu::new();
        assert_eq!(apu.pulse1_duty_envelope, 0x00);
        assert_eq!(apu.pulse2_duty_envelope, 0x00);
        assert_eq!(apu.triangle_linear_counter, 0x00);
        assert_eq!(apu.noise_envelope, 0x00);
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
        apu.write(0x4000, 0x80);
        apu.write(0x4015, 0x0F);

        apu.reset();

        assert_eq!(apu.pulse1_duty_envelope, 0x00);
        assert_eq!(apu.status_control, 0x00);
    }

    // ========================================
    // Pulse 1 Register Tests ($4000-$4003)
    // ========================================

    #[test]
    fn test_write_pulse1_registers() {
        let mut apu = Apu::new();
        apu.write(0x4000, 0xBF);
        apu.write(0x4001, 0x08);
        apu.write(0x4002, 0xA9);
        apu.write(0x4003, 0x0F);

        assert_eq!(apu.pulse1_duty_envelope, 0xBF);
        assert_eq!(apu.pulse1_sweep, 0x08);
        assert_eq!(apu.pulse1_timer_low, 0xA9);
        assert_eq!(apu.pulse1_length_timer_high, 0x0F);
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
        apu.write(0x4004, 0x80);
        apu.write(0x4005, 0x10);
        apu.write(0x4006, 0x55);
        apu.write(0x4007, 0x20);

        assert_eq!(apu.pulse2_duty_envelope, 0x80);
        assert_eq!(apu.pulse2_sweep, 0x10);
        assert_eq!(apu.pulse2_timer_low, 0x55);
        assert_eq!(apu.pulse2_length_timer_high, 0x20);
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
        apu.write(0x4008, 0x81);
        apu.write(0x4009, 0x00);
        apu.write(0x400A, 0xDD);
        apu.write(0x400B, 0x18);

        assert_eq!(apu.triangle_linear_counter, 0x81);
        assert_eq!(apu.triangle_unused, 0x00);
        assert_eq!(apu.triangle_timer_low, 0xDD);
        assert_eq!(apu.triangle_length_timer_high, 0x18);
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
        apu.write(0x400C, 0x30);
        apu.write(0x400D, 0x00);
        apu.write(0x400E, 0x07);
        apu.write(0x400F, 0x10);

        assert_eq!(apu.noise_envelope, 0x30);
        assert_eq!(apu.noise_unused, 0x00);
        assert_eq!(apu.noise_mode_period, 0x07);
        assert_eq!(apu.noise_length_counter, 0x10);
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
        apu.write(0x4015, 0x0F);

        // Stub: returns 0 (no channels active)
        assert_eq!(apu.read(0x4015), 0x00);
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

        // Configure Pulse 1 for a tone
        apu.write(0x4000, 0xBF); // Duty, envelope
        apu.write(0x4001, 0x08); // Sweep
        apu.write(0x4002, 0xA9); // Timer low
        apu.write(0x4003, 0x00); // Timer high
        apu.write(0x4015, 0x01); // Enable Pulse 1

        assert_eq!(apu.pulse1_duty_envelope, 0xBF);
        assert_eq!(apu.status_control, 0x01);
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

        assert_eq!(apu.pulse1_duty_envelope, 0x01);
        assert_eq!(apu.pulse2_duty_envelope, 0x02);
        assert_eq!(apu.triangle_linear_counter, 0x03);
        assert_eq!(apu.noise_envelope, 0x04);
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
}
