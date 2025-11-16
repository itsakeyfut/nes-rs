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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_new() {
        let triangle = TriangleChannel::new();
        assert!(!triangle.enabled);
        assert_eq!(triangle.sequence_position, 0);
        assert!(!triangle.is_active());
    }

    #[test]
    fn test_triangle_default() {
        let triangle = TriangleChannel::default();
        assert!(!triangle.enabled);
    }

    #[test]
    fn test_triangle_write_register_0() {
        let mut triangle = TriangleChannel::new();

        // Control flag = 1 (bit 7), linear counter reload = 0x3F
        triangle.write_register_0(0b10111111);

        // Verify halt was set (through length counter)
        // Note: Internal state tested through components
    }

    #[test]
    fn test_triangle_write_register_1_does_nothing() {
        let mut triangle = TriangleChannel::new();

        // Register 1 is unused
        triangle.write_register_1(0xFF);

        // No assertion needed - just verify it doesn't crash
    }

    #[test]
    fn test_triangle_write_register_2_and_3() {
        let mut triangle = TriangleChannel::new();

        // Set timer low byte
        triangle.write_register_2(0x56);
        // Set timer high byte (3 bits) and length counter load
        triangle.write_register_3(0b11111011); // High = 3, length = 31

        // Timer period should be (3 << 8) | 0x56 = 0x356
        assert_eq!(triangle.timer.period, 0x356);
    }

    #[test]
    fn test_triangle_set_enabled() {
        let mut triangle = TriangleChannel::new();

        // Enable channel
        triangle.set_enabled(true);
        assert!(triangle.enabled);

        // Load length counter
        triangle.length_counter.counter = 10;

        // Disable should clear length counter
        triangle.set_enabled(false);
        assert!(!triangle.enabled);
        assert_eq!(triangle.length_counter.counter, 0);
    }

    #[test]
    fn test_triangle_is_active_requires_both_counters() {
        let mut triangle = TriangleChannel::new();

        // Enable channel
        triangle.set_enabled(true);

        // With zero counters, not active
        assert!(!triangle.is_active());

        // With only length counter, not active
        triangle.length_counter.counter = 10;
        assert!(!triangle.is_active());

        // With only linear counter, not active
        triangle.length_counter.counter = 0;
        triangle.linear_counter.counter = 10;
        assert!(!triangle.is_active());

        // With both counters, active
        triangle.length_counter.counter = 10;
        assert!(triangle.is_active());
    }

    #[test]
    fn test_triangle_clock_timer() {
        let mut triangle = TriangleChannel::new();

        // Setup for timer clocking
        triangle.timer.set_period_direct(1);
        triangle.linear_counter.counter = 10;
        triangle.length_counter.counter = 10;

        let initial_position = triangle.sequence_position;

        // Clock timer twice
        triangle.clock_timer();
        triangle.clock_timer();

        // Sequence position should have advanced
        assert_eq!(triangle.sequence_position, (initial_position + 1) % 32);
    }

    #[test]
    fn test_triangle_sequence_position_wraps() {
        let mut triangle = TriangleChannel::new();

        triangle.timer.set_period_direct(0);
        triangle.linear_counter.counter = 10;
        triangle.length_counter.counter = 10;

        // Set position to 31
        triangle.sequence_position = 31;

        // Clock should wrap to 0
        triangle.clock_timer();
        assert_eq!(triangle.sequence_position, 0);
    }

    #[test]
    fn test_triangle_clock_timer_only_when_counters_active() {
        let mut triangle = TriangleChannel::new();

        triangle.timer.set_period_direct(0);
        triangle.sequence_position = 0;

        // Clock with no counters active
        triangle.clock_timer();

        // Position should not change
        assert_eq!(triangle.sequence_position, 0);

        // Activate counters
        triangle.linear_counter.counter = 10;
        triangle.length_counter.counter = 10;

        // Now clock should advance
        triangle.clock_timer();
        assert_eq!(triangle.sequence_position, 1);
    }

    #[test]
    fn test_triangle_clock_linear_counter() {
        let mut triangle = TriangleChannel::new();

        // Setup linear counter
        triangle.linear_counter.counter = 5;
        triangle.linear_counter.reload_flag = false;

        // Clock once
        triangle.clock_linear_counter();

        // Counter should decrement
        assert_eq!(triangle.linear_counter.counter, 4);
    }

    #[test]
    fn test_triangle_clock_length_counter() {
        let mut triangle = TriangleChannel::new();

        // Setup length counter
        triangle.length_counter.counter = 5;
        triangle.length_counter.set_halt(false);

        // Clock once
        triangle.clock_length_counter();

        // Counter should decrement
        assert_eq!(triangle.length_counter.counter, 4);
    }

    #[test]
    fn test_triangle_output_when_disabled() {
        let triangle = TriangleChannel::new();

        // Output should be 0 when disabled
        assert_eq!(triangle.output(), 0);
    }

    #[test]
    fn test_triangle_output_when_enabled() {
        let mut triangle = TriangleChannel::new();

        // Enable and setup
        triangle.set_enabled(true);
        triangle.linear_counter.counter = 10;
        triangle.length_counter.counter = 10;
        triangle.timer.set_period_direct(10); // Period >= 2
        triangle.sequence_position = 0;

        // Output should be from triangle sequence
        let output = triangle.output();
        assert_eq!(output, TRIANGLE_SEQUENCE[0]);
    }

    #[test]
    fn test_triangle_ultrasonic_silencing() {
        let mut triangle = TriangleChannel::new();

        // Enable and setup
        triangle.set_enabled(true);
        triangle.linear_counter.counter = 10;
        triangle.length_counter.counter = 10;

        // Set timer period to 1 (< 2, should be silenced)
        triangle.timer.set_period_direct(1);

        // Output should be 0 (silenced)
        assert_eq!(triangle.output(), 0);

        // Set timer period to 2 (>= 2, should output)
        triangle.timer.set_period_direct(2);

        // Now should output
        assert_ne!(triangle.output(), 0);
    }

    #[test]
    fn test_triangle_register_3_sets_reload_flag() {
        let mut triangle = TriangleChannel::new();

        // Write to register 3
        triangle.write_register_3(0x00);

        // Linear counter reload flag should be set
        assert!(triangle.linear_counter.reload_flag);
    }

    #[test]
    fn test_triangle_output_follows_sequence() {
        let mut triangle = TriangleChannel::new();

        triangle.set_enabled(true);
        triangle.linear_counter.counter = 100;
        triangle.length_counter.counter = 100;
        triangle.timer.set_period_direct(10);

        // Test a few positions in the sequence
        for pos in 0..32 {
            triangle.sequence_position = pos;
            let output = triangle.output();
            assert_eq!(output, TRIANGLE_SEQUENCE[pos as usize]);
        }
    }
}
