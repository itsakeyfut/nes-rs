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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_new() {
        let sweep = Sweep::new(1);
        assert!(!sweep.enabled);
        assert_eq!(sweep.channel, 1);
        assert_eq!(sweep.period, 0);
        assert!(!sweep.negate);
        assert_eq!(sweep.shift, 0);
        assert!(!sweep.reload);
    }

    #[test]
    fn test_sweep_write_control() {
        let mut sweep = Sweep::new(1);

        // Enabled (bit 7), period=3, negate (bit 3), shift=2
        sweep.write_control(0b10111010);

        assert!(sweep.enabled);
        assert_eq!(sweep.period, 3);
        assert!(sweep.negate);
        assert_eq!(sweep.shift, 2);
        assert!(sweep.reload);
    }

    #[test]
    fn test_sweep_calculate_target_period_add() {
        let mut sweep = Sweep::new(1);

        sweep.negate = false;
        sweep.shift = 2;

        // Current period = 100, change = 100 >> 2 = 25
        // Target = 100 + 25 = 125
        assert_eq!(sweep.calculate_target_period(100), 125);
    }

    #[test]
    fn test_sweep_calculate_target_period_negate_channel_1() {
        let mut sweep = Sweep::new(1);

        sweep.negate = true;
        sweep.shift = 2;

        // Channel 1 uses one's complement
        // Current period = 100, change = 100 >> 2 = 25
        // Target = 100 - 25 - 1 = 74
        assert_eq!(sweep.calculate_target_period(100), 74);
    }

    #[test]
    fn test_sweep_calculate_target_period_negate_channel_2() {
        let mut sweep = Sweep::new(2);

        sweep.negate = true;
        sweep.shift = 2;

        // Channel 2 uses two's complement
        // Current period = 100, change = 100 >> 2 = 25
        // Target = 100 - 25 = 75
        assert_eq!(sweep.calculate_target_period(100), 75);
    }

    #[test]
    fn test_sweep_is_muting_period_too_low() {
        let sweep = Sweep::new(1);

        // Period < 8 should mute
        assert!(sweep.is_muting(7));
        assert!(sweep.is_muting(0));
    }

    #[test]
    fn test_sweep_is_muting_target_too_high() {
        let mut sweep = Sweep::new(1);

        sweep.negate = false;
        sweep.shift = 0; // No change, target = current

        // Period that would result in target > 0x7FF
        assert!(sweep.is_muting(0x800));
        assert!(sweep.is_muting(0xFFF));
    }

    #[test]
    fn test_sweep_is_muting_valid_range() {
        let mut sweep = Sweep::new(1);

        sweep.negate = false;
        sweep.shift = 1;

        // Period = 100, change = 50, target = 150 (valid)
        assert!(!sweep.is_muting(100));
    }

    #[test]
    fn test_sweep_clock_disabled() {
        let mut sweep = Sweep::new(1);

        sweep.enabled = false;
        sweep.divider = 0;
        sweep.shift = 1;

        let result = sweep.clock(100);

        // Should not update period when disabled
        assert_eq!(result, None);
    }

    #[test]
    fn test_sweep_clock_shift_zero() {
        let mut sweep = Sweep::new(1);

        sweep.enabled = true;
        sweep.divider = 0;
        sweep.shift = 0; // No shift means no period update

        let result = sweep.clock(100);

        // Should not update period when shift is 0
        assert_eq!(result, None);
    }

    #[test]
    fn test_sweep_clock_muting() {
        let mut sweep = Sweep::new(1);

        sweep.enabled = true;
        sweep.divider = 0;
        sweep.shift = 1;
        sweep.negate = false;

        // Use a period that would cause muting (target > 0x7FF)
        let result = sweep.clock(0x700);

        // Should not update period when muting
        assert_eq!(result, None);
    }

    #[test]
    fn test_sweep_clock_updates_period() {
        let mut sweep = Sweep::new(1);

        sweep.enabled = true;
        sweep.divider = 0;
        sweep.shift = 2;
        sweep.negate = false;

        // Current = 100, change = 25, target = 125
        let result = sweep.clock(100);

        assert_eq!(result, Some(125));
    }

    #[test]
    fn test_sweep_clock_divider_reload() {
        let mut sweep = Sweep::new(1);

        sweep.period = 3;
        sweep.divider = 1;
        sweep.enabled = true;

        sweep.clock(100);

        // Divider should decrement
        assert_eq!(sweep.divider, 0);

        // Next clock should reload divider
        sweep.clock(100);
        assert_eq!(sweep.divider, 3);
    }

    #[test]
    fn test_sweep_clock_reload_flag() {
        let mut sweep = Sweep::new(1);

        sweep.period = 5;
        sweep.divider = 2;
        sweep.reload = true;

        sweep.clock(100);

        // Reload flag should reset divider and clear the flag
        assert_eq!(sweep.divider, 5);
        assert!(!sweep.reload);
    }

    #[test]
    fn test_sweep_different_channels_negate_differently() {
        let mut sweep1 = Sweep::new(1);
        let mut sweep2 = Sweep::new(2);

        sweep1.negate = true;
        sweep2.negate = true;
        sweep1.shift = 1;
        sweep2.shift = 1;

        // Same current period, different channel
        let target1 = sweep1.calculate_target_period(100);
        let target2 = sweep2.calculate_target_period(100);

        // Channel 1: 100 - 50 - 1 = 49
        // Channel 2: 100 - 50 = 50
        assert_eq!(target1, 49);
        assert_eq!(target2, 50);
        assert_ne!(target1, target2);
    }
}
