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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_counter_new() {
        let lc = LengthCounter::new();
        assert_eq!(lc.counter, 0);
        assert!(!lc.halt);
    }

    #[test]
    fn test_length_counter_set_halt() {
        let mut lc = LengthCounter::new();

        lc.set_halt(true);
        assert!(lc.halt);

        lc.set_halt(false);
        assert!(!lc.halt);
    }

    #[test]
    fn test_length_counter_load() {
        let mut lc = LengthCounter::new();

        // Load index 0
        lc.load(0);
        assert_eq!(lc.counter, LENGTH_COUNTER_TABLE[0]);

        // Load index 31 (max valid index)
        lc.load(31);
        assert_eq!(lc.counter, LENGTH_COUNTER_TABLE[31]);

        // Load with upper bits set (should be masked)
        lc.load(0xFF);
        assert_eq!(lc.counter, LENGTH_COUNTER_TABLE[31]);
    }

    #[test]
    fn test_length_counter_is_active() {
        let mut lc = LengthCounter::new();

        assert!(!lc.is_active());

        lc.counter = 1;
        assert!(lc.is_active());

        lc.counter = 0;
        assert!(!lc.is_active());
    }

    #[test]
    fn test_length_counter_clock_decrements() {
        let mut lc = LengthCounter::new();

        lc.counter = 5;
        lc.halt = false;

        lc.clock();
        assert_eq!(lc.counter, 4);

        lc.clock();
        assert_eq!(lc.counter, 3);
    }

    #[test]
    fn test_length_counter_clock_with_halt() {
        let mut lc = LengthCounter::new();

        lc.counter = 5;
        lc.halt = true;

        lc.clock();
        assert_eq!(lc.counter, 5); // Should not decrement

        lc.clock();
        assert_eq!(lc.counter, 5); // Still not decrementing
    }

    #[test]
    fn test_length_counter_clock_at_zero() {
        let mut lc = LengthCounter::new();

        lc.counter = 0;
        lc.halt = false;

        lc.clock();
        assert_eq!(lc.counter, 0); // Should stay at zero
    }
}
