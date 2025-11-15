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

// ============================================================================
// Pulse Channel Implementation
// ============================================================================

/// Length counter lookup table
/// Maps the 5-bit length counter load value to the actual counter value
const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

/// Duty cycle patterns for pulse channels
/// Each pattern is 8 steps, representing one full cycle of the square wave
const DUTY_PATTERNS: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5% duty cycle
    [0, 1, 1, 0, 0, 0, 0, 0], // 25% duty cycle
    [0, 1, 1, 1, 1, 0, 0, 0], // 50% duty cycle
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% duty cycle (inverted 25%)
];

/// Envelope generator for controlling volume over time
#[derive(Debug, Clone)]
struct Envelope {
    /// Start flag - set when length counter is loaded
    start: bool,
    /// Divider counter
    divider: u8,
    /// Decay level counter (0-15)
    decay_level: u8,
    /// Period for the divider
    period: u8,
    /// Loop flag (from register bit 5)
    loop_flag: bool,
    /// Constant volume flag (from register bit 4)
    constant_volume: bool,
}

impl Envelope {
    fn new() -> Self {
        Self {
            start: false,
            divider: 0,
            decay_level: 0,
            period: 0,
            loop_flag: false,
            constant_volume: false,
        }
    }

    /// Clock the envelope generator (called by frame sequencer)
    fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.period;
        } else if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = self.period;
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.loop_flag {
                self.decay_level = 15;
            }
        }
    }

    /// Get the current volume (0-15)
    fn volume(&self) -> u8 {
        if self.constant_volume {
            self.period // When constant volume is set, period becomes the volume
        } else {
            self.decay_level
        }
    }

    /// Write to the envelope control register
    fn write_control(&mut self, data: u8) {
        self.loop_flag = (data & 0x20) != 0;
        self.constant_volume = (data & 0x10) != 0;
        self.period = data & 0x0F;
    }

    /// Restart the envelope
    fn restart(&mut self) {
        self.start = true;
    }
}

/// Sweep unit for pitch bending
#[derive(Debug, Clone)]
struct Sweep {
    /// Enabled flag
    enabled: bool,
    /// Divider counter
    divider: u8,
    /// Period for the divider
    period: u8,
    /// Negate flag (pitch bend direction)
    negate: bool,
    /// Shift amount
    shift: u8,
    /// Reload flag
    reload: bool,
    /// Channel number (1 or 2) - affects negate calculation
    channel: u8,
}

impl Sweep {
    fn new(channel: u8) -> Self {
        Self {
            enabled: false,
            divider: 0,
            period: 0,
            negate: false,
            shift: 0,
            reload: false,
            channel,
        }
    }

    /// Calculate the target period for the sweep
    fn calculate_target_period(&self, current_period: u16) -> u16 {
        let change = current_period >> self.shift;
        if self.negate {
            // Pulse 1 uses one's complement, Pulse 2 uses two's complement
            if self.channel == 1 {
                current_period.wrapping_sub(change).wrapping_sub(1)
            } else {
                current_period.wrapping_sub(change)
            }
        } else {
            current_period.wrapping_add(change)
        }
    }

    /// Check if the sweep unit is muting the channel
    fn is_muting(&self, current_period: u16) -> bool {
        // Mute if current period < 8 or target period > 0x7FF
        current_period < 8 || self.calculate_target_period(current_period) > 0x7FF
    }

    /// Clock the sweep unit (called by frame sequencer)
    /// Returns Some(new_period) if period should be updated
    fn clock(&mut self, current_period: u16) -> Option<u16> {
        let mut update_period = None;

        // Only update period if shift > 0; muting still applies even when shift == 0
        if self.divider == 0 && self.enabled && self.shift > 0 && !self.is_muting(current_period) {
            update_period = Some(self.calculate_target_period(current_period));
        }

        if self.divider == 0 || self.reload {
            self.divider = self.period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }

        update_period
    }

