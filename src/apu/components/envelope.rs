//! Envelope generator for controlling volume over time

/// Envelope generator for controlling volume over time
#[derive(Debug, Clone)]
pub struct Envelope {
    /// Start flag - set when length counter is loaded
    pub(crate) start: bool,
    /// Divider counter
    divider: u8,
    /// Decay level counter (0-15)
    pub(crate) decay_level: u8,
    /// Period for the divider
    pub(crate) period: u8,
    /// Loop flag (from register bit 5)
    pub(crate) loop_flag: bool,
    /// Constant volume flag (from register bit 4)
    pub(crate) constant_volume: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            start: false,
            divider: 0,
            decay_level: 0,
            period: 0,
            loop_flag: false,
            constant_volume: false,
        }
    }

    /// Clock the envelope generator (called by frame sequencer)
    pub fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.period;
        } else if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = self.period;
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.loop_flag {
                self.decay_level = 15;
            }
        }
    }

    /// Get the current volume (0-15)
    pub fn volume(&self) -> u8 {
        if self.constant_volume {
            self.period // When constant volume is set, period becomes the volume
        } else {
            self.decay_level
        }
    }

    /// Write to the envelope control register
    pub fn write_control(&mut self, data: u8) {
        self.loop_flag = (data & 0x20) != 0;
        self.constant_volume = (data & 0x10) != 0;
        self.period = data & 0x0F;
    }

    /// Restart the envelope
    pub fn restart(&mut self) {
        self.start = true;
    }
}
