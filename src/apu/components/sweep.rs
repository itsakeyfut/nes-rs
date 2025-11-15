//! Sweep unit for pitch bending

/// Sweep unit for pitch bending
#[derive(Debug, Clone)]
pub struct Sweep {
    /// Enabled flag
    enabled: bool,
    /// Divider counter
    divider: u8,
    /// Period for the divider
    period: u8,
    /// Negate flag (pitch bend direction)
    negate: bool,
    /// Shift amount
    shift: u8,
    /// Reload flag
    reload: bool,
    /// Channel number (1 or 2) - affects negate calculation
    pub(crate) channel: u8,
}

impl Sweep {
    pub fn new(channel: u8) -> Self {
        Self {
            enabled: false,
            divider: 0,
            period: 0,
            negate: false,
            shift: 0,
            reload: false,
            channel,
        }
    }

    /// Calculate the target period for the sweep
    pub fn calculate_target_period(&self, current_period: u16) -> u16 {
        let change = current_period >> self.shift;
        if self.negate {
            // Pulse 1 uses one's complement, Pulse 2 uses two's complement
            if self.channel == 1 {
                current_period.wrapping_sub(change).wrapping_sub(1)
            } else {
                current_period.wrapping_sub(change)
            }
        } else {
            current_period.wrapping_add(change)
        }
    }

    /// Check if the sweep unit is muting the channel
    pub fn is_muting(&self, current_period: u16) -> bool {
        // Mute if current period < 8 or target period > 0x7FF
        current_period < 8 || self.calculate_target_period(current_period) > 0x7FF
    }

    /// Clock the sweep unit (called by frame sequencer)
    /// Returns Some(new_period) if period should be updated
    pub fn clock(&mut self, current_period: u16) -> Option<u16> {
        let mut update_period = None;

        // Only update period if shift > 0; muting still applies even when shift == 0
        if self.divider == 0 && self.enabled && self.shift > 0 && !self.is_muting(current_period) {
            update_period = Some(self.calculate_target_period(current_period));
        }

        if self.divider == 0 || self.reload {
            self.divider = self.period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }

        update_period
    }

    /// Write to the sweep control register
    pub fn write_control(&mut self, data: u8) {
        self.enabled = (data & 0x80) != 0;
        self.period = (data >> 4) & 0x07;
        self.negate = (data & 0x08) != 0;
        self.shift = data & 0x07;
        self.reload = true;
    }
}