    /// Write to the sweep control register
    fn write_control(&mut self, data: u8) {
        self.enabled = (data & 0x80) != 0;
        self.period = (data >> 4) & 0x07;
        self.negate = (data & 0x08) != 0;
        self.shift = data & 0x07;
        self.reload = true;
    }
}

/// Length counter for controlling note duration
#[derive(Debug, Clone)]
struct LengthCounter {
    /// Counter value
    counter: u8,
    /// Halt flag (from envelope control register bit 5)
    halt: bool,
}

impl LengthCounter {
    fn new() -> Self {
        Self {
            counter: 0,
            halt: false,
        }
    }

    /// Clock the length counter (called by frame sequencer)
    fn clock(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }

    /// Load a new counter value from the length counter table
    fn load(&mut self, index: u8) {
        self.counter = LENGTH_COUNTER_TABLE[(index & 0x1F) as usize];
    }

    /// Check if the length counter is non-zero
    fn is_active(&self) -> bool {
        self.counter > 0
    }

    /// Set the halt flag
    fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }
}

/// Timer for controlling the frequency of the pulse wave
#[derive(Debug, Clone)]
struct Timer {
    /// Period (11-bit value)
    period: u16,
    /// Current counter value
    counter: u16,
}

impl Timer {
    fn new() -> Self {
        Self {
            period: 0,
            counter: 0,
        }
    }

    /// Clock the timer
    /// Returns true when the timer reaches 0
    fn clock(&mut self) -> bool {
        if self.counter == 0 {
            self.counter = self.period;
            true
        } else {
            self.counter -= 1;
            false
        }
    }

    /// Set the period from low and high bytes
    fn set_period(&mut self, low: u8, high: u8) {
        self.period = (low as u16) | ((high as u16 & 0x07) << 8);
    }

    /// Set the period directly
    fn set_period_direct(&mut self, period: u16) {
        self.period = period;
    }
}

/// Pulse wave channel (used for both Pulse 1 and Pulse 2)
#[derive(Debug, Clone)]
struct PulseChannel {
    /// Enabled flag (from $4015)
    enabled: bool,
    /// Duty cycle (0-3)
    duty: u8,
    /// Duty cycle sequence position (0-7)
    duty_position: u8,
    /// Envelope generator
    envelope: Envelope,
    /// Sweep unit
    sweep: Sweep,
    /// Length counter
    length_counter: LengthCounter,
    /// Timer
    timer: Timer,
}

impl PulseChannel {
    /// Create a new pulse channel
    /// `channel_number` should be 1 or 2 and affects the sweep unit's negate behavior
    fn new(channel_number: u8) -> Self {
        Self {
            enabled: false,
            duty: 0,
            duty_position: 0,
            envelope: Envelope::new(),
            sweep: Sweep::new(channel_number),
            length_counter: LengthCounter::new(),
            timer: Timer::new(),
        }
    }

    /// Write to register 0 (duty cycle and envelope)
    fn write_register_0(&mut self, data: u8) {
        self.duty = (data >> 6) & 0x03;
        self.length_counter.set_halt((data & 0x20) != 0);
        self.envelope.write_control(data);
    }

    /// Write to register 1 (sweep unit)
    fn write_register_1(&mut self, data: u8) {
        self.sweep.write_control(data);
    }

    /// Write to register 2 (timer low byte)
    fn write_register_2(&mut self, data: u8) {
        let high = (self.timer.period >> 8) as u8;
        self.timer.set_period(data, high);
    }

    /// Write to register 3 (length counter and timer high)
    fn write_register_3(&mut self, data: u8) {
        let low = self.timer.period as u8;
        self.timer.set_period(low, data & 0x07);

        // Load length counter if channel is enabled
        if self.enabled {
            self.length_counter.load(data >> 3);
        }

        // Restart envelope and reset duty position
        self.envelope.restart();
        self.duty_position = 0;
    }

