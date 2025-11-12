// Instructions module for 6502 CPU
// Implements load, store, and transfer instructions

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;

impl super::Cpu {
    // ========================================
    // Helper Functions
    // ========================================

    /// Helper function to read a value from memory using an addressing result
    ///
    /// If the addressing result contains an immediate value, returns that value.
    /// Otherwise, reads from the address specified in the addressing result.
    #[inline]
    fn read_operand(&self, bus: &Bus, addr_result: &AddressingResult) -> u8 {
        if let Some(value) = addr_result.value {
            value
        } else {
            bus.read(addr_result.address)
        }
    }

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

    // ========================================
    // Register Transfer Instructions
    // ========================================
    // Transfer instructions move values between registers.
    // Most transfer instructions update the Zero (Z) and Negative (N) flags,
    // except TXS which does not affect any flags.

    /// TAX - Transfer Accumulator to X
    ///
    /// Copies the value from the accumulator (A) to the X register.
    /// Updates Zero and Negative flags based on the transferred value.
    ///
    /// Flags affected: Z, N
    pub fn tax(&mut self) {
        self.x = self.a;
        self.update_zero_and_negative_flags(self.x);
    }

    /// TAY - Transfer Accumulator to Y
    ///
    /// Copies the value from the accumulator (A) to the Y register.
    /// Updates Zero and Negative flags based on the transferred value.
    ///
    /// Flags affected: Z, N
    pub fn tay(&mut self) {
        self.y = self.a;
        self.update_zero_and_negative_flags(self.y);
    }

    /// TXA - Transfer X to Accumulator
    ///
    /// Copies the value from the X register to the accumulator (A).
    /// Updates Zero and Negative flags based on the transferred value.
    ///
    /// Flags affected: Z, N
    pub fn txa(&mut self) {
        self.a = self.x;
        self.update_zero_and_negative_flags(self.a);
    }

    /// TYA - Transfer Y to Accumulator
    ///
    /// Copies the value from the Y register to the accumulator (A).
    /// Updates Zero and Negative flags based on the transferred value.
    ///
    /// Flags affected: Z, N
    pub fn tya(&mut self) {
        self.a = self.y;
        self.update_zero_and_negative_flags(self.a);
    }

    /// TSX - Transfer Stack Pointer to X
    ///
    /// Copies the value from the stack pointer (SP) to the X register.
    /// Updates Zero and Negative flags based on the transferred value.
    ///
    /// Flags affected: Z, N
    pub fn tsx(&mut self) {
        self.x = self.sp;
        self.update_zero_and_negative_flags(self.x);
    }

