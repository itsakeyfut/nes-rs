//! Length counter for controlling note duration

use crate::apu::constants::LENGTH_COUNTER_TABLE;

/// Length counter for controlling note duration
#[derive(Debug, Clone)]
pub struct LengthCounter {
    /// Counter value
    pub(crate) counter: u8,
    /// Halt flag (from envelope control register bit 5)
    pub(crate) halt: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            halt: false,
        }
    }

    /// Clock the length counter (called by frame sequencer)
    pub fn clock(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }

    /// Load a new counter value from the length counter table
    pub fn load(&mut self, index: u8) {
        self.counter = LENGTH_COUNTER_TABLE[(index & 0x1F) as usize];
    }

    /// Check if the length counter is non-zero
    pub fn is_active(&self) -> bool {
        self.counter > 0
    }

    /// Set the halt flag
    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }
}
