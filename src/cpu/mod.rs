// CPU module - 6502 processor implementation
// This module will contain the 6502 CPU emulation

/// CPU structure representing the 6502 processor state
pub struct Cpu {
    // Registers
    pub a: u8,      // Accumulator
    pub x: u8,      // Index Register X
    pub y: u8,      // Index Register Y
    pub sp: u8,     // Stack Pointer
    pub pc: u16,    // Program Counter
    pub status: u8, // Processor Status flags
}

impl Cpu {
    /// Create a new CPU instance with default values
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD, // Stack pointer typically starts at 0xFD
            pc: 0,
            status: 0x24, // Default flags: Unused bit set, Interrupt disable set
        }
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0x24;
        // PC will be loaded from reset vector
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.status, 0x24);
    }

    #[test]
    fn test_cpu_reset() {
        let mut cpu = Cpu::new();
        cpu.a = 0x42;
        cpu.x = 0x10;
        cpu.reset();
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
    }
}