    /// TXS - Transfer X to Stack Pointer
    ///
    /// Copies the value from the X register to the stack pointer (SP).
    /// Does NOT affect any flags (unlike other transfer instructions).
    ///
    /// Flags affected: None
    pub fn txs(&mut self) {
        self.sp = self.x;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;
    use crate::cpu::Cpu;

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

    // ========================================
    // Register Transfer Instruction Tests
    // ========================================

    #[test]
    fn test_tax() {
        let mut cpu = Cpu::new();

        // Set accumulator to 0x42
        cpu.a = 0x42;

        cpu.tax();

        assert_eq!(cpu.x, 0x42, "X should equal A");
        assert_eq!(cpu.a, 0x42, "A should remain unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tax_zero() {
        let mut cpu = Cpu::new();

        // Set accumulator to 0x00
        cpu.a = 0x00;

        cpu.tax();

        assert_eq!(cpu.x, 0x00, "X should equal A (0x00)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tax_negative() {
        let mut cpu = Cpu::new();

        // Set accumulator to 0x80 (bit 7 set)
        cpu.a = 0x80;

        cpu.tax();

        assert_eq!(cpu.x, 0x80, "X should equal A (0x80)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_tay() {
        let mut cpu = Cpu::new();

        // Set accumulator to 0x42
        cpu.a = 0x42;

        cpu.tay();

        assert_eq!(cpu.y, 0x42, "Y should equal A");
        assert_eq!(cpu.a, 0x42, "A should remain unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tay_zero() {
        let mut cpu = Cpu::new();

        // Set accumulator to 0x00
        cpu.a = 0x00;

        cpu.tay();

        assert_eq!(cpu.y, 0x00, "Y should equal A (0x00)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_txa() {
        let mut cpu = Cpu::new();

        // Set X register to 0x42
        cpu.x = 0x42;

        cpu.txa();

        assert_eq!(cpu.a, 0x42, "A should equal X");
        assert_eq!(cpu.x, 0x42, "X should remain unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_txa_zero() {
        let mut cpu = Cpu::new();

        // Set X register to 0x00
        cpu.x = 0x00;

        cpu.txa();

        assert_eq!(cpu.a, 0x00, "A should equal X (0x00)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_txa_negative() {
        let mut cpu = Cpu::new();

        // Set X register to 0xFF (bit 7 set)
        cpu.x = 0xFF;

        cpu.txa();

        assert_eq!(cpu.a, 0xFF, "A should equal X (0xFF)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_tya() {
        let mut cpu = Cpu::new();

        // Set Y register to 0x42
        cpu.y = 0x42;

        cpu.tya();

        assert_eq!(cpu.a, 0x42, "A should equal Y");
        assert_eq!(cpu.y, 0x42, "Y should remain unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tya_zero() {
        let mut cpu = Cpu::new();

        // Set Y register to 0x00
        cpu.y = 0x00;

        cpu.tya();

        assert_eq!(cpu.a, 0x00, "A should equal Y (0x00)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tsx() {
        let mut cpu = Cpu::new();

        // Set stack pointer to 0x42
        cpu.sp = 0x42;

        cpu.tsx();

        assert_eq!(cpu.x, 0x42, "X should equal SP");
        assert_eq!(cpu.sp, 0x42, "SP should remain unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tsx_zero() {
        let mut cpu = Cpu::new();

        // Set stack pointer to 0x00
        cpu.sp = 0x00;

        cpu.tsx();

        assert_eq!(cpu.x, 0x00, "X should equal SP (0x00)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_tsx_negative() {
        let mut cpu = Cpu::new();

        // Set stack pointer to 0xFF (bit 7 set)
        cpu.sp = 0xFF;

        cpu.tsx();

        assert_eq!(cpu.x, 0xFF, "X should equal SP (0xFF)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_txs_does_not_affect_flags() {
        let mut cpu = Cpu::new();

        // Set X register to 0x00 (would normally set Zero flag)
        cpu.x = 0x00;

        // Set flags to verify they're not affected
        cpu.set_zero(false);
        cpu.set_negative(true);

        cpu.txs();

        assert_eq!(cpu.sp, 0x00, "SP should equal X");
        assert_eq!(cpu.x, 0x00, "X should remain unchanged");

        // Verify flags were NOT affected
        assert!(!cpu.get_zero(), "Zero flag should not be affected");
        assert!(cpu.get_negative(), "Negative flag should not be affected");
    }

    #[test]
    fn test_txs() {
        let mut cpu = Cpu::new();

        // Set X register to 0x42
        cpu.x = 0x42;

        cpu.txs();

        assert_eq!(cpu.sp, 0x42, "SP should equal X");
        assert_eq!(cpu.x, 0x42, "X should remain unchanged");
    }

    #[test]
    fn test_txs_with_negative_value() {
        let mut cpu = Cpu::new();

        // Set X register to 0xFF (bit 7 set)
        cpu.x = 0xFF;

        // Clear flags initially
        cpu.set_zero(false);
        cpu.set_negative(false);

        cpu.txs();

        assert_eq!(cpu.sp, 0xFF, "SP should equal X (0xFF)");

        // TXS does not affect flags
        assert!(!cpu.get_zero(), "Zero flag should not be set");
        assert!(!cpu.get_negative(), "Negative flag should not be set");
    }

    // ========================================
    // Integration Tests
    // ========================================

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

    #[test]
    fn test_transfer_chain() {
        let mut cpu = Cpu::new();

        // Start with A = 0x42
        cpu.a = 0x42;

        // Transfer A -> X
        cpu.tax();
        assert_eq!(cpu.x, 0x42);

        // Transfer X -> A (should still be 0x42)
        cpu.txa();
        assert_eq!(cpu.a, 0x42);

        // Transfer A -> Y
        cpu.tay();
        assert_eq!(cpu.y, 0x42);

        // Transfer Y -> A
        cpu.tya();
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_stack_pointer_transfer() {
        let mut cpu = Cpu::new();

        // Set X to 0xFF
        cpu.x = 0xFF;

        // Transfer X -> SP
        cpu.txs();
        assert_eq!(cpu.sp, 0xFF);

        // Modify X
        cpu.x = 0x00;

        // Transfer SP -> X
        cpu.tsx();
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.sp, 0xFF);
    }
}
