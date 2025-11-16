//! Frame counter for the APU
//!
//! The frame counter is a divider that generates low-frequency clock signals
//! to drive the APU's envelope, sweep, and length counter units.
//!
//! It operates in two modes:
//! - 4-step mode: Generates IRQs and runs at approximately 240 Hz
//! - 5-step mode: No IRQs and runs at approximately 192 Hz

use crate::apu::constants::{
    FRAME_COUNTER_4_STEP_CYCLES, FRAME_COUNTER_4_STEP_PERIOD, FRAME_COUNTER_5_STEP_CYCLES,
    FRAME_COUNTER_5_STEP_PERIOD,
};

/// Events that the frame counter can generate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameEvent {
    /// Quarter frame event - clock envelopes and linear counters
    QuarterFrame,
    /// Half frame event - clock envelopes, linear counters, length counters, and sweep units
    HalfFrame,
    /// Set IRQ flag (only in 4-step mode)
    SetIrq,
}

/// Frame counter sequencer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameMode {
    /// 4-step mode (default) - approximately 240 Hz
    FourStep,
    /// 5-step mode - approximately 192 Hz
    FiveStep,
}

/// Frame counter for clocking APU components
#[derive(Debug, Clone)]
pub struct FrameCounter {
    /// Current mode (4-step or 5-step)
    mode: FrameMode,
    /// Cycle counter within the current frame
    cycle: u32,
    /// Current step in the sequence (0-3 for 4-step, 0-4 for 5-step)
    step: usize,
    /// IRQ inhibit flag (bit 6 of $4017)
    irq_inhibit: bool,
    /// Frame interrupt flag
    irq_pending: bool,
    /// Flag indicating if we need to reset on the next clock
    reset_pending: bool,
    /// Delay counter for $4017 write effects (takes 3-4 CPU cycles)
    write_delay: u8,
}

impl FrameCounter {
    /// Create a new frame counter in 4-step mode
    pub fn new() -> Self {
        Self {
            mode: FrameMode::FourStep,
            cycle: 0,
            step: 0,
            irq_inhibit: false,
            irq_pending: false,
            reset_pending: false,
            write_delay: 0,
        }
    }

    /// Reset the frame counter to its initial state
    pub fn reset(&mut self) {
        self.mode = FrameMode::FourStep;
        self.cycle = 0;
        self.step = 0;
        self.irq_inhibit = false;
        self.irq_pending = false;
        self.reset_pending = false;
        self.write_delay = 0;
    }

    /// Write to the frame counter control register ($4017)
    ///
    /// Bit 7: Mode (0 = 4-step, 1 = 5-step)
    /// Bit 6: IRQ inhibit flag
    ///
    /// # Arguments
    ///
    /// * `value` - The value written to $4017
    ///
    /// # Returns
    ///
    /// Optional frame events that should be triggered immediately
    pub fn write_control(&mut self, value: u8) -> Vec<FrameEvent> {
        let new_mode = if (value & 0x80) != 0 {
            FrameMode::FiveStep
        } else {
            FrameMode::FourStep
        };
        let new_irq_inhibit = (value & 0x40) != 0;

        // If IRQ inhibit is set, clear the IRQ flag
        if new_irq_inhibit {
            self.irq_pending = false;
        }

        self.mode = new_mode;
        self.irq_inhibit = new_irq_inhibit;

        // Writing to $4017 resets the frame counter
        // The actual hardware has a 3-4 cycle delay, but for simplicity
        // we reset immediately to avoid complexity in testing
        self.cycle = 0;
        self.step = 0;
        self.write_delay = 0;
        self.reset_pending = false;

        // In 5-step mode, clock immediately on write
        let mut events = Vec::new();
        if self.mode == FrameMode::FiveStep {
            events.push(FrameEvent::HalfFrame);
        }
        events
    }

    /// Clock the frame counter (called every CPU cycle)
    ///
    /// # Returns
    ///
    /// Optional frame events that should be triggered
    pub fn clock(&mut self) -> Vec<FrameEvent> {
        let mut events = Vec::new();

        // Increment cycle counter
        self.cycle += 1;

        // Check which mode we're in and process accordingly
        match self.mode {
            FrameMode::FourStep => {
                self.clock_4_step(&mut events);
            }
            FrameMode::FiveStep => {
                self.clock_5_step(&mut events);
            }
        }

        events
    }

    /// Clock the 4-step sequencer
    fn clock_4_step(&mut self, events: &mut Vec<FrameEvent>) {
        // Check if we've hit a frame step
        if self.step < 4 && self.cycle == FRAME_COUNTER_4_STEP_CYCLES[self.step] {
            match self.step {
                0 => {
                    // Step 1: Quarter frame
                    events.push(FrameEvent::QuarterFrame);
                }
                1 => {
                    // Step 2: Half frame
                    events.push(FrameEvent::HalfFrame);
                }
                2 => {
                    // Step 3: Quarter frame
                    events.push(FrameEvent::QuarterFrame);
                }
                3 => {
                    // Step 4: Half frame + IRQ
                    events.push(FrameEvent::HalfFrame);
                    // Set IRQ flag if not inhibited
                    if !self.irq_inhibit {
                        self.irq_pending = true;
                        events.push(FrameEvent::SetIrq);
                    }
                }
                _ => {}
            }
            self.step += 1;
        }

        // Reset at end of frame
        if self.cycle >= FRAME_COUNTER_4_STEP_PERIOD {
            // The IRQ flag is also set at cycle 29830 in 4-step mode
            if !self.irq_inhibit {
                self.irq_pending = true;
                events.push(FrameEvent::SetIrq);
            }
            self.cycle = 0;
            self.step = 0;
        }
    }

