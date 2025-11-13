// Stack operation instructions for 6502 CPU
// These instructions handle pushing and pulling values to/from the stack.

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::flags;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Stack Operation Instructions
    // ========================================

    /// PHA - Push Accumulator
    ///
    /// Pushes the contents of the accumulator onto the stack.
    /// The stack pointer is decremented after the push.
    ///
    /// Operation: [SP] = A, SP = SP - 1
    ///
    /// Flags affected: None
    ///
    /// Cycles: 3 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// LDA #$42    ; Load $42 into accumulator
    /// PHA         ; Push $42 onto stack
    /// ```
    pub fn pha(&mut self, bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.stack_push(bus, self.a);
        0
    }

    /// PLA - Pull Accumulator
    ///
    /// Pulls a byte from the stack and stores it in the accumulator.
    /// The stack pointer is incremented before the pull.
    ///
    /// Operation: SP = SP + 1, A = [SP]
    ///
    /// Flags affected:
    /// - Z: Set if the pulled value is zero
    /// - N: Set if bit 7 of the pulled value is set
    ///
    /// Cycles: 4 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// PLA         ; Pull value from stack into accumulator
    /// ```
    pub fn pla(&mut self, bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.a = self.stack_pop(bus);
        self.update_zero_and_negative_flags(self.a);
        0
    }

    /// PHP - Push Processor Status
    ///
    /// Pushes the processor status register (P) onto the stack.
    /// According to 6502 specification, the B flag is set to 1 when pushed by PHP.
    /// The UNUSED flag (bit 5) is always set to 1 when pushed.
    ///
    /// Operation: [SP] = P | 0x30, SP = SP - 1
    ///
    /// Flags affected: None (the pushed value has B=1, but the CPU's status register is unchanged)
    ///
    /// Cycles: 3 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// PHP         ; Push processor status onto stack
    /// ```
    ///
    /// # Implementation Note
    /// The 6502 always pushes the status register with the B flag set to 1 and
    /// the UNUSED flag set to 1 (bits 4 and 5). This means the pushed value is
    /// (status | 0x30). The actual CPU status register is not modified.
    pub fn php(&mut self, bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        // Push status with B flag and UNUSED flag set to 1
        // B flag (bit 4) and UNUSED flag (bit 5) must be set when pushed
        let status_to_push = self.status | flags::BREAK | flags::UNUSED;
        self.stack_push(bus, status_to_push);
        0
    }

    /// PLP - Pull Processor Status
    ///
    /// Pulls a byte from the stack and stores it in the processor status register (P).
    /// The stack pointer is incremented before the pull.
    ///
    /// According to 6502 specification, the UNUSED flag (bit 5) is always set to 1
    /// in the status register, and the B flag from the stack is ignored.
    ///
    /// Operation: SP = SP + 1, P = [SP] | 0x20 (with B flag ignored)
    ///
    /// Flags affected: All flags are loaded from the stack
    /// - C: Carry
    /// - Z: Zero
    /// - I: Interrupt Disable
    /// - D: Decimal (unused in NES)
    /// - V: Overflow
    /// - N: Negative
    /// - B: Ignored from stack (not modified in status register)
    /// - UNUSED: Always set to 1
    ///
    /// Cycles: 4 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// PLP         ; Pull processor status from stack
    /// ```
    ///
    /// # Implementation Note
    /// When pulling status from stack:
    /// 1. The UNUSED flag (bit 5) is always set to 1 in the CPU status register
    /// 2. The B flag (bit 4) from the stack is ignored (not copied to status register)
    ///
    /// This is important for RTI (Return from Interrupt) which behaves differently.
    pub fn plp(&mut self, bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        let status_from_stack = self.stack_pop(bus);

        // Save the current B flag before updating status
        let current_b_flag = self.get_flag(flags::BREAK);

        // Set status from stack, forcing UNUSED flag (bit 5) to 1
        self.status = status_from_stack | flags::UNUSED;

        // Restore the B flag to its previous value (ignore B flag from stack)
        self.update_flag(flags::BREAK, current_b_flag);

        0
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;
    use crate::cpu::flags;
    use crate::cpu::Cpu;

    // ========================================
    // PHA (Push Accumulator) Tests
    // ========================================

    #[test]
    fn test_pha_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let initial_sp = cpu.sp;

        // Execute PHA
        let addr_result = AddressingResult::new(0); // Unused for PHA
        let cycles = cpu.pha(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "PHA should not return additional cycles");
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(1),
            "SP should decrement after PHA"
        );

        // Verify the accumulator value was pushed to stack
        let stack_addr = 0x0100 | (initial_sp as u16);
        assert_eq!(
            bus.read(stack_addr),
            0x42,
            "Accumulator value should be on stack"
        );
    }

    #[test]
    fn test_pha_no_flag_modification() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_overflow(true);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute PHA
        let addr_result = AddressingResult::new(0);
        cpu.pha(&mut bus, &addr_result);

        assert_eq!(
            cpu.status, initial_status,
            "PHA should not modify any flags"
        );
    }

    #[test]
    fn test_pha_multiple_values() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let initial_sp = cpu.sp;

        // Push multiple values
        cpu.a = 0x11;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        cpu.a = 0x22;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        cpu.a = 0x33;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(3),
            "SP should decrement by 3"
        );

        // Verify values on stack (LIFO order)
        assert_eq!(bus.read(0x0100 | (initial_sp as u16)), 0x11);
        assert_eq!(
            bus.read(0x0100 | ((initial_sp.wrapping_sub(1)) as u16)),
            0x22
        );
        assert_eq!(
            bus.read(0x0100 | ((initial_sp.wrapping_sub(2)) as u16)),
            0x33
        );
    }

    #[test]
    fn test_pha_zero_value() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x00;
        let initial_sp = cpu.sp;

        cpu.pha(&mut bus, &AddressingResult::new(0));

        let stack_addr = 0x0100 | (initial_sp as u16);
        assert_eq!(bus.read(stack_addr), 0x00, "Should push zero value");
    }

    #[test]
    fn test_pha_ff_value() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0xFF;
        let initial_sp = cpu.sp;

        cpu.pha(&mut bus, &AddressingResult::new(0));

        let stack_addr = 0x0100 | (initial_sp as u16);
        assert_eq!(bus.read(stack_addr), 0xFF, "Should push 0xFF value");
    }

    // ========================================
    // PLA (Pull Accumulator) Tests
    // ========================================

    #[test]
    fn test_pla_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push a value onto stack first
        cpu.a = 0x42;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        let sp_after_push = cpu.sp;

        // Clear accumulator
        cpu.a = 0x00;

        // Execute PLA
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.pla(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "PLA should not return additional cycles");
        assert_eq!(cpu.a, 0x42, "Accumulator should have pulled value");
        assert_eq!(
            cpu.sp,
            sp_after_push.wrapping_add(1),
            "SP should increment after PLA"
        );
    }

    #[test]
    fn test_pla_sets_zero_flag() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push zero value
        cpu.a = 0x00;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Set accumulator to non-zero and clear zero flag
        cpu.a = 0xFF;
        cpu.set_zero(false);

        // Execute PLA
        cpu.pla(&mut bus, &AddressingResult::new(0));

        assert_eq!(cpu.a, 0x00, "Accumulator should be zero");
        assert!(cpu.get_zero(), "Zero flag should be set when pulling zero");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear for zero"
        );
    }

    #[test]
    fn test_pla_sets_negative_flag() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push negative value (bit 7 set)
        cpu.a = 0x80;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Clear flags
        cpu.a = 0x00;
        cpu.set_negative(false);
        cpu.set_zero(false);

        // Execute PLA
        cpu.pla(&mut bus, &AddressingResult::new(0));

        assert_eq!(cpu.a, 0x80, "Accumulator should have value 0x80");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when bit 7 is set"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear");
    }

    #[test]
    fn test_pla_clears_flags_for_positive() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push positive value
        cpu.a = 0x42;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Set flags that should be cleared
        cpu.a = 0x00;
        cpu.set_zero(true);
        cpu.set_negative(true);

        // Execute PLA
        cpu.pla(&mut bus, &AddressingResult::new(0));

        assert_eq!(cpu.a, 0x42, "Accumulator should have value 0x42");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_pla_other_flags_unchanged() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push a value
        cpu.a = 0x42;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Set other flags
        cpu.set_carry(true);
        cpu.set_interrupt_disable(false);
        cpu.set_overflow(true);

        let carry = cpu.get_carry();
        let interrupt = cpu.get_interrupt_disable();
        let overflow = cpu.get_overflow();

        // Execute PLA
        cpu.pla(&mut bus, &AddressingResult::new(0));

        assert_eq!(cpu.get_carry(), carry, "Carry flag should be unchanged");
        assert_eq!(
            cpu.get_interrupt_disable(),
            interrupt,
            "Interrupt flag should be unchanged"
        );
        assert_eq!(
            cpu.get_overflow(),
            overflow,
            "Overflow flag should be unchanged"
        );
    }

    #[test]
    fn test_pha_pla_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let original_value = 0x42;
        let initial_sp = cpu.sp;

        // Push accumulator
        cpu.a = original_value;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Modify accumulator
        cpu.a = 0xFF;

        // Pull accumulator
        cpu.pla(&mut bus, &AddressingResult::new(0));

        assert_eq!(cpu.a, original_value, "Should restore original value");
        assert_eq!(cpu.sp, initial_sp, "SP should be restored");
    }

    #[test]
    fn test_pla_multiple_values() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push three values
        cpu.a = 0x11;
        cpu.pha(&mut bus, &AddressingResult::new(0));
        cpu.a = 0x22;
        cpu.pha(&mut bus, &AddressingResult::new(0));
        cpu.a = 0x33;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Pull them back (LIFO order)
        cpu.pla(&mut bus, &AddressingResult::new(0));
        assert_eq!(cpu.a, 0x33, "Should pull last value first");

        cpu.pla(&mut bus, &AddressingResult::new(0));
        assert_eq!(cpu.a, 0x22, "Should pull second value");

        cpu.pla(&mut bus, &AddressingResult::new(0));
        assert_eq!(cpu.a, 0x11, "Should pull first value last");
    }

    // ========================================
    // PHP (Push Processor Status) Tests
    // ========================================

    #[test]
    fn test_php_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set some flags
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_negative(true);

        let initial_sp = cpu.sp;

        // Execute PHP
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.php(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "PHP should not return additional cycles");
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(1),
            "SP should decrement after PHP"
        );

        // Verify the status was pushed to stack with B flag set
        let stack_addr = 0x0100 | (initial_sp as u16);
        let pushed_status = bus.read(stack_addr);

        // The pushed status should have B flag (bit 4) and UNUSED flag (bit 5) set
        assert_eq!(
            pushed_status & flags::BREAK,
            flags::BREAK,
            "B flag should be set in pushed status"
        );
        assert_eq!(
            pushed_status & flags::UNUSED,
            flags::UNUSED,
            "UNUSED flag should be set in pushed status"
        );

        // Check that other flags match
        assert_eq!(
            pushed_status & flags::CARRY,
            flags::CARRY,
            "Carry flag should match"
        );
        assert_eq!(
            pushed_status & flags::ZERO,
            flags::ZERO,
            "Zero flag should match"
        );
        assert_eq!(
            pushed_status & flags::NEGATIVE,
            flags::NEGATIVE,
            "Negative flag should match"
        );
    }

    #[test]
    fn test_php_sets_b_flag_when_pushed() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear B flag in status register
        cpu.set_break(false);
        assert!(!cpu.get_break(), "B flag should be clear in CPU");

        let initial_sp = cpu.sp;

        // Execute PHP
        cpu.php(&mut bus, &AddressingResult::new(0));

        // Check that B flag is still clear in CPU status
        assert!(!cpu.get_break(), "B flag should still be clear in CPU");

        // But the pushed value should have B flag set
        let stack_addr = 0x0100 | (initial_sp as u16);
        let pushed_status = bus.read(stack_addr);
        assert_eq!(
            pushed_status & flags::BREAK,
            flags::BREAK,
            "B flag should be set in pushed value"
        );
    }

    #[test]
    fn test_php_no_flag_modification() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(false);
        cpu.set_break(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute PHP
        cpu.php(&mut bus, &AddressingResult::new(0));

        assert_eq!(
            cpu.status, initial_status,
            "PHP should not modify CPU status register"
        );
    }

    #[test]
    fn test_php_all_flags_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags
        cpu.status = 0xFF;
        let initial_sp = cpu.sp;

        cpu.php(&mut bus, &AddressingResult::new(0));

        let stack_addr = 0x0100 | (initial_sp as u16);
        let pushed_status = bus.read(stack_addr);

        // Should be 0xFF (all flags set)
        assert_eq!(pushed_status, 0xFF, "All flags should be set");
    }

    #[test]
    fn test_php_all_flags_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear all flags except UNUSED (which must always be 1)
        cpu.status = flags::UNUSED;
        let initial_sp = cpu.sp;

        cpu.php(&mut bus, &AddressingResult::new(0));

        let stack_addr = 0x0100 | (initial_sp as u16);
        let pushed_status = bus.read(stack_addr);

        // Should have B flag and UNUSED flag set (0x30)
        assert_eq!(
            pushed_status, 0x30,
            "Should have B and UNUSED flags set (0x30)"
        );
    }

    // ========================================
    // PLP (Pull Processor Status) Tests
    // ========================================

    #[test]
    fn test_plp_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set initial status
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_negative(true);

        // Push status
        cpu.php(&mut bus, &AddressingResult::new(0));

        let sp_after_push = cpu.sp;

        // Change all flags
        cpu.set_carry(false);
        cpu.set_zero(false);
        cpu.set_negative(false);

        // Execute PLP
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.plp(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "PLP should not return additional cycles");
        assert_eq!(
            cpu.sp,
            sp_after_push.wrapping_add(1),
            "SP should increment after PLP"
        );

        // Verify flags were restored
        assert!(cpu.get_carry(), "Carry flag should be restored");
        assert!(cpu.get_zero(), "Zero flag should be restored");
        assert!(cpu.get_negative(), "Negative flag should be restored");
    }

    #[test]
    fn test_plp_unused_flag_always_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Manually push a status value with UNUSED flag clear (which shouldn't happen)
        cpu.stack_push(&mut bus, 0x00);

        // Execute PLP
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // UNUSED flag should be forced to 1
        assert!(
            cpu.get_flag(flags::UNUSED),
            "UNUSED flag must always be set after PLP"
        );
    }

    #[test]
    fn test_plp_ignores_b_flag() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set B flag in CPU
        cpu.set_break(true);
        let initial_b_flag = cpu.get_break();

        // Manually push a status value with B flag clear
        cpu.stack_push(&mut bus, 0x00);

        // Execute PLP
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // B flag should remain unchanged from before PLP
        assert_eq!(
            cpu.get_break(),
            initial_b_flag,
            "B flag should be ignored from stack"
        );
    }

    #[test]
    fn test_plp_restores_all_flags() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set specific flag pattern
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        // Push status
        cpu.php(&mut bus, &AddressingResult::new(0));

        // Change all flags to opposite
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(true);
        cpu.set_overflow(false);
        cpu.set_negative(true);

        // Execute PLP
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // Verify original flags were restored
        assert!(cpu.get_carry(), "Carry should be restored");
        assert!(!cpu.get_zero(), "Zero should be restored");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be restored"
        );
        assert!(!cpu.get_decimal(), "Decimal should be restored");
        assert!(cpu.get_overflow(), "Overflow should be restored");
        assert!(!cpu.get_negative(), "Negative should be restored");
    }

    #[test]
    fn test_php_plp_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set specific status pattern (ensuring UNUSED flag is set as it should always be)
        cpu.status = 0b11100111; // N, V, UNUSED, C, Z, I set
        let initial_status = cpu.status;
        let initial_sp = cpu.sp;

        // Push status
        cpu.php(&mut bus, &AddressingResult::new(0));

        // Modify status
        cpu.status = 0b00111000;

        // Pull status
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // Status should be restored (except B flag handling)
        // The B flag from the modified status (0b00111000) should be preserved
        let expected_status = (initial_status & !flags::BREAK) | (0b00111000 & flags::BREAK);
        assert_eq!(
            cpu.status & !flags::BREAK,
            expected_status & !flags::BREAK,
            "Status should be restored (ignoring B flag)"
        );
        assert_eq!(cpu.sp, initial_sp, "SP should be restored");
    }

    #[test]
    fn test_plp_with_all_flags_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push 0xFF (all flags set)
        cpu.stack_push(&mut bus, 0xFF);

        // Execute PLP
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // All flags should be set (except B flag is ignored)
        assert!(cpu.get_carry(), "Carry should be set");
        assert!(cpu.get_zero(), "Zero should be set");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be set"
        );
        assert!(cpu.get_decimal(), "Decimal should be set");
        assert!(cpu.get_overflow(), "Overflow should be set");
        assert!(cpu.get_negative(), "Negative should be set");
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED should be set");
    }

    #[test]
    fn test_plp_with_all_flags_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push 0x00 (all flags clear)
        cpu.stack_push(&mut bus, 0x00);

        // Execute PLP
        cpu.plp(&mut bus, &AddressingResult::new(0));

        // All flags should be clear (except UNUSED which is forced to 1)
        assert!(!cpu.get_carry(), "Carry should be clear");
        assert!(!cpu.get_zero(), "Zero should be clear");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should be clear"
        );
        assert!(!cpu.get_decimal(), "Decimal should be clear");
        assert!(!cpu.get_overflow(), "Overflow should be clear");
        assert!(!cpu.get_negative(), "Negative should be clear");
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED must always be set");
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_stack_operations_integration() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let initial_sp = cpu.sp;

        // Push accumulator
        cpu.a = 0x42;
        cpu.pha(&mut bus, &AddressingResult::new(0));

        // Push status
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.php(&mut bus, &AddressingResult::new(0));

        // Verify stack pointer moved
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(2));

        // Modify accumulator and flags
        cpu.a = 0xFF;
        cpu.set_carry(false);
        cpu.set_zero(false);

        // Pull status
        cpu.plp(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_carry(), "Carry should be restored");
        assert!(cpu.get_zero(), "Zero should be restored");

        // Pull accumulator
        cpu.pla(&mut bus, &AddressingResult::new(0));
        assert_eq!(cpu.a, 0x42, "Accumulator should be restored");

        // Stack pointer should be back to initial
        assert_eq!(cpu.sp, initial_sp, "SP should be back to initial value");
    }

    #[test]
    fn test_nested_stack_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Simulate nested subroutine calls with stack operations
        cpu.a = 0x11;
        cpu.pha(&mut bus, &AddressingResult::new(0)); // Level 1

        cpu.set_carry(true);
        cpu.php(&mut bus, &AddressingResult::new(0)); // Level 1 status

        cpu.a = 0x22;
        cpu.pha(&mut bus, &AddressingResult::new(0)); // Level 2

        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.php(&mut bus, &AddressingResult::new(0)); // Level 2 status

        // Unwind stack (LIFO)
        cpu.plp(&mut bus, &AddressingResult::new(0)); // Restore level 2 status
        assert!(!cpu.get_carry());
        assert!(cpu.get_zero());

        cpu.pla(&mut bus, &AddressingResult::new(0)); // Restore level 2 accumulator
        assert_eq!(cpu.a, 0x22);

        cpu.plp(&mut bus, &AddressingResult::new(0)); // Restore level 1 status
        assert!(cpu.get_carry());

        cpu.pla(&mut bus, &AddressingResult::new(0)); // Restore level 1 accumulator
        assert_eq!(cpu.a, 0x11);
    }
}