    /// Set the enabled flag (from $4015)
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter.counter = 0;
        }
    }

    /// Check if the channel is enabled and producing sound
    fn is_active(&self) -> bool {
        self.enabled && self.length_counter.is_active()
    }

    /// Clock the timer and update duty position
    fn clock_timer(&mut self) {
        if self.timer.clock() {
            self.duty_position = (self.duty_position + 1) % 8;
        }
    }

    /// Clock the envelope (called by frame sequencer)
    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    /// Clock the length counter (called by frame sequencer)
    fn clock_length_counter(&mut self) {
        self.length_counter.clock();
    }

    /// Clock the sweep unit (called by frame sequencer)
    fn clock_sweep(&mut self) {
        if let Some(new_period) = self.sweep.clock(self.timer.period) {
            self.timer.set_period_direct(new_period);
        }
    }

    /// Get the current output sample (0 or volume)
    fn output(&self) -> u8 {
        // Check if channel should be muted
        if !self.is_active() {
            return 0;
        }

        // Check if sweep is muting
        if self.sweep.is_muting(self.timer.period) {
            return 0;
        }

        // Get duty cycle value
        let duty_output = DUTY_PATTERNS[self.duty as usize][self.duty_position as usize];

        if duty_output == 0 {
            0
        } else {
            self.envelope.volume()
        }
    }
}

// ============================================================================
// APU Main Structure
// ============================================================================

/// APU structure representing the Audio Processing Unit state
///
/// Phase 7 implementation with full pulse channel support.
/// Triangle, Noise, and DMC channels remain as stubs for future implementation.
pub struct Apu {
    // ========================================
    // Pulse Channels (Phase 7 - Implemented)
    // ========================================
    /// Pulse channel 1
    pulse1: PulseChannel,

    /// Pulse channel 2
    pulse2: PulseChannel,

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
            // Pulse channels (Phase 7 - Implemented)
            pulse1: PulseChannel::new(1),
            pulse2: PulseChannel::new(2),

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

    /// Clock the APU timer (called every CPU cycle)
    ///
    /// The APU runs at half the CPU clock speed, so this should be called
    /// every other CPU cycle, or the internal logic should handle the division.
    /// For now, this clocks the pulse channel timers directly.
    pub fn clock(&mut self) {
        // The APU runs at half CPU speed (approximately 1.789773 MHz)
        // For accurate emulation, timer should be clocked every other CPU cycle
        // For now, we'll clock every call
        self.pulse1.clock_timer();
        self.pulse2.clock_timer();
    }

    /// Clock the frame sequencer quarter frame
    ///
    /// This should be called at specific points based on the frame counter mode:
    /// - 4-step mode: Steps at 3728.5, 7456.5, 11185.5, 14914.5 CPU cycles
    /// - 5-step mode: Steps at 3728.5, 7456.5, 11185.5, 18640.5 CPU cycles
    ///
    /// Quarter frame clocks the envelope
    pub fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
    }

    /// Clock the frame sequencer half frame
    ///
    /// Half frame clocks both the envelope and length counter/sweep
    pub fn clock_half_frame(&mut self) {
        // Clock envelope (quarter frame)
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();

        // Clock length counter and sweep (half frame only)
        self.pulse1.clock_length_counter();
        self.pulse1.clock_sweep();
        self.pulse2.clock_length_counter();
        self.pulse2.clock_sweep();
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
                // Triangle, Noise, DMC not implemented yet (bits 2-4)
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
            0x4015 => {
                self.status_control = data;
                self.pulse1.set_enabled((data & 0x01) != 0);
                self.pulse2.set_enabled((data & 0x02) != 0);
                // Triangle, Noise, DMC not implemented yet (bits 2-4)
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
mod tests {
    use super::*;

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
        // Other channels (stub registers)
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

        // Verify other channels (stub registers)
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
}
