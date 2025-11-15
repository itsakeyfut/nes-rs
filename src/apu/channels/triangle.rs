//! Triangle wave channel implementation

use crate::apu::components::{LengthCounter, LinearCounter, Timer};
use crate::apu::constants::TRIANGLE_SEQUENCE;

/// Triangle wave channel for bass and melody sounds
#[derive(Debug, Clone)]
pub struct TriangleChannel {
    /// Enabled flag (from $4015)
    pub(crate) enabled: bool,
    /// Linear counter
    pub(crate) linear_counter: LinearCounter,
    /// Length counter
    pub(crate) length_counter: LengthCounter,
    /// Timer
    pub(crate) timer: Timer,
    /// Sequencer position (0-31)
    pub(crate) sequence_position: u8,
}

impl Default for TriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl TriangleChannel {
    /// Create a new triangle channel
    pub fn new() -> Self {
        Self {
            enabled: false,
            linear_counter: LinearCounter::new(),
            length_counter: LengthCounter::new(),
            timer: Timer::new(),
            sequence_position: 0,
        }
    }

    /// Write to register 0 ($4008 - linear counter setup)
    pub fn write_register_0(&mut self, data: u8) {
        // Bit 7: Control flag (also doubles as length counter halt)
        self.length_counter.set_halt((data & 0x80) != 0);
        self.linear_counter.write_control(data);
    }

    /// Write to register 1 ($4009 - unused)
    pub fn write_register_1(&mut self, _data: u8) {
        // Unused register, do nothing
    }

    /// Write to register 2 ($400A - timer low byte)
    pub fn write_register_2(&mut self, data: u8) {
        let high = (self.timer.period >> 8) as u8;
        self.timer.set_period(data, high);
    }

    /// Write to register 3 ($400B - length counter load and timer high)
    pub fn write_register_3(&mut self, data: u8) {
        let low = self.timer.period as u8;
        self.timer.set_period(low, data & 0x07);

        // Always load length counter; $4015 controls whether it is cleared
        self.length_counter.load(data >> 3);

        // Set linear counter reload flag
        self.linear_counter.set_reload_flag();
    }

    /// Set the enabled flag (from $4015)
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter.counter = 0;
        }
    }

    /// Check if the channel is enabled and producing sound
    /// Triangle channel requires both linear counter and length counter to be active
    pub fn is_active(&self) -> bool {
        self.enabled && self.linear_counter.is_active() && self.length_counter.is_active()
    }

    /// Clock the timer and update sequence position
    pub fn clock_timer(&mut self) {
        // Triangle channel only advances sequencer when both counters are non-zero
        if self.linear_counter.is_active() && self.length_counter.is_active() && self.timer.clock()
        {
            self.sequence_position = (self.sequence_position + 1) % 32;
        }
    }

    /// Clock the linear counter (called by frame sequencer quarter frame)
    pub fn clock_linear_counter(&mut self) {
        self.linear_counter.clock();
    }

    /// Clock the length counter (called by frame sequencer half frame)
    pub fn clock_length_counter(&mut self) {
        self.length_counter.clock();
    }

    /// Get the current output sample (0-15)
    /// Implements ultrasonic silencing (mute if timer period < 2)
    pub fn output(&self) -> u8 {
        // Check if channel should be muted
        if !self.enabled {
            return 0;
        }

        // Check if either counter is zero
        if !self.linear_counter.is_active() || !self.length_counter.is_active() {
            return 0;
        }

        // Ultrasonic silencing: mute if timer period < 2
        // This prevents clicking at very high frequencies
        if self.timer.period < 2 {
            return 0;
        }

        // Return current position in triangle sequence
        TRIANGLE_SEQUENCE[self.sequence_position as usize]
    }
}
