// Load and Store instructions for 6502 CPU

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Load Instructions
    // ========================================
    // Load instructions read a value from memory into a register
    // and update the Zero (Z) and Negative (N) flags.

    /// LDA - Load Accumulator
    ///
    /// Loads a byte from memory into the accumulator (A register).
    /// Updates Zero and Negative flags based on the loaded value.
    ///
    /// Flags affected: Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn lda(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.a = value;
        self.update_zero_and_negative_flags(value);
    }

    /// LDX - Load X Register
    ///
    /// Loads a byte from memory into the X register.
    /// Updates Zero and Negative flags based on the loaded value.
    ///
    /// Flags affected: Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn ldx(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.x = value;
        self.update_zero_and_negative_flags(value);
    }

    /// LDY - Load Y Register
    ///
    /// Loads a byte from memory into the Y register.
    /// Updates Zero and Negative flags based on the loaded value.
    ///
    /// Flags affected: Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn ldy(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.y = value;
        self.update_zero_and_negative_flags(value);
    }

    // ========================================
    // Store Instructions
    // ========================================
    // Store instructions write a register value to memory.
    // They do NOT affect any processor flags.

    /// STA - Store Accumulator
    ///
    /// Stores the value from the accumulator (A register) into memory.
    /// Does not affect any flags.
    ///
    /// Flags affected: None
    ///
    /// # Arguments
    /// * `bus` - The memory bus to write to
    /// * `addr_result` - The addressing result containing the memory address
    pub fn sta(&self, bus: &mut Bus, addr_result: &AddressingResult) {
        bus.write(addr_result.address, self.a);
    }

    /// STX - Store X Register
    ///
    /// Stores the value from the X register into memory.
    /// Does not affect any flags.
    ///
    /// Flags affected: None
    ///
    /// # Arguments
    /// * `bus` - The memory bus to write to
    /// * `addr_result` - The addressing result containing the memory address
    pub fn stx(&self, bus: &mut Bus, addr_result: &AddressingResult) {
        bus.write(addr_result.address, self.x);
    }

    /// STY - Store Y Register
    ///
    /// Stores the value from the Y register into memory.
    /// Does not affect any flags.
    ///
    /// Flags affected: None
    ///
    /// # Arguments
    /// * `bus` - The memory bus to write to
    /// * `addr_result` - The addressing result containing the memory address
    pub fn sty(&self, bus: &mut Bus, addr_result: &AddressingResult) {
        bus.write(addr_result.address, self.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;

    // ========================================
    // Load Instruction Tests
    // ========================================

    #[test]
    fn test_lda_immediate() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x42 into accumulator
        let addr_result = AddressingResult::immediate(0x42);
        cpu.lda(&bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "Accumulator should be 0x42");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_lda_zero() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x00 into accumulator
        let addr_result = AddressingResult::immediate(0x00);
        cpu.lda(&bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "Accumulator should be 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_lda_negative() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x80 (bit 7 set) into accumulator
        let addr_result = AddressingResult::immediate(0x80);
        cpu.lda(&bus, &addr_result);

        assert_eq!(cpu.a, 0x80, "Accumulator should be 0x80");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Write test value to memory
        bus.write(0x1234, 0x42);

        // Load from memory
        let addr_result = AddressingResult::new(0x1234);
        cpu.lda(&bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "Accumulator should be 0x42");
    }

    #[test]
    fn test_ldx_immediate() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x42 into X register
        let addr_result = AddressingResult::immediate(0x42);
        cpu.ldx(&bus, &addr_result);

        assert_eq!(cpu.x, 0x42, "X register should be 0x42");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ldx_zero() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x00 into X register
        let addr_result = AddressingResult::immediate(0x00);
        cpu.ldx(&bus, &addr_result);

        assert_eq!(cpu.x, 0x00, "X register should be 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ldx_negative() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0xFF (bit 7 set) into X register
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.ldx(&bus, &addr_result);

        assert_eq!(cpu.x, 0xFF, "X register should be 0xFF");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_ldy_immediate() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x42 into Y register
        let addr_result = AddressingResult::immediate(0x42);
        cpu.ldy(&bus, &addr_result);

        assert_eq!(cpu.y, 0x42, "Y register should be 0x42");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ldy_zero() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0x00 into Y register
        let addr_result = AddressingResult::immediate(0x00);
        cpu.ldy(&bus, &addr_result);

        assert_eq!(cpu.y, 0x00, "Y register should be 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ldy_negative() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Load 0xF0 (bit 7 set) into Y register
        let addr_result = AddressingResult::immediate(0xF0);
        cpu.ldy(&bus, &addr_result);

        assert_eq!(cpu.y, 0xF0, "Y register should be 0xF0");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // Store Instruction Tests
    // ========================================

    #[test]
    fn test_sta() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set accumulator to 0x42
        cpu.a = 0x42;

        // Set some flags to verify they're not affected
        cpu.set_zero(true);
        cpu.set_negative(true);

        // Store to memory
        let addr_result = AddressingResult::new(0x1234);
        cpu.sta(&mut bus, &addr_result);

        // Verify memory was written
        assert_eq!(bus.read(0x1234), 0x42, "Memory should contain 0x42");

        // Verify flags were not affected
        assert!(cpu.get_zero(), "Zero flag should not be affected");
        assert!(cpu.get_negative(), "Negative flag should not be affected");
    }

    #[test]
    fn test_stx() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set X register to 0x42
        cpu.x = 0x42;

        // Set some flags to verify they're not affected
        cpu.set_zero(true);
        cpu.set_negative(true);

        // Store to memory
        let addr_result = AddressingResult::new(0x1234);
        cpu.stx(&mut bus, &addr_result);

        // Verify memory was written
        assert_eq!(bus.read(0x1234), 0x42, "Memory should contain 0x42");

        // Verify flags were not affected
        assert!(cpu.get_zero(), "Zero flag should not be affected");
        assert!(cpu.get_negative(), "Negative flag should not be affected");
    }

    #[test]
    fn test_sty() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set Y register to 0x42
        cpu.y = 0x42;

        // Set some flags to verify they're not affected
        cpu.set_zero(true);
        cpu.set_negative(true);

        // Store to memory
        let addr_result = AddressingResult::new(0x1234);
        cpu.sty(&mut bus, &addr_result);

        // Verify memory was written
        assert_eq!(bus.read(0x1234), 0x42, "Memory should contain 0x42");

        // Verify flags were not affected
        assert!(cpu.get_zero(), "Zero flag should not be affected");
        assert!(cpu.get_negative(), "Negative flag should not be affected");
    }

    #[test]
    fn test_sta_zero_value() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set accumulator to 0x00
        cpu.a = 0x00;

        // Store to memory
        let addr_result = AddressingResult::new(0x1234);
        cpu.sta(&mut bus, &addr_result);

        // Verify memory was written
        assert_eq!(bus.read(0x1234), 0x00, "Memory should contain 0x00");

        // Zero flag should not be set by STA
        assert!(!cpu.get_zero(), "Zero flag should not be set by STA");
    }

    #[test]
    fn test_load_and_store_roundtrip() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Load immediate value
        let addr_result = AddressingResult::immediate(0x42);
        cpu.lda(&bus, &addr_result);

        // Store to memory
        let store_addr = AddressingResult::new(0x1234);
        cpu.sta(&mut bus, &store_addr);

        // Verify memory
        assert_eq!(bus.read(0x1234), 0x42);

        // Clear accumulator
        cpu.a = 0x00;

        // Load from memory
        cpu.lda(&bus, &store_addr);

        // Verify accumulator
        assert_eq!(cpu.a, 0x42);
    }
}
