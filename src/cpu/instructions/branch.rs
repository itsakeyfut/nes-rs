// Branch instructions for 6502 CPU
// These instructions perform conditional branches based on processor status flags.
// All branch instructions use relative addressing mode and do not modify any flags.

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Branch Instructions
    // ========================================

    /// BCC - Branch if Carry Clear
    ///
    /// Branches if the Carry flag (C) is clear (0).
    ///
    /// This instruction is typically used after comparison or arithmetic operations
    /// to test if there was no carry/borrow. Common use cases:
    /// - After CMP: Branch if first operand < second operand (unsigned)
    /// - After ADC: Branch if there was no carry
    /// - After subtraction operations to check for no borrow
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// CMP #$10    ; Compare A with $10
    /// BCC label   ; Branch if A < $10 (carry clear)
    /// ```
    pub fn bcc(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(!self.get_carry(), addr_result)
    }

    /// BCS - Branch if Carry Set
    ///
    /// Branches if the Carry flag (C) is set (1).
    ///
    /// This instruction is typically used after comparison or arithmetic operations
    /// to test if there was a carry/borrow. Common use cases:
    /// - After CMP: Branch if first operand >= second operand (unsigned)
    /// - After ADC: Branch if there was a carry
    /// - In multi-byte arithmetic operations
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// CMP #$10    ; Compare A with $10
    /// BCS label   ; Branch if A >= $10 (carry set)
    /// ```
    pub fn bcs(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(self.get_carry(), addr_result)
    }

    /// BEQ - Branch if Equal (Zero Set)
    ///
    /// Branches if the Zero flag (Z) is set (1).
    ///
    /// This instruction is typically used after comparison or arithmetic operations
    /// to test if the result was zero. Common use cases:
    /// - After CMP: Branch if values are equal
    /// - After load operations: Branch if value is zero
    /// - Loop counters: Branch when counter reaches zero
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// LDA counter ; Load counter value
    /// BEQ done    ; Branch if counter is zero
    /// ```
    pub fn beq(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(self.get_zero(), addr_result)
    }

    /// BNE - Branch if Not Equal (Zero Clear)
    ///
    /// Branches if the Zero flag (Z) is clear (0).
    ///
    /// This instruction is typically used after comparison or arithmetic operations
    /// to test if the result was not zero. Common use cases:
    /// - After CMP: Branch if values are not equal
    /// - Loop conditions: Continue loop while counter is not zero
    /// - Checking for non-zero values
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// loop:
    ///     DEX         ; Decrement X
    ///     BNE loop    ; Continue loop while X != 0
    /// ```
    pub fn bne(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(!self.get_zero(), addr_result)
    }

    /// BMI - Branch if Minus (Negative Set)
    ///
    /// Branches if the Negative flag (N) is set (1).
    ///
    /// This instruction is typically used to test if a value is negative
    /// (bit 7 is set). Common use cases:
    /// - Testing sign of a value after load or arithmetic
    /// - After CMP: Testing relative magnitude with signed interpretation
    /// - Processing signed numbers
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// LDA value   ; Load value
    /// BMI negative ; Branch if bit 7 is set (negative)
    /// ```
    pub fn bmi(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(self.get_negative(), addr_result)
    }

    /// BPL - Branch if Plus (Negative Clear)
    ///
    /// Branches if the Negative flag (N) is clear (0).
    ///
    /// This instruction is typically used to test if a value is positive
    /// (bit 7 is clear). Common use cases:
    /// - Testing sign of a value after load or arithmetic
    /// - After CMP: Testing relative magnitude with signed interpretation
    /// - Processing signed numbers
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// LDA value   ; Load value
    /// BPL positive ; Branch if bit 7 is clear (positive)
    /// ```
    pub fn bpl(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(!self.get_negative(), addr_result)
    }

    /// BVC - Branch if Overflow Clear
    ///
    /// Branches if the Overflow flag (V) is clear (0).
    ///
    /// This instruction is typically used after arithmetic operations (ADC, SBC)
    /// to test if signed overflow did not occur. Common use cases:
    /// - Checking for valid results in signed arithmetic
    /// - After ADC/SBC: Branch if no signed overflow occurred
    /// - Validating signed arithmetic operations
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// ADC #$10    ; Add with carry
    /// BVC no_overflow ; Branch if no signed overflow
    /// ```
    pub fn bvc(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(!self.get_overflow(), addr_result)
    }

    /// BVS - Branch if Overflow Set
    ///
    /// Branches if the Overflow flag (V) is set (1).
    ///
    /// This instruction is typically used after arithmetic operations (ADC, SBC)
    /// to test if signed overflow occurred. Common use cases:
    /// - Detecting invalid results in signed arithmetic
    /// - After ADC/SBC: Branch if signed overflow occurred
    /// - Error handling for signed arithmetic
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - 2 cycles if branch not taken
    /// - 3 cycles if branch taken
    /// - 4 cycles if branch taken and crosses page boundary
    ///
    /// # Arguments
    /// * `bus` - The memory bus (for reading the branch offset)
    /// * `addr_result` - The addressing result containing the branch target address
    ///
    /// # Returns
    /// The number of additional cycles used (0, 1, or 2)
    ///
    /// # Example
    /// ```text
    /// ADC #$70    ; Add with carry
    /// BVS overflow ; Branch if signed overflow occurred
    /// ```
    pub fn bvs(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.branch(self.get_overflow(), addr_result)
    }

    // ========================================
    // Helper Methods
    // ========================================

    /// Internal branch helper method
    ///
    /// Performs the actual branch logic used by all branch instructions.
    /// If the condition is true, updates the program counter to the target address
    /// and returns the number of additional cycles used.
    ///
    /// Branch timing:
    /// - Base cost: 2 cycles (included in instruction execution, not returned here)
    /// - +1 cycle if branch is taken (returned as 1)
    /// - +1 cycle if branch crosses page boundary (returned as 2 total)
    ///
    /// # Arguments
    /// * `condition` - Whether to take the branch
    /// * `addr_result` - The addressing result containing the branch target and page cross info
    ///
    /// # Returns
    /// Additional cycles: 0 if not taken, 1 if taken, 2 if taken and crossed page
    #[inline]
    fn branch(&mut self, condition: bool, addr_result: &AddressingResult) -> u8 {
        if condition {
            // Branch is taken - update PC to target address
            self.pc = addr_result.address;

            // Add 1 cycle for taken branch, plus 1 more if page boundary crossed
            if addr_result.page_crossed {
                2
            } else {
                1
            }
        } else {
            // Branch not taken - no additional cycles
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;
    use crate::cpu::Cpu;

    // ========================================
    // BCC (Branch if Carry Clear) Tests
    // ========================================

    #[test]
    fn test_bcc_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_carry(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bcc(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bcc_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_carry(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bcc(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    #[test]
    fn test_bcc_page_cross() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_carry(false);
        cpu.pc = 0x01FF;

        let addr_result = AddressingResult::new(0x0250).with_page_cross(true);
        let cycles = cpu.bcc(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 2, "Should take 2 additional cycles on page cross");
    }

    // ========================================
    // BCS (Branch if Carry Set) Tests
    // ========================================

    #[test]
    fn test_bcs_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_carry(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bcs(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bcs_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_carry(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bcs(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BEQ (Branch if Equal) Tests
    // ========================================

    #[test]
    fn test_beq_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.beq(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.beq(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BNE (Branch if Not Equal) Tests
    // ========================================

    #[test]
    fn test_bne_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bne(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bne_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bne(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    #[test]
    fn test_bne_loop_simulation() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Simulate a typical loop: DEX / BNE loop
        cpu.x = 5;
        cpu.pc = 0x0200;

        for i in (1..=5).rev() {
            cpu.x = i;
            cpu.set_zero(false);

            let addr_result = AddressingResult::new(0x0200);
            let cycles = cpu.bne(&bus, &addr_result);

            assert_eq!(cpu.pc, 0x0200, "Should branch back to loop start");
            assert_eq!(cycles, 1, "Should take 1 additional cycle");
        }

        // Final iteration - zero flag set, branch not taken
        cpu.x = 0;
        cpu.set_zero(true);
        cpu.pc = 0x0202;

        let addr_result = AddressingResult::new(0x0200);
        let cycles = cpu.bne(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0202, "Should not branch when zero");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BMI (Branch if Minus) Tests
    // ========================================

    #[test]
    fn test_bmi_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_negative(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bmi(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bmi_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_negative(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bmi(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BPL (Branch if Plus) Tests
    // ========================================

    #[test]
    fn test_bpl_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_negative(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bpl(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bpl_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_negative(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bpl(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BVC (Branch if Overflow Clear) Tests
    // ========================================

    #[test]
    fn test_bvc_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_overflow(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bvc(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bvc_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_overflow(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bvc(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // BVS (Branch if Overflow Set) Tests
    // ========================================

    #[test]
    fn test_bvs_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_overflow(true);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bvs(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0250, "PC should be updated to branch target");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_bvs_not_taken() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_overflow(false);
        cpu.pc = 0x0200;

        let addr_result = AddressingResult::new(0x0250);
        let cycles = cpu.bvs(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "PC should not change");
        assert_eq!(cycles, 0, "Should take no additional cycles");
    }

    // ========================================
    // Edge Cases and Integration Tests
    // ========================================

    #[test]
    fn test_branch_backward() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(true);
        cpu.pc = 0x0250;

        // Branch backward to earlier address
        let addr_result = AddressingResult::new(0x0200);
        let cycles = cpu.beq(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "Should branch backward");
        assert_eq!(cycles, 1, "Should take 1 additional cycle");
    }

    #[test]
    fn test_branch_backward_page_cross() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(true);
        cpu.pc = 0x0200;

        // Branch backward across page boundary
        let addr_result = AddressingResult::new(0x01F0).with_page_cross(true);
        let cycles = cpu.beq(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x01F0, "Should branch backward across page");
        assert_eq!(cycles, 2, "Should take 2 additional cycles on page cross");
    }

    #[test]
    fn test_branch_no_flag_modification() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute various branches
        let addr_result = AddressingResult::new(0x0250);
        cpu.bcs(&bus, &addr_result);
        cpu.bne(&bus, &addr_result);
        cpu.bpl(&bus, &addr_result);
        cpu.bvs(&bus, &addr_result);

        assert_eq!(
            cpu.status, initial_status,
            "Branch instructions should not modify any flags"
        );
    }

    #[test]
    fn test_all_branches_comprehensive() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Test BCC/BCS
        cpu.pc = 0x0200;
        cpu.set_carry(false);
        let cycles = cpu.bcc(&bus, &AddressingResult::new(0x0210));
        assert_eq!(cpu.pc, 0x0210);
        assert_eq!(cycles, 1);

        cpu.pc = 0x0200;
        cpu.set_carry(true);
        let cycles = cpu.bcs(&bus, &AddressingResult::new(0x0220));
        assert_eq!(cpu.pc, 0x0220);
        assert_eq!(cycles, 1);

        // Test BEQ/BNE
        cpu.pc = 0x0200;
        cpu.set_zero(true);
        let cycles = cpu.beq(&bus, &AddressingResult::new(0x0230));
        assert_eq!(cpu.pc, 0x0230);
        assert_eq!(cycles, 1);

        cpu.pc = 0x0200;
        cpu.set_zero(false);
        let cycles = cpu.bne(&bus, &AddressingResult::new(0x0240));
        assert_eq!(cpu.pc, 0x0240);
        assert_eq!(cycles, 1);

        // Test BMI/BPL
        cpu.pc = 0x0200;
        cpu.set_negative(true);
        let cycles = cpu.bmi(&bus, &AddressingResult::new(0x0250));
        assert_eq!(cpu.pc, 0x0250);
        assert_eq!(cycles, 1);

        cpu.pc = 0x0200;
        cpu.set_negative(false);
        let cycles = cpu.bpl(&bus, &AddressingResult::new(0x0260));
        assert_eq!(cpu.pc, 0x0260);
        assert_eq!(cycles, 1);

        // Test BVC/BVS
        cpu.pc = 0x0200;
        cpu.set_overflow(false);
        let cycles = cpu.bvc(&bus, &AddressingResult::new(0x0270));
        assert_eq!(cpu.pc, 0x0270);
        assert_eq!(cycles, 1);

        cpu.pc = 0x0200;
        cpu.set_overflow(true);
        let cycles = cpu.bvs(&bus, &AddressingResult::new(0x0280));
        assert_eq!(cpu.pc, 0x0280);
        assert_eq!(cycles, 1);
    }

    #[test]
    fn test_branch_to_same_address() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.set_zero(true);
        cpu.pc = 0x0200;

        // Branch to same address (infinite loop pattern)
        let addr_result = AddressingResult::new(0x0200);
        let cycles = cpu.beq(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200, "Should branch to same address");
        assert_eq!(cycles, 1, "Should still take 1 cycle");
    }

    #[test]
    fn test_branch_cycle_accuracy_with_multiple_conditions() {
        // Test realistic scenario: Multiple branches in sequence with varying cycle counts
        // This simulates typical 6502 code patterns where cycle timing matters
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        let mut total_cycles = 0;

        // Scenario 1: Branch not taken (0 extra cycles)
        cpu.pc = 0x0200;
        cpu.set_zero(false);
        total_cycles += cpu.beq(&bus, &AddressingResult::new(0x0250));
        assert_eq!(cpu.pc, 0x0200, "PC should not change when branch not taken");
        assert_eq!(total_cycles, 0, "No extra cycles for branch not taken");

        // Scenario 2: Branch taken, same page (1 extra cycle)
        cpu.pc = 0x0200;
        cpu.set_zero(true);
        total_cycles += cpu.beq(&bus, &AddressingResult::new(0x0250));
        assert_eq!(cpu.pc, 0x0250, "PC should update when branch taken");
        assert_eq!(total_cycles, 1, "1 extra cycle for branch taken");

        // Scenario 3: Branch taken, page crossed (2 extra cycles)
        cpu.pc = 0x01FE;
        cpu.set_carry(true);
        total_cycles +=
            cpu.bcs(&bus, &AddressingResult::new(0x0210).with_page_cross(true));
        assert_eq!(cpu.pc, 0x0210, "PC should update to new page");
        assert_eq!(
            total_cycles, 3,
            "Total should be 3 (0 + 1 + 2)"
        );

        // Scenario 4: Another branch not taken
        cpu.pc = 0x0210;
        cpu.set_negative(false);
        total_cycles += cpu.bmi(&bus, &AddressingResult::new(0x0300));
        assert_eq!(cpu.pc, 0x0210, "PC should not change");
        assert_eq!(
            total_cycles, 3,
            "Total should remain 3"
        );

        // Scenario 5: Branch backward with page cross (2 extra cycles)
        cpu.pc = 0x0300;
        cpu.set_overflow(true);
        total_cycles +=
            cpu.bvs(&bus, &AddressingResult::new(0x02F0).with_page_cross(true));
        assert_eq!(cpu.pc, 0x02F0, "PC should branch backward across page");
        assert_eq!(
            total_cycles, 5,
            "Total should be 5 (3 + 2)"
        );

        // Verify cycle counting is deterministic and accurate
        // This is critical for timing-sensitive 6502 code (scanline timing, music, etc.)
        assert_eq!(
            total_cycles, 5,
            "Final cycle count should be exactly 5 for this sequence"
        );
    }
}
