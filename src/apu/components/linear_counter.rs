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
