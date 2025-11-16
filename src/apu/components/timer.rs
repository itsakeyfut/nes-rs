//! Timer for controlling the frequency of waveforms

/// Timer for controlling the frequency of the pulse wave
#[derive(Debug, Clone)]
pub struct Timer {
    /// Period (11-bit value)
    pub(crate) period: u16,
    /// Current counter value
    counter: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            period: 0,
            counter: 0,
        }
    }

    /// Clock the timer
    /// Returns true when the timer reaches 0
    pub fn clock(&mut self) -> bool {
        if self.counter == 0 {
            self.counter = self.period;
            true
        } else {
            self.counter -= 1;
            false
        }
    }

    /// Set the period from low and high bytes
    pub fn set_period(&mut self, low: u8, high: u8) {
        self.period = (low as u16) | ((high as u16 & 0x07) << 8);
    }

    /// Set the period directly
    pub fn set_period_direct(&mut self, period: u16) {
        self.period = period;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_new() {
        let timer = Timer::new();
        assert_eq!(timer.period, 0);
        assert_eq!(timer.counter, 0);
    }

    #[test]
    fn test_timer_set_period() {
        let mut timer = Timer::new();

        // Low = 0x34, High = 0x5 (only lower 3 bits used)
        timer.set_period(0x34, 0x5);

        // Period should be (5 << 8) | 0x34 = 0x534
        assert_eq!(timer.period, 0x534);

        // Test with upper bits in high byte
        timer.set_period(0xFF, 0xFF);
        // Only lower 3 bits of high byte should be used: (7 << 8) | 0xFF = 0x7FF
        assert_eq!(timer.period, 0x7FF);
    }

    #[test]
    fn test_timer_set_period_direct() {
        let mut timer = Timer::new();

        timer.set_period_direct(0x123);
        assert_eq!(timer.period, 0x123);
    }

    #[test]
    fn test_timer_clock_returns_true_at_zero() {
        let mut timer = Timer::new();

        timer.period = 5;
        timer.counter = 0;

        let result = timer.clock();

        assert!(result);
        assert_eq!(timer.counter, 5); // Reloaded
    }

    #[test]
    fn test_timer_clock_returns_false_while_counting() {
        let mut timer = Timer::new();

        timer.period = 5;
        timer.counter = 3;

        let result = timer.clock();

        assert!(!result);
        assert_eq!(timer.counter, 2); // Decremented
    }

    #[test]
    fn test_timer_clock_full_cycle() {
        let mut timer = Timer::new();

        timer.period = 3;
        timer.counter = 3;

        // Clock 3 times (3 -> 2 -> 1 -> 0)
        assert!(!timer.clock()); // 3 -> 2
        assert!(!timer.clock()); // 2 -> 1
        assert!(!timer.clock()); // 1 -> 0

        // Next clock should reload and return true
        assert!(timer.clock()); // 0 -> period
        assert_eq!(timer.counter, 3);
    }

    #[test]
    fn test_timer_clock_period_zero() {
        let mut timer = Timer::new();

        timer.period = 0;
        timer.counter = 0;

        // Should reload to 0 every time
        assert!(timer.clock());
        assert_eq!(timer.counter, 0);

        assert!(timer.clock());
        assert_eq!(timer.counter, 0);
    }
}
