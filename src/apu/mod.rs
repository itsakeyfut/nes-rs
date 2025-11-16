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

// Module declarations
mod channels;
mod components;
mod constants;

// Re-exports
pub use channels::{NoiseChannel, PulseChannel, TriangleChannel};

// APU Main Structure
// ============================================================================

/// APU structure representing the Audio Processing Unit state
///
/// Phase 7 implementation with full pulse, triangle, and noise channel support.
/// DMC channel remains as stub for future implementation.
pub struct Apu {
    // ========================================
    // Pulse Channels (Phase 7 - Implemented)
    // ========================================
    /// Pulse channel 1
    pub(crate) pulse1: PulseChannel,

    /// Pulse channel 2
    pub(crate) pulse2: PulseChannel,

    // ========================================
    // Triangle Channel (Phase 7 - Implemented)
    // ========================================
    /// Triangle channel
    pub(crate) triangle: TriangleChannel,

    // ========================================
    // Noise Channel (Phase 7 - Implemented)
    // ========================================
    /// Noise channel
    pub(crate) noise: NoiseChannel,

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
            // Pulse channels (Phase 7 - Implemented)
            pulse1: PulseChannel::new(1),
            pulse2: PulseChannel::new(2),

            // Triangle channel (Phase 7 - Implemented)
            triangle: TriangleChannel::new(),

            // Noise channel (Phase 7 - Implemented)
            noise: NoiseChannel::new(),

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

    /// Clock the APU timer (called every CPU cycle)
    ///
    /// The APU runs at half the CPU clock speed, so this should be called
    /// every other CPU cycle, or the internal logic should handle the division.
    /// For now, this clocks the pulse and triangle channel timers directly.
    pub fn clock(&mut self) {
        // The APU runs at half CPU speed (approximately 1.789773 MHz)
        // For accurate emulation, timer should be clocked every other CPU cycle
        // For now, we'll clock every call
        self.pulse1.clock_timer();
        self.pulse2.clock_timer();
        self.triangle.clock_timer();
        self.noise.clock_timer();
    }

    /// Clock the frame sequencer quarter frame
    ///
    /// This should be called at specific points based on the frame counter mode:
    /// - 4-step mode: Steps at 3728.5, 7456.5, 11185.5, 14914.5 CPU cycles
    /// - 5-step mode: Steps at 3728.5, 7456.5, 11185.5, 18640.5 CPU cycles
    ///
    /// Quarter frame clocks the envelope and linear counter
    pub fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    /// Clock the frame sequencer half frame
    ///
    /// Half frame clocks both the envelope and length counter/sweep
    pub fn clock_half_frame(&mut self) {
        // Clock envelope and linear counter (quarter frame)
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();

        // Clock length counter and sweep (half frame only)
        self.pulse1.clock_length_counter();
        self.pulse1.clock_sweep();
        self.pulse2.clock_length_counter();
        self.pulse2.clock_sweep();
        self.triangle.clock_length_counter();
        self.noise.clock_length_counter();
    }

    /// Get the mixed output sample from all channels
    ///
    /// Returns a value representing the mixed audio output.
    /// The NES uses a non-linear mixing formula for pulse channels:
    /// pulse_out = 95.88 / ((8128 / (pulse1 + pulse2)) + 100)
    ///
    /// For now, this returns a simple digital mix (0-30 range).
    pub fn output(&self) -> u8 {
        let pulse1_out = self.pulse1.output();
        let pulse2_out = self.pulse2.output();

        // Simple linear mix for now
        // In a full implementation, use the non-linear mixing formula
        pulse1_out.saturating_add(pulse2_out)
    }

    /// Get the output from pulse channel 1
    pub fn pulse1_output(&self) -> u8 {
        self.pulse1.output()
    }

    /// Get the output from pulse channel 2
    pub fn pulse2_output(&self) -> u8 {
        self.pulse2.output()
    }

    /// Get the output from triangle channel
    pub fn triangle_output(&self) -> u8 {
        self.triangle.output()
    }

    /// Get the output from noise channel
    pub fn noise_output(&self) -> u8 {
        self.noise.output()
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
            0x4015 => {
                let mut status = 0u8;
                if self.pulse1.length_counter.is_active() {
                    status |= 0x01;
                }
                if self.pulse2.length_counter.is_active() {
                    status |= 0x02;
                }
                if self.triangle.length_counter.is_active() {
                    status |= 0x04;
                }
                if self.noise.length_counter.is_active() {
                    status |= 0x08;
                }
                // DMC not implemented yet (bit 4)
                // Frame interrupt and DMC interrupt flags not implemented (bits 6-7)
                status
            }

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
            0x4000 => self.pulse1.write_register_0(data),
            0x4001 => self.pulse1.write_register_1(data),
            0x4002 => self.pulse1.write_register_2(data),
            0x4003 => self.pulse1.write_register_3(data),

            // Pulse 2 ($4004-$4007)
            0x4004 => self.pulse2.write_register_0(data),
            0x4005 => self.pulse2.write_register_1(data),
            0x4006 => self.pulse2.write_register_2(data),
            0x4007 => self.pulse2.write_register_3(data),

            // Triangle ($4008-$400B)
            0x4008 => self.triangle.write_register_0(data),
            0x4009 => self.triangle.write_register_1(data),
            0x400A => self.triangle.write_register_2(data),
            0x400B => self.triangle.write_register_3(data),

            // Noise ($400C-$400F)
            0x400C => self.noise.write_register_0(data),
            0x400D => self.noise.write_register_1(data),
            0x400E => self.noise.write_register_2(data),
            0x400F => self.noise.write_register_3(data),

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
            0x4015 => {
                self.status_control = data;
                self.pulse1.set_enabled((data & 0x01) != 0);
                self.pulse2.set_enabled((data & 0x02) != 0);
                self.triangle.set_enabled((data & 0x04) != 0);
                self.noise.set_enabled((data & 0x08) != 0);
                // DMC not implemented yet (bit 4)
            }

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
mod tests;
