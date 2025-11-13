// Miscellaneous instructions for 6502 CPU
// These instructions include NOP, BRK (software interrupt), and RTI (return from interrupt).

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::flags;
use crate::cpu::vectors;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Miscellaneous Instructions
    // ========================================

    /// NOP - No Operation
    ///
    /// Does nothing. This instruction is used for timing delays or as a placeholder.
    ///
    /// Operation: None
    ///
    /// Flags affected: None
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - Unused (implied addressing mode)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// NOP         ; Do nothing, wait 2 cycles
    /// ```
    pub fn nop(&mut self, _bus: &Bus, _addr_result: &AddressingResult) -> u8 {
        // Do nothing - this is the entire purpose of NOP
        0
    }

    /// BRK - Break / Software Interrupt
    ///
    /// Triggers a software interrupt (IRQ). This is used to invoke interrupt handlers
    /// or for debugging purposes.
    ///
    /// Operation:
    /// 1. PC is incremented by 2 (to skip the padding byte after BRK)
    /// 2. Push PC high byte to stack
    /// 3. Push PC low byte to stack
    /// 4. Push status flags with B flag set to stack
    /// 5. Set I (Interrupt Disable) flag
    /// 6. Load PC from IRQ vector at $FFFE-$FFFF
    ///
    /// Flags affected:
    /// - I: Set to 1 (disable interrupts)
    /// - B: Set to 1 in the pushed status (but not in the CPU status register)
    ///
    /// Cycles: 7 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations and reading IRQ vector
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// BRK         ; Trigger software interrupt
    /// ```
    ///
    /// # Implementation Note
    /// According to 6502 specification:
    /// - PC is incremented by 2 before being pushed (BRK has a padding byte)
    /// - The B flag is set to 1 in the pushed status register
    /// - The I flag is set after pushing status
    /// - The actual CPU status register's B flag is not modified
    /// - After BRK, execution continues from the address stored at $FFFE-$FFFF
    pub fn brk(&mut self, bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        // Increment PC by 2 (BRK has a padding byte after it)
        self.pc = self.pc.wrapping_add(2);

        // Push PC to stack (high byte first, then low byte)
        self.stack_push_u16(bus, self.pc);

        // Push status flags with B flag and UNUSED flag set
        // According to 6502 spec, when BRK pushes status, B flag is set
        let status_to_push = self.status | flags::BREAK | flags::UNUSED;
        self.stack_push(bus, status_to_push);

        // Set the Interrupt Disable flag
        self.set_interrupt_disable(true);

        // Load PC from IRQ vector ($FFFE-$FFFF)
        // Low byte at $FFFE, high byte at $FFFF
        let lo = bus.read(vectors::IRQ) as u16;
        let hi = bus.read(vectors::IRQ.wrapping_add(1)) as u16;
        self.pc = (hi << 8) | lo;

        0
    }

    /// RTI - Return from Interrupt
    ///
    /// Returns from an interrupt handler (NMI or IRQ).
    /// Restores the processor status and program counter from the stack.
    ///
    /// Operation:
    /// 1. Pull status flags from stack
    /// 2. Pull PC low byte from stack
    /// 3. Pull PC high byte from stack
    ///
    /// Flags affected:
    /// All flags are restored from the stack:
    /// - C: Carry
    /// - Z: Zero
    /// - I: Interrupt Disable
    /// - D: Decimal (unused in NES)
    /// - V: Overflow
    /// - N: Negative
    ///
    /// Cycles: 6 cycles
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
    /// ; Inside an interrupt handler:
    /// ; ... handle interrupt ...
    /// RTI         ; Return from interrupt
    /// ```
    ///
    /// # Implementation Note
    /// Like PLP (Pull Processor Status), RTI ignores the B flag from the stack.
    /// The B flag is not a real flag in the 6502 - it's only set when pushing status
    /// to distinguish between BRK (B=1) and hardware interrupts IRQ/NMI (B=0).
    ///
    /// The UNUSED flag (bit 5) is always set to 1 in the status register after pulling.
    pub fn rti(&mut self, bus: &Bus, _addr_result: &AddressingResult) -> u8 {
        // Pull status flags from stack
        let status_from_stack = self.stack_pop(bus);

        // Save the current B flag before updating status
        let current_b_flag = self.get_flag(flags::BREAK);

        // Set status from stack, forcing UNUSED flag (bit 5) to 1
        self.status = status_from_stack | flags::UNUSED;

        // Restore the B flag to its previous value (ignore B flag from stack)
        self.update_flag(flags::BREAK, current_b_flag);

        // Pull PC from stack (low byte first, then high byte)
        self.pc = self.stack_pop_u16(bus);

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
    // NOP (No Operation) Tests
    // ========================================

    #[test]
    fn test_nop_does_nothing() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Save initial state
        let initial_a = cpu.a;
        let initial_x = cpu.x;
        let initial_y = cpu.y;
        let initial_sp = cpu.sp;
        let initial_pc = cpu.pc;
        let initial_status = cpu.status;

        // Execute NOP
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.nop(&bus, &addr_result);

        // Verify nothing changed
        assert_eq!(cycles, 0, "NOP should not return additional cycles");
        assert_eq!(cpu.a, initial_a, "Accumulator should not change");
        assert_eq!(cpu.x, initial_x, "X register should not change");
        assert_eq!(cpu.y, initial_y, "Y register should not change");
        assert_eq!(cpu.sp, initial_sp, "Stack pointer should not change");
        assert_eq!(cpu.pc, initial_pc, "Program counter should not change");
        assert_eq!(cpu.status, initial_status, "Status flags should not change");
    }

    #[test]
    fn test_nop_with_various_states() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Set various CPU states
        cpu.a = 0x42;
        cpu.x = 0x13;
        cpu.y = 0x37;
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_negative(true);
        cpu.set_overflow(true);

        let initial_status = cpu.status;

        // Execute NOP
        cpu.nop(&bus, &AddressingResult::new(0));

        // Verify state unchanged
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0x13);
        assert_eq!(cpu.y, 0x37);
        assert_eq!(cpu.status, initial_status);
    }

    // ========================================
    // BRK (Break / Software Interrupt) Tests
    // ========================================

    #[test]
    fn test_brk_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector at $FFFE-$FFFF
        let irq_handler_addr: u16 = 0x8000;
        bus.write(0xFFFE, (irq_handler_addr & 0xFF) as u8); // Low byte
        bus.write(0xFFFF, (irq_handler_addr >> 8) as u8); // High byte

        // Set initial PC and status
        cpu.pc = 0x1000;
        cpu.set_carry(true);
        cpu.set_zero(false);
        let initial_sp = cpu.sp;

        // Execute BRK
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.brk(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "BRK should not return additional cycles");

        // Verify PC was loaded from IRQ vector
        assert_eq!(
            cpu.pc, irq_handler_addr,
            "PC should be loaded from IRQ vector"
        );

        // Verify Interrupt Disable flag is set
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt Disable flag should be set"
        );

        // Verify stack operations
        // BRK pushes PC+2, then status with B flag set
        // SP should have decreased by 3 (2 bytes for PC, 1 for status)
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(3),
            "SP should decrement by 3"
        );

        // Verify PC+2 was pushed to stack (high byte first)
        let expected_pc = 0x1000u16.wrapping_add(2);
        let pushed_pc_hi = bus.read(0x0100 | (initial_sp as u16));
        let pushed_pc_lo = bus.read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
        let pushed_pc = ((pushed_pc_hi as u16) << 8) | (pushed_pc_lo as u16);
        assert_eq!(
            pushed_pc, expected_pc,
            "PC+2 should be pushed to stack (0x1002)"
        );

        // Verify status with B flag set was pushed
        let pushed_status = bus.read(0x0100 | (initial_sp.wrapping_sub(2) as u16));
        assert_eq!(
            pushed_status & flags::BREAK,
            flags::BREAK,
            "Pushed status should have B flag set"
        );
        assert_eq!(
            pushed_status & flags::UNUSED,
            flags::UNUSED,
            "Pushed status should have UNUSED flag set"
        );
        assert_eq!(
            pushed_status & flags::CARRY,
            flags::CARRY,
            "Pushed status should preserve carry flag"
        );
    }

    #[test]
    fn test_brk_sets_interrupt_disable() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80);

        // Clear interrupt disable flag
        cpu.set_interrupt_disable(false);
        assert!(
            !cpu.get_interrupt_disable(),
            "I flag should initially be clear"
        );

        // Execute BRK
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Verify I flag is now set
        assert!(
            cpu.get_interrupt_disable(),
            "I flag should be set after BRK"
        );
    }

    #[test]
    fn test_brk_pushes_status_with_b_flag() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80);

        // Ensure B flag is NOT set in CPU status
        cpu.set_break(false);
        assert!(!cpu.get_break(), "B flag should be clear in CPU");

        let initial_sp = cpu.sp;

        // Execute BRK
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Verify B flag is still NOT set in CPU status
        assert!(!cpu.get_break(), "B flag should remain clear in CPU status");

        // Verify pushed status has B flag set
        let pushed_status = bus.read(0x0100 | (initial_sp.wrapping_sub(2) as u16));
        assert_eq!(
            pushed_status & flags::BREAK,
            flags::BREAK,
            "Pushed status must have B flag set"
        );
    }

    #[test]
    fn test_brk_increments_pc_by_2() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x90);

        cpu.pc = 0x1234;
        let initial_sp = cpu.sp;

        // Execute BRK
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Verify PC+2 was pushed (0x1234 + 2 = 0x1236)
        let pushed_pc_hi = bus.read(0x0100 | (initial_sp as u16));
        let pushed_pc_lo = bus.read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
        let pushed_pc = ((pushed_pc_hi as u16) << 8) | (pushed_pc_lo as u16);

        assert_eq!(pushed_pc, 0x1236, "Should push PC+2 (0x1236)");
    }

    #[test]
    fn test_brk_preserves_other_flags() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80);

        // Set various flags
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_overflow(true);
        cpu.set_negative(true);
        cpu.set_decimal(true);

        let initial_sp = cpu.sp;

        // Execute BRK
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Verify flags are preserved in pushed status
        let pushed_status = bus.read(0x0100 | (initial_sp.wrapping_sub(2) as u16));

        assert_eq!(
            pushed_status & flags::CARRY,
            flags::CARRY,
            "Carry should be preserved"
        );
        assert_eq!(
            pushed_status & flags::ZERO,
            flags::ZERO,
            "Zero should be preserved"
        );
        assert_eq!(
            pushed_status & flags::OVERFLOW,
            flags::OVERFLOW,
            "Overflow should be preserved"
        );
        assert_eq!(
            pushed_status & flags::NEGATIVE,
            flags::NEGATIVE,
            "Negative should be preserved"
        );
        assert_eq!(
            pushed_status & flags::DECIMAL,
            flags::DECIMAL,
            "Decimal should be preserved"
        );
    }

    // ========================================
    // RTI (Return from Interrupt) Tests
    // ========================================

    #[test]
    fn test_rti_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Simulate interrupt entry: push PC then status
        let original_pc = 0x1234;
        let original_status = 0b11000111; // N, V, I, Z, C set

        // Manually push PC and status to stack (as interrupt would do)
        // Order: PC high, PC low, status
        cpu.stack_push_u16(&mut bus, original_pc);
        cpu.stack_push(&mut bus, original_status);

        let sp_after_push = cpu.sp;

        // Change CPU state
        cpu.pc = 0x8000;
        cpu.status = 0x00;
        cpu.set_flag(flags::UNUSED); // UNUSED must always be set

        // Execute RTI
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.rti(&bus, &addr_result);

        assert_eq!(cycles, 0, "RTI should not return additional cycles");

        // Verify PC was restored
        assert_eq!(cpu.pc, original_pc, "PC should be restored to 0x1234");

        // Verify status was restored (with UNUSED always set)
        let expected_status = original_status | flags::UNUSED;
        assert_eq!(
            cpu.status, expected_status,
            "Status should be restored from stack"
        );

        // Verify individual flags
        assert!(cpu.get_negative(), "Negative flag should be set");
        assert!(cpu.get_overflow(), "Overflow flag should be set");
        assert!(cpu.get_interrupt_disable(), "Interrupt flag should be set");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(cpu.get_carry(), "Carry flag should be set");

        // Verify stack pointer was restored
        assert_eq!(
            cpu.sp,
            sp_after_push.wrapping_add(3),
            "SP should be restored (increment by 3)"
        );
    }

    #[test]
    fn test_rti_restores_all_flags() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Save the current B flag
        let initial_b_flag = cpu.get_break();

        // Push PC and status pattern (as interrupt would do)
        let status_pattern = 0b11111010; // N, V, UNUSED, B, D, Z set; I, C clear
        cpu.stack_push_u16(&mut bus, 0x5678);
        cpu.stack_push(&mut bus, status_pattern);

        // Change all flags
        cpu.status = 0b01000101; // Different pattern
        cpu.set_flag(flags::UNUSED);

        // Execute RTI
        cpu.rti(&bus, &AddressingResult::new(0));

        // Verify flags match pattern (with UNUSED always set, and B flag preserved)
        // B flag should remain as it was before RTI (ignored from stack)
        let expected_status = (status_pattern | flags::UNUSED) & !flags::BREAK
            | (if initial_b_flag { flags::BREAK } else { 0 });

        assert_eq!(
            cpu.status, expected_status,
            "All flags should be restored except B flag which is preserved"
        );

        // Verify individual flags (except B)
        assert!(cpu.get_negative(), "Negative should be set");
        assert!(cpu.get_overflow(), "Overflow should be set");
        assert!(!cpu.get_interrupt_disable(), "Interrupt should be clear");
        assert!(cpu.get_decimal(), "Decimal should be set");
        assert!(cpu.get_zero(), "Zero should be set");
        assert!(!cpu.get_carry(), "Carry should be clear");
    }

    #[test]
    fn test_rti_unused_flag_always_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push PC and status with UNUSED flag clear (shouldn't normally happen)
        let status_no_unused = 0b00000001; // Only carry set, UNUSED clear
        cpu.stack_push_u16(&mut bus, 0x1000);
        cpu.stack_push(&mut bus, status_no_unused);

        // Execute RTI
        cpu.rti(&bus, &AddressingResult::new(0));

        // UNUSED flag must be set
        assert!(
            cpu.get_flag(flags::UNUSED),
            "UNUSED flag must always be set after RTI"
        );
    }

    #[test]
    fn test_rti_restores_pc_correctly() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let test_cases = [0x0000, 0x1234, 0x8000, 0xFFFF];

        for &test_pc in &test_cases {
            cpu.sp = 0xFD; // Reset stack pointer

            // Push PC and status (as interrupt would do)
            cpu.stack_push_u16(&mut bus, test_pc);
            cpu.stack_push(&mut bus, 0x00);

            // Execute RTI
            cpu.rti(&bus, &AddressingResult::new(0));

            assert_eq!(
                cpu.pc, test_pc,
                "PC should be restored to 0x{:04X}",
                test_pc
            );
        }
    }

    #[test]
    fn test_rti_ignores_b_flag() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set B flag in CPU to true
        cpu.set_break(true);
        let initial_b_flag = cpu.get_break();

        // Push PC and status with B flag clear (opposite of CPU's B flag)
        let status_with_b_clear = flags::CARRY; // B flag is clear, carry is set
        cpu.stack_push_u16(&mut bus, 0x1000);
        cpu.stack_push(&mut bus, status_with_b_clear);

        // Execute RTI
        cpu.rti(&bus, &AddressingResult::new(0));

        // Like PLP, RTI should ignore the B flag from stack and preserve the original
        assert_eq!(
            cpu.get_break(),
            initial_b_flag,
            "RTI should ignore B flag from stack (like PLP)"
        );

        // But other flags should be restored
        assert!(cpu.get_carry(), "Carry flag should be restored");
    }

    // ========================================
    // BRK/RTI Integration Tests
    // ========================================

    #[test]
    fn test_brk_rti_roundtrip() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector to point to interrupt handler
        let irq_handler = 0x8000;
        bus.write(0xFFFE, (irq_handler & 0xFF) as u8);
        bus.write(0xFFFF, (irq_handler >> 8) as u8);

        // Set initial state
        let original_pc = 0x1000;
        cpu.pc = original_pc;
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);
        cpu.set_interrupt_disable(false);

        let original_carry = cpu.get_carry();
        let original_zero = cpu.get_zero();
        let original_overflow = cpu.get_overflow();
        let original_negative = cpu.get_negative();

        // Execute BRK
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Verify we jumped to interrupt handler
        assert_eq!(cpu.pc, irq_handler, "Should jump to IRQ handler");

        // Verify I flag is set
        assert!(cpu.get_interrupt_disable(), "I flag should be set");

        // Now simulate returning from interrupt with RTI
        cpu.rti(&bus, &AddressingResult::new(0));

        // Verify PC is restored to PC+2 (because BRK pushes PC+2)
        assert_eq!(
            cpu.pc,
            original_pc.wrapping_add(2),
            "PC should be restored to original PC+2"
        );

        // Verify flags are restored (except I might differ)
        assert_eq!(
            cpu.get_carry(),
            original_carry,
            "Carry flag should be restored"
        );
        assert_eq!(
            cpu.get_zero(),
            original_zero,
            "Zero flag should be restored"
        );
        assert_eq!(
            cpu.get_overflow(),
            original_overflow,
            "Overflow flag should be restored"
        );
        assert_eq!(
            cpu.get_negative(),
            original_negative,
            "Negative flag should be restored"
        );
    }

    #[test]
    fn test_nested_interrupts() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80);

        let initial_sp = cpu.sp;

        // First interrupt (BRK)
        cpu.pc = 0x1000;
        cpu.set_carry(true);
        cpu.brk(&mut bus, &AddressingResult::new(0));

        let sp_after_first_brk = cpu.sp;

        // Second interrupt (BRK) - nested
        cpu.pc = 0x2000;
        cpu.set_zero(true);
        cpu.brk(&mut bus, &AddressingResult::new(0));

        // Stack should have 6 bytes pushed (3 per BRK)
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(6));

        // Return from second interrupt
        cpu.rti(&bus, &AddressingResult::new(0));
        assert_eq!(
            cpu.pc, 0x2002,
            "Should return from second interrupt to 0x2002"
        );
        assert_eq!(cpu.sp, sp_after_first_brk);

        // Return from first interrupt
        cpu.rti(&bus, &AddressingResult::new(0));
        assert_eq!(
            cpu.pc, 0x1002,
            "Should return from first interrupt to 0x1002"
        );
        assert_eq!(cpu.sp, initial_sp);
    }
}
