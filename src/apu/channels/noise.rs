//! Noise channel implementation for percussion and sound effects

use crate::apu::components::{Envelope, LengthCounter, Timer};
use crate::apu::constants::NOISE_PERIOD_TABLE;

/// Noise channel for percussion and sound effects
#[derive(Debug, Clone)]
pub struct NoiseChannel {
    /// Enabled flag (from $4015)
    pub(crate) enabled: bool,
    /// Envelope generator
    pub(crate) envelope: Envelope,
    /// Length counter
    pub(crate) length_counter: LengthCounter,
    /// Timer
    pub(crate) timer: Timer,
    /// Linear Feedback Shift Register (15-bit)
    pub(crate) lfsr: u16,
    /// Mode flag (false = mode 0, true = mode 1)
    pub(crate) mode: bool,
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseChannel {
    /// Create a new noise channel
    pub fn new() -> Self {
        Self {
            enabled: false,
            envelope: Envelope::new(),
            length_counter: LengthCounter::new(),
            timer: Timer::new(),
            lfsr: 1, // LFSR starts at 1
            mode: false,
        }
    }

    /// Write to register 0 ($400C - envelope)
    pub fn write_register_0(&mut self, data: u8) {
        self.length_counter.set_halt((data & 0x20) != 0);
        self.envelope.write_control(data);
    }

    /// Write to register 1 ($400D - unused)
    pub fn write_register_1(&mut self, _data: u8) {
        // Unused register, do nothing
    }

    /// Write to register 2 ($400E - mode and period)
    pub fn write_register_2(&mut self, data: u8) {
        // Bit 7: Mode flag (0 = mode 0, 1 = mode 1)
        self.mode = (data & 0x80) != 0;
        // Bits 0-3: Period index
        let period_index = (data & 0x0F) as usize;
        let period = NOISE_PERIOD_TABLE[period_index];
        self.timer.set_period_direct(period);
    }

    /// Write to register 3 ($400F - length counter)
    pub fn write_register_3(&mut self, data: u8) {
        // Always load length counter; $4015 controls whether it is cleared
        self.length_counter.load(data >> 3);
        // Restart envelope
        self.envelope.restart();
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

    /// Clock the timer and update LFSR
    pub fn clock_timer(&mut self) {
        if self.timer.clock() {
            // Clock the LFSR
            let feedback = if self.mode {
                // Mode 1: Feedback from bits 0 and 6 (produces white noise)
                ((self.lfsr & 0x01) ^ ((self.lfsr >> 6) & 0x01)) & 0x01
            } else {
                // Mode 0: Feedback from bits 0 and 1 (produces metallic/tonal noise)
                ((self.lfsr & 0x01) ^ ((self.lfsr >> 1) & 0x01)) & 0x01
            };

            // Shift right and insert feedback at bit 14
            self.lfsr >>= 1;
            self.lfsr |= feedback << 14;
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

    /// Get the current output sample (0 or volume)
    pub fn output(&self) -> u8 {
        // Check if channel should be muted
        if !self.is_active() {
            return 0;
        }

        // Output is based on bit 0 of the LFSR
        // If bit 0 is 0, output the envelope volume; otherwise output 0
        if (self.lfsr & 0x01) == 0 {
            self.envelope.volume()
        } else {
            0
        }
    }
}
