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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_new() {
        let env = Envelope::new();
        assert!(!env.start);
        assert_eq!(env.decay_level, 0);
        assert_eq!(env.period, 0);
        assert!(!env.loop_flag);
        assert!(!env.constant_volume);
    }

    #[test]
    fn test_envelope_write_control() {
        let mut env = Envelope::new();

        // Loop flag (bit 5), constant volume (bit 4), period = 7
        env.write_control(0b00110111);

        assert!(env.loop_flag);
        assert!(env.constant_volume);
        assert_eq!(env.period, 7);
    }

    #[test]
    fn test_envelope_restart() {
        let mut env = Envelope::new();

        env.restart();

        assert!(env.start);
    }

    #[test]
    fn test_envelope_volume_constant_mode() {
        let mut env = Envelope::new();

        // Constant volume mode with volume 12
        env.write_control(0b00011100); // Constant=1, Volume=12

        assert_eq!(env.volume(), 12);
    }

    #[test]
    fn test_envelope_volume_decay_mode() {
        let mut env = Envelope::new();

        // Decay mode
        env.write_control(0b00000000); // Constant=0
        env.decay_level = 8;

        assert_eq!(env.volume(), 8);
    }

    #[test]
    fn test_envelope_clock_with_start_flag() {
        let mut env = Envelope::new();

        env.period = 5;
        env.start = true;
        env.decay_level = 0;

        env.clock();

        // After clock with start flag, decay level should be 15
        assert!(!env.start);
        assert_eq!(env.decay_level, 15);
        assert_eq!(env.divider, 5);
    }

    #[test]
    fn test_envelope_clock_divider_countdown() {
        let mut env = Envelope::new();

        env.period = 3;
        env.divider = 2;
        env.start = false;

        env.clock();

        // Divider should decrement
        assert_eq!(env.divider, 1);
    }

    #[test]
    fn test_envelope_clock_decay() {
        let mut env = Envelope::new();

        env.period = 0; // Divider reloads to 0, so decay happens every clock
        env.divider = 0;
        env.decay_level = 10;
        env.start = false;

        env.clock();

        // Decay level should decrement
        assert_eq!(env.decay_level, 9);
        assert_eq!(env.divider, 0);
    }

    #[test]
    fn test_envelope_clock_decay_with_loop() {
        let mut env = Envelope::new();

        env.period = 0;
        env.divider = 0;
        env.decay_level = 0;
        env.loop_flag = true;
        env.start = false;

        env.clock();

        // With loop flag, decay level should wrap to 15
        assert_eq!(env.decay_level, 15);
    }

    #[test]
    fn test_envelope_clock_decay_without_loop() {
        let mut env = Envelope::new();

        env.period = 0;
        env.divider = 0;
        env.decay_level = 0;
        env.loop_flag = false;
        env.start = false;

        env.clock();

        // Without loop flag, decay level stays at 0
        assert_eq!(env.decay_level, 0);
    }

    #[test]
    fn test_envelope_full_decay_sequence() {
        let mut env = Envelope::new();

        // Setup: period=0, no loop, start from 15
        env.write_control(0b00000000); // No loop, no constant, period=0
        env.restart();
        env.clock(); // Processes start flag, sets decay to 15

        assert_eq!(env.decay_level, 15);

        // Clock 15 times to decay to 0
        for i in (0..15).rev() {
            env.clock();
            assert_eq!(env.decay_level, i);
        }

        // One more clock, should stay at 0 (no loop)
        env.clock();
        assert_eq!(env.decay_level, 0);
    }

    #[test]
    fn test_envelope_decay_with_period() {
        let mut env = Envelope::new();

        env.period = 2;
        env.restart();
        env.clock(); // Process start flag

        assert_eq!(env.decay_level, 15);
        assert_eq!(env.divider, 2);

        // Clock twice to count down divider
        env.clock();
        assert_eq!(env.divider, 1);
        assert_eq!(env.decay_level, 15); // No decay yet

        env.clock();
        assert_eq!(env.divider, 0);
        assert_eq!(env.decay_level, 15); // No decay yet

        // Now divider is 0, next clock reloads and decays
        env.clock();
        assert_eq!(env.divider, 2);
        assert_eq!(env.decay_level, 14); // Decayed
    }
}