    /// Clock the 5-step sequencer
    fn clock_5_step(&mut self, events: &mut Vec<FrameEvent>) {
        // Check if we've hit a frame step
        if self.step < 5 && self.cycle == FRAME_COUNTER_5_STEP_CYCLES[self.step] {
            match self.step {
                0 => {
                    // Step 1: Quarter frame
                    events.push(FrameEvent::QuarterFrame);
                }
                1 => {
                    // Step 2: Half frame
                    events.push(FrameEvent::HalfFrame);
                }
                2 => {
                    // Step 3: Quarter frame
                    events.push(FrameEvent::QuarterFrame);
                }
                3 => {
                    // Step 4: Half frame
                    events.push(FrameEvent::HalfFrame);
                }
                4 => {
                    // Step 5: Nothing happens
                }
                _ => {}
            }
            self.step += 1;
        }

        // Reset at end of frame (no IRQ in 5-step mode)
        if self.cycle >= FRAME_COUNTER_5_STEP_PERIOD {
            self.cycle = 0;
            self.step = 0;
        }
    }

    /// Check if there's a pending IRQ
    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    /// Clear the IRQ flag (when $4015 is read)
    pub fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    /// Get the current mode
    pub fn mode(&self) -> FrameMode {
        self.mode
    }

    /// Get the current cycle count
    pub fn cycle(&self) -> u32 {
        self.cycle
    }

    /// Get the current step
    pub fn step(&self) -> usize {
        self.step
    }

    /// Check if IRQ is inhibited
    pub fn irq_inhibited(&self) -> bool {
        self.irq_inhibit
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_counter_init() {
        let fc = FrameCounter::new();
        assert_eq!(fc.mode(), FrameMode::FourStep);
        assert_eq!(fc.cycle(), 0);
        assert_eq!(fc.step(), 0);
        assert!(!fc.irq_pending());
        assert!(!fc.irq_inhibited());
    }

    #[test]
    fn test_4_step_mode() {
        let mut fc = FrameCounter::new();

        // Clock to step 1 (7457 cycles)
        let mut events = Vec::new();
        for _ in 0..7457 {
            events.extend(fc.clock());
        }
        assert!(events.contains(&FrameEvent::QuarterFrame));
        assert_eq!(fc.step(), 1);

        // Clock to step 2 (14913 cycles total)
        events.clear();
        for _ in 0..(14913 - 7457) {
            events.extend(fc.clock());
        }
        assert!(events.contains(&FrameEvent::HalfFrame));
        assert_eq!(fc.step(), 2);

        // Clock to step 3 (22371 cycles total)
        events.clear();
        for _ in 0..(22371 - 14913) {
            events.extend(fc.clock());
        }
        assert!(events.contains(&FrameEvent::QuarterFrame));
        assert_eq!(fc.step(), 3);

        // Clock to step 4 (29829 cycles total)
        events.clear();
        for _ in 0..(29829 - 22371) {
            events.extend(fc.clock());
        }
        assert!(events.contains(&FrameEvent::HalfFrame));
        assert!(events.contains(&FrameEvent::SetIrq));
        assert!(fc.irq_pending());
    }

    #[test]
    fn test_5_step_mode() {
        let mut fc = FrameCounter::new();
        fc.write_control(0x80); // Set 5-step mode

        // Clock to step 1 (7457 cycles)
        let mut events = Vec::new();
        for _ in 0..7457 {
            events.extend(fc.clock());
        }
        assert!(events.contains(&FrameEvent::QuarterFrame));

        // Clock to step 4 (29829 cycles total)
        events.clear();
        for _ in 0..(29829 - 7457) {
            events.extend(fc.clock());
        }
        // Should have half frame, but no IRQ
        assert!(events.contains(&FrameEvent::HalfFrame));
        assert!(!events.contains(&FrameEvent::SetIrq));
        assert!(!fc.irq_pending());
    }

    #[test]
    fn test_irq_inhibit() {
        let mut fc = FrameCounter::new();
        fc.write_control(0x40); // Set IRQ inhibit

        // Clock to step 4
        for _ in 0..29829 {
            fc.clock();
        }

        // IRQ should not be set
        assert!(!fc.irq_pending());
    }

    #[test]
    fn test_irq_clear_on_read() {
        let mut fc = FrameCounter::new();

        // Clock to step 4 to set IRQ
        for _ in 0..29829 {
            fc.clock();
        }

        assert!(fc.irq_pending());
        fc.clear_irq();
        assert!(!fc.irq_pending());
    }

    #[test]
    fn test_mode_switch() {
        let mut fc = FrameCounter::new();
        assert_eq!(fc.mode(), FrameMode::FourStep);

        fc.write_control(0x80);
        assert_eq!(fc.mode(), FrameMode::FiveStep);

        fc.write_control(0x00);
        assert_eq!(fc.mode(), FrameMode::FourStep);
    }
}
