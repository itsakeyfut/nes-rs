//! Pulse wave channel implementation

use crate::apu::components::{Envelope, LengthCounter, Sweep, Timer};
use crate::apu::constants::DUTY_PATTERNS;

/// Pulse wave channel (used for both Pulse 1 and Pulse 2)
#[derive(Debug, Clone)]
pub struct PulseChannel {
    /// Enabled flag (from $4015)
    pub(crate) enabled: bool,
    /// Duty cycle (0-3)
    pub(crate) duty: u8,
    /// Duty cycle sequence position (0-7)
    duty_position: u8,
    /// Envelope generator
    pub(crate) envelope: Envelope,
    /// Sweep unit
    pub(crate) sweep: Sweep,
    /// Length counter
    pub(crate) length_counter: LengthCounter,
    /// Timer
    pub(crate) timer: Timer,
}

impl PulseChannel {
    /// Create a new pulse channel
    /// `channel_number` should be 1 or 2 and affects the sweep unit's negate behavior
    pub fn new(channel_number: u8) -> Self {
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
    pub fn write_register_0(&mut self, data: u8) {
        self.duty = (data >> 6) & 0x03;
        self.length_counter.set_halt((data & 0x20) != 0);
        self.envelope.write_control(data);
    }

    /// Write to register 1 (sweep unit)
    pub fn write_register_1(&mut self, data: u8) {
        self.sweep.write_control(data);
    }

    /// Write to register 2 (timer low byte)
    pub fn write_register_2(&mut self, data: u8) {
        let high = (self.timer.period >> 8) as u8;
        self.timer.set_period(data, high);
    }

    /// Write to register 3 (length counter and timer high)
    pub fn write_register_3(&mut self, data: u8) {
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
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter.counter = 0;
        }
    }

    /// Check if the channel is enabled and producing sound
    pub fn is_active(&self) -> bool {
        self.enabled && self.length_counter.is_active()
    }

    /// Clock the timer and update duty position
    pub fn clock_timer(&mut self) {
        if self.timer.clock() {
            self.duty_position = (self.duty_position + 1) % 8;
        }
    }

    /// Clock the envelope (called by frame sequencer)
    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    /// Clock the length counter (called by frame sequencer)
    pub fn clock_length_counter(&mut self) {
        self.length_counter.clock();
    }

    /// Clock the sweep unit (called by frame sequencer)
    pub fn clock_sweep(&mut self) {
        if let Some(new_period) = self.sweep.clock(self.timer.period) {
            self.timer.set_period_direct(new_period);
        }
    }

    /// Get the current output sample (0 or volume)
    pub fn output(&self) -> u8 {
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
