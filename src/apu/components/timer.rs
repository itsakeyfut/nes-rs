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
