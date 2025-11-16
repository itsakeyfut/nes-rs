//! Linear counter for the triangle channel

/// Linear counter for the triangle channel
/// The linear counter gates the length counter and provides an additional
/// mechanism for controlling note duration
#[derive(Debug, Clone)]
pub struct LinearCounter {
    /// Counter value
    pub(crate) counter: u8,
    /// Reload value (from register bits 6-0)
    pub(crate) reload_value: u8,
    /// Control flag (from register bit 7)
    /// When set, the linear counter is reloaded every clock
    pub(crate) control_flag: bool,
    /// Reload flag - set when register 3 is written
    pub(crate) reload_flag: bool,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            reload_value: 0,
            control_flag: false,
            reload_flag: false,
        }
    }

    /// Clock the linear counter (called by frame sequencer quarter frame)
    pub fn clock(&mut self) {
        if self.reload_flag {
            self.counter = self.reload_value;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !self.control_flag {
            self.reload_flag = false;
        }
    }

    /// Check if the linear counter is non-zero
    pub fn is_active(&self) -> bool {
        self.counter > 0
    }

    /// Write to the linear counter control register
    pub fn write_control(&mut self, data: u8) {
        self.control_flag = (data & 0x80) != 0;
        self.reload_value = data & 0x7F;
    }

    /// Set the reload flag (when register 3 is written)
    pub fn set_reload_flag(&mut self) {
        self.reload_flag = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_counter_new() {
        let lc = LinearCounter::new();
        assert_eq!(lc.counter, 0);
        assert_eq!(lc.reload_value, 0);
        assert!(!lc.control_flag);
        assert!(!lc.reload_flag);
    }

    #[test]
    fn test_linear_counter_write_control() {
        let mut lc = LinearCounter::new();

        // Control flag (bit 7), reload value = 0x3F
        lc.write_control(0b10111111);

        assert!(lc.control_flag);
        assert_eq!(lc.reload_value, 0x3F);
    }

    #[test]
    fn test_linear_counter_set_reload_flag() {
        let mut lc = LinearCounter::new();

        lc.set_reload_flag();
        assert!(lc.reload_flag);
    }

    #[test]
    fn test_linear_counter_is_active() {
        let mut lc = LinearCounter::new();

        assert!(!lc.is_active());

        lc.counter = 1;
        assert!(lc.is_active());

        lc.counter = 0;
        assert!(!lc.is_active());
    }

    #[test]
    fn test_linear_counter_clock_with_reload_flag() {
        let mut lc = LinearCounter::new();

        lc.reload_value = 10;
        lc.reload_flag = true;
        lc.counter = 0;

        lc.clock();

        // Counter should be reloaded
        assert_eq!(lc.counter, 10);
    }

    #[test]
    fn test_linear_counter_clock_decrements() {
        let mut lc = LinearCounter::new();

        lc.reload_flag = false;
        lc.counter = 5;

        lc.clock();
        assert_eq!(lc.counter, 4);

        lc.clock();
        assert_eq!(lc.counter, 3);
    }

    #[test]
    fn test_linear_counter_clock_at_zero() {
        let mut lc = LinearCounter::new();

        lc.reload_flag = false;
        lc.counter = 0;

        lc.clock();
        assert_eq!(lc.counter, 0); // Should stay at zero
    }

    #[test]
    fn test_linear_counter_clock_clears_reload_flag() {
        let mut lc = LinearCounter::new();

        lc.reload_flag = true;
        lc.control_flag = false;
        lc.reload_value = 5;

        lc.clock();

        // Reload flag should be cleared when control flag is not set
        assert!(!lc.reload_flag);
    }

    #[test]
    fn test_linear_counter_clock_keeps_reload_flag_with_control() {
        let mut lc = LinearCounter::new();

        lc.reload_flag = true;
        lc.control_flag = true;
        lc.reload_value = 5;

        lc.clock();

        // Reload flag should remain when control flag is set
        assert!(lc.reload_flag);
    }
}
