//! DMC (Delta Modulation Channel) implementation for sample playback

use crate::apu::components::Timer;
use crate::apu::constants::DMC_RATE_TABLE;

/// DMC channel for sample playback
#[derive(Debug, Clone)]
pub struct DmcChannel {
    /// Enabled flag (from $4015)
    pub(crate) enabled: bool,

    /// IRQ enable flag
    pub(crate) irq_enabled: bool,

    /// Loop flag
    pub(crate) loop_flag: bool,

    /// Rate timer
    pub(crate) timer: Timer,

    /// Sample address ($C000 + address * 64)
    pub(crate) sample_address: u16,

    /// Sample length (length * 16 + 1 bytes)
    pub(crate) sample_length: u16,

    /// Current address being read
    pub(crate) current_address: u16,

    /// Bytes remaining in current sample
    pub(crate) bytes_remaining: u16,

    /// Sample buffer (8 bits)
    pub(crate) sample_buffer: u8,

    /// Sample buffer has data
    pub(crate) sample_buffer_empty: bool,

    /// Shift register (8 bits)
    pub(crate) shift_register: u8,

    /// Bits remaining in shift register
    pub(crate) bits_remaining: u8,

    /// Output level (7-bit counter, 0-127)
    pub(crate) output_level: u8,

    /// Silence flag
    pub(crate) silence_flag: bool,

    /// IRQ pending flag
    pub(crate) irq_flag: bool,
}

impl Default for DmcChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl DmcChannel {
    /// Create a new DMC channel
    pub fn new() -> Self {
        Self {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            timer: Timer::new(),
            sample_address: 0xC000,
            sample_length: 0,
            current_address: 0xC000,
            bytes_remaining: 0,
            sample_buffer: 0,
            sample_buffer_empty: true,
            shift_register: 0,
            bits_remaining: 0,
            output_level: 0,
            silence_flag: true,
            irq_flag: false,
        }
    }

    /// Write to register 0 ($4010 - IRQ enable, loop, and rate)
    /// Bit 7: IRQ enabled flag
    /// Bit 6: Loop flag
    /// Bits 0-3: Rate index
    pub fn write_register_0(&mut self, data: u8) {
        self.irq_enabled = (data & 0x80) != 0;
        self.loop_flag = (data & 0x40) != 0;

        // Clear IRQ flag if IRQ is disabled
        if !self.irq_enabled {
            self.irq_flag = false;
        }

        // Set rate from rate table
        let rate_index = (data & 0x0F) as usize;
        self.timer.set_period_direct(DMC_RATE_TABLE[rate_index]);
    }

    /// Write to register 1 ($4011 - direct load)
    /// Bits 0-6: Direct load value (7-bit)
    pub fn write_register_1(&mut self, data: u8) {
        // Load the 7-bit value directly into output level
        self.output_level = data & 0x7F;
    }

    /// Write to register 2 ($4012 - sample address)
    /// Sample address = $C000 + (value * 64)
    pub fn write_register_2(&mut self, data: u8) {
        self.sample_address = 0xC000 + ((data as u16) << 6);
    }

    /// Write to register 3 ($4013 - sample length)
    /// Sample length = (value * 16) + 1 bytes
    pub fn write_register_3(&mut self, data: u8) {
        self.sample_length = ((data as u16) << 4) + 1;
    }

    /// Set the enabled flag (from $4015)
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            // Start playing sample
            self.restart_sample();
        }
    }

    /// Restart the sample from the beginning
    fn restart_sample(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    /// Check if the channel is active (has bytes remaining)
    pub fn is_active(&self) -> bool {
        self.bytes_remaining > 0
    }

    /// Check if IRQ is pending
    pub fn irq_pending(&self) -> bool {
        self.irq_flag
    }

    /// Clock the timer and output unit
    pub fn clock_timer(&mut self) {
        if !self.timer.clock() {
            return;
        }

        // Timer has reached zero, clock the output unit
        self.clock_output_unit();
    }

    /// Clock the output unit (called when timer reaches zero)
    fn clock_output_unit(&mut self) {
        // If we've consumed all bits, try to reload the shift register
        if self.bits_remaining == 0 {
            if self.sample_buffer_empty {
                self.silence_flag = true;
            } else {
                self.silence_flag = false;
                self.shift_register = self.sample_buffer;
                self.sample_buffer_empty = true;
                self.bits_remaining = 8;
            }
        }

        // Nothing to do if we still have no bits to output
        if self.bits_remaining == 0 {
            return;
        }

        // If silence flag is clear, adjust output level based on current bit
        if !self.silence_flag {
            let bit = self.shift_register & 0x01;

            if bit == 1 {
                // Increment output level if not at maximum
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                // Decrement output level if not at minimum
                self.output_level -= 2;
            }
        }

        // Shift the register right and decrement bits remaining
        self.shift_register >>= 1;
        self.bits_remaining -= 1;
    }

    /// Load a sample byte (called by memory reader)
    /// This should be called when the sample buffer is empty and bytes remain
    pub fn load_sample_byte(&mut self, byte: u8) {
        // Guard against underflow (should not happen with proper usage)
        if self.bytes_remaining == 0 {
            return;
        }

        self.sample_buffer = byte;
        self.sample_buffer_empty = false;

        // Advance to next byte
        // Wrap to $8000 if it goes past $FFFF (wrapping_add handles overflow to $0000, then we check)
        self.current_address = self.current_address.wrapping_add(1);
        if self.current_address == 0x0000 {
            self.current_address = 0x8000;
        }

        self.bytes_remaining -= 1;

        // Check if sample is finished
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart_sample();
            } else if self.irq_enabled {
                self.irq_flag = true;
            }
        }
    }

    /// Check if a sample byte needs to be read
    /// Returns Some(address) if a byte should be read, None otherwise
    pub fn needs_sample_read(&self) -> Option<u16> {
        if self.sample_buffer_empty && self.bytes_remaining > 0 {
            Some(self.current_address)
        } else {
            None
        }
    }

    /// Get the current output level (0-127)
    pub fn output(&self) -> u8 {
        self.output_level
    }
}
