// Jump and Subroutine instructions for 6502 CPU
// These instructions perform unconditional jumps and subroutine calls/returns.

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

/// Stack base address (Stack lives at $0100-$01FF)
const STACK_BASE: u16 = 0x0100;

impl Cpu {
    // ========================================
    // Stack Helper Methods
    // ========================================

    /// Push a byte onto the stack
    ///
    /// The 6502 stack grows downward from $01FF to $0100.
    /// SP points to the next free location (empty stack has SP=$FF).
    ///
    /// # Arguments
    /// * `bus` - The memory bus for writing the value
    /// * `value` - The byte to push onto the stack
    #[inline]
    pub(crate) fn stack_push(&mut self, bus: &mut Bus, value: u8) {
        let addr = STACK_BASE | (self.sp as u16);
        bus.write(addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pull a byte from the stack
    ///
    /// The 6502 stack grows downward from $01FF to $0100.
    /// SP points to the next free location, so we increment first, then read.
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading the value
    ///
    /// # Returns
    /// The byte pulled from the stack
    #[inline]
    pub(crate) fn stack_pop(&mut self, bus: &Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = STACK_BASE | (self.sp as u16);
        bus.read(addr)
    }

    /// Push a 16-bit value onto the stack (high byte first)
    ///
    /// Used by JSR to push the return address onto the stack.
    /// Pushes high byte first, then low byte (little-endian on stack).
    ///
    /// # Arguments
    /// * `bus` - The memory bus for writing the value
    /// * `value` - The 16-bit value to push onto the stack
    #[inline]
    pub(crate) fn stack_push_u16(&mut self, bus: &mut Bus, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.stack_push(bus, hi);
        self.stack_push(bus, lo);
    }

    /// Pull a 16-bit value from the stack (low byte first)
    ///
    /// Used by RTS to pull the return address from the stack.
    /// Pulls low byte first, then high byte (little-endian on stack).
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading the value
    ///
    /// # Returns
    /// The 16-bit value pulled from the stack
    #[inline]
    pub(crate) fn stack_pop_u16(&mut self, bus: &Bus) -> u16 {
        let lo = self.stack_pop(bus) as u16;
        let hi = self.stack_pop(bus) as u16;
        (hi << 8) | lo
    }

    // ========================================
    // Jump Instructions
    // ========================================

    /// JMP - Jump to Address
    ///
    /// Unconditional jump to the specified address.
    /// Sets the program counter to the target address.
    ///
    /// This instruction supports two addressing modes:
    /// - Absolute: JMP $1234 - Jump directly to address $1234
    /// - Indirect: JMP ($1234) - Jump to address stored at $1234-$1235
    ///
    /// Flags affected: None
    ///
    /// Cycles:
    /// - Absolute: 3 cycles
    /// - Indirect: 5 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - The addressing result containing the jump target
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// JMP $8000   ; Jump to address $8000
    /// JMP ($FFFC) ; Jump to address stored at $FFFC-$FFFD (reset vector)
    /// ```
    ///
    /// # Note
    /// When using indirect addressing, the 6502 has a hardware bug:
    /// If the pointer is at a page boundary ($xxFF), the high byte
    /// wraps to $xx00 instead of $(xx+1)00. This bug is emulated
    /// in the addr_indirect addressing mode implementation.
    pub fn jmp(&mut self, _bus: &Bus, addr_result: &AddressingResult) -> u8 {
        self.pc = addr_result.address;
        0
    }

    // ========================================
    // Subroutine Instructions
    // ========================================

    /// JSR - Jump to Subroutine
    ///
    /// Jumps to a subroutine at the specified address and pushes the return
    /// address onto the stack. The return address is the address of the last
    /// byte of the JSR instruction (PC - 1), not the address of the next
    /// instruction. This is a quirk of the 6502.
    ///
    /// The stack is located at $0100-$01FF, and grows downward. The return
    /// address is pushed high byte first, then low byte.
    ///
    /// Flags affected: None
    ///
    /// Cycles: 6 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations
    /// * `addr_result` - The addressing result containing the subroutine address
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// JSR init_sprite  ; Call subroutine at init_sprite
    ///                  ; Return address is pushed to stack
    /// ```
    ///
    /// # Implementation Note
    /// The 6502 pushes PC-1, not PC. This means that RTS needs to add 1
    /// to the return address it pulls from the stack. This design allows
    /// the programmer to modify the return address on the stack to implement
    /// tricks like computed jumps or skipping bytes.
    pub fn jsr(&mut self, bus: &mut Bus, addr_result: &AddressingResult) -> u8 {
        // Push return address - 1 (current PC - 1)
        // At this point, PC points to the next instruction after JSR
        let return_addr = self.pc.wrapping_sub(1);
        self.stack_push_u16(bus, return_addr);

        // Jump to subroutine address
        self.pc = addr_result.address;
        0
    }

    /// RTS - Return from Subroutine
    ///
    /// Returns from a subroutine by pulling the return address from the stack
    /// and jumping to the next instruction (return address + 1).
    ///
    /// The stack pointer is incremented twice to remove the 16-bit return
    /// address from the stack (low byte first, then high byte).
    ///
    /// Flags affected: None
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
    /// init_sprite:
    ///     LDA #$00
    ///     STA $2003
    ///     RTS          ; Return to caller
    /// ```
    ///
    /// # Implementation Note
    /// Since JSR pushes PC-1, RTS must add 1 to the pulled address to get
    /// the correct return location. This is a 6502 convention.
    pub fn rts(&mut self, bus: &Bus, _addr_result: &AddressingResult) -> u8 {
        // Pull return address from stack
        let return_addr = self.stack_pop_u16(bus);

        // Add 1 to get the actual return address (JSR pushes PC-1)
        self.pc = return_addr.wrapping_add(1);
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;
    use crate::cpu::Cpu;

    // ========================================
    // Stack Helper Tests
    // ========================================

    #[test]
    fn test_stack_push_pop() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Initial stack pointer should be 0xFD
        assert_eq!(cpu.sp, 0xFD);

        // Push a byte
        cpu.stack_push(&mut bus, 0x42);
        assert_eq!(cpu.sp, 0xFC, "SP should decrement after push");

        // Push another byte
        cpu.stack_push(&mut bus, 0x13);
        assert_eq!(cpu.sp, 0xFB, "SP should decrement after second push");

        // Pop bytes (LIFO order)
        let value1 = cpu.stack_pop(&bus);
        assert_eq!(value1, 0x13, "Should pop most recent value first");
        assert_eq!(cpu.sp, 0xFC, "SP should increment after pop");

        let value2 = cpu.stack_pop(&bus);
        assert_eq!(value2, 0x42, "Should pop first value second");
        assert_eq!(cpu.sp, 0xFD, "SP should be back to initial value");
    }

    #[test]
    fn test_stack_push_pop_u16() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let initial_sp = cpu.sp;

        // Push a 16-bit value
        cpu.stack_push_u16(&mut bus, 0x1234);
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(2),
            "SP should decrement by 2"
        );

        // Pop the 16-bit value
        let value = cpu.stack_pop_u16(&bus);
        assert_eq!(value, 0x1234, "Should pop the same 16-bit value");
        assert_eq!(cpu.sp, initial_sp, "SP should be back to initial value");
    }

    #[test]
    fn test_stack_wrapping() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set SP to edge case
        cpu.sp = 0x01;

        // Push should wrap to 0x00
        cpu.stack_push(&mut bus, 0x42);
        assert_eq!(cpu.sp, 0x00, "SP should wrap to 0x00");

        // Push again should wrap to 0xFF
        cpu.stack_push(&mut bus, 0x13);
        assert_eq!(cpu.sp, 0xFF, "SP should wrap to 0xFF");

        // Pop should restore values correctly
        let value1 = cpu.stack_pop(&bus);
        assert_eq!(value1, 0x13);
        assert_eq!(cpu.sp, 0x00);

        let value2 = cpu.stack_pop(&bus);
        assert_eq!(value2, 0x42);
        assert_eq!(cpu.sp, 0x01);
    }

    #[test]
    fn test_stack_location() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Verify stack is at $0100-$01FF
        cpu.sp = 0xFF;
        cpu.stack_push(&mut bus, 0x42);

        // Stack should have written to $01FF
        assert_eq!(bus.read(0x01FF), 0x42);

        cpu.sp = 0x00;
        cpu.stack_push(&mut bus, 0x13);

        // Stack should have written to $0100
        assert_eq!(bus.read(0x0100), 0x13);
    }

    // ========================================
    // JMP Instruction Tests
    // ========================================

    #[test]
    fn test_jmp_absolute() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.pc = 0x0200;

        // Jump to $8000
        let addr_result = AddressingResult::new(0x8000);
        let cycles = cpu.jmp(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x8000, "PC should jump to target address");
        assert_eq!(cycles, 0, "JMP should not return additional cycles");
    }

    #[test]
    fn test_jmp_forward() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.pc = 0x0200;

        // Jump forward
        let addr_result = AddressingResult::new(0x0300);
        cpu.jmp(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jmp_backward() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.pc = 0x0300;

        // Jump backward
        let addr_result = AddressingResult::new(0x0200);
        cpu.jmp(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0200);
    }

    #[test]
    fn test_jmp_to_zero_page() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.pc = 0x8000;

        // Jump to zero page
        let addr_result = AddressingResult::new(0x0080);
        cpu.jmp(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0080);
    }

    #[test]
    fn test_jmp_indirect_normal() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0200;

        // Set up the pointer address in the instruction stream
        bus.write(0x0200, 0x20); // Pointer low byte ($0120)
        bus.write(0x0201, 0x01); // Pointer high byte

        // Set up indirect jump: pointer at $0120 contains $0634
        bus.write(0x0120, 0x34); // Target address low byte
        bus.write(0x0121, 0x06); // Target address high byte

        // Use addr_indirect to get the target address
        let addr_result = cpu.addr_indirect(&bus);

        // Execute JMP
        cpu.jmp(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x0634, "Should jump to address from pointer");
    }

    #[test]
    fn test_jmp_indirect_page_boundary_bug() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0200;

        // Test the 6502 page boundary bug
        // Pointer at $01FF should read high byte from $0100 (not $0200)
        bus.write(0x01FF, 0x34); // Low byte
        bus.write(0x0100, 0x06); // High byte (wraps to start of page)
        bus.write(0x0200, 0xFF); // This should NOT be read

        // Manually set up PC to read from $01FF
        cpu.pc = 0x0200;
        // Set up bus with pointer address
        bus.write(0x0200, 0xFF); // Pointer low byte
        bus.write(0x0201, 0x01); // Pointer high byte

        // Use addr_indirect to get the target address (bug is emulated here)
        let addr_result = cpu.addr_indirect(&bus);

        assert_eq!(
            addr_result.address, 0x0634,
            "Should read high byte from $0100 (page boundary bug)"
        );

        // Execute JMP
        cpu.jmp(&bus, &addr_result);
        assert_eq!(cpu.pc, 0x0634);
    }

    #[test]
    fn test_jmp_no_flag_modification() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_overflow(true);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute JMP
        let addr_result = AddressingResult::new(0x8000);
        cpu.jmp(&bus, &addr_result);

        assert_eq!(
            cpu.status, initial_status,
            "JMP should not modify any flags"
        );
    }

    // ========================================
    // JSR Instruction Tests
    // ========================================

    #[test]
    fn test_jsr_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0203; // After reading JSR operands, PC points to next instruction
        let initial_sp = cpu.sp;

        // Call subroutine at $8000
        let addr_result = AddressingResult::new(0x8000);
        let cycles = cpu.jsr(&mut bus, &addr_result);

        assert_eq!(cpu.pc, 0x8000, "PC should jump to subroutine address");
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(2),
            "SP should decrement by 2"
        );
        assert_eq!(cycles, 0, "JSR should not return additional cycles");

        // Verify return address on stack (should be PC-1 = 0x0202)
        // Stack grows downward, so high byte is at higher address
        let stack_lo = bus.read(0x0100 | ((initial_sp.wrapping_sub(1)) as u16));
        let stack_hi = bus.read(0x0100 | (initial_sp as u16));

        let return_addr = ((stack_hi as u16) << 8) | (stack_lo as u16);
        assert_eq!(
            return_addr, 0x0202,
            "Return address should be PC-1 (0x0202)"
        );
    }

    #[test]
    fn test_jsr_multiple_calls() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let initial_sp = cpu.sp;

        // First call
        cpu.pc = 0x0203;
        let addr_result = AddressingResult::new(0x8000);
        cpu.jsr(&mut bus, &addr_result);

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(2));

        // Second call (nested)
        cpu.pc = 0x8005;
        let addr_result = AddressingResult::new(0x9000);
        cpu.jsr(&mut bus, &addr_result);

        assert_eq!(cpu.pc, 0x9000);
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(4));
    }

    #[test]
    fn test_jsr_no_flag_modification() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0203;

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_overflow(false);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute JSR
        let addr_result = AddressingResult::new(0x8000);
        cpu.jsr(&mut bus, &addr_result);

        assert_eq!(
            cpu.status, initial_status,
            "JSR should not modify any flags"
        );
    }

    // ========================================
    // RTS Instruction Tests
    // ========================================

    #[test]
    fn test_rts_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Simulate JSR: push return address - 1
        let return_addr = 0x0202;
        cpu.stack_push_u16(&mut bus, return_addr);

        let sp_after_jsr = cpu.sp;

        // Execute RTS
        let addr_result = AddressingResult::new(0); // Unused for RTS
        let cycles = cpu.rts(&bus, &addr_result);

        assert_eq!(
            cpu.pc,
            return_addr.wrapping_add(1),
            "PC should be return address + 1"
        );
        assert_eq!(
            cpu.sp,
            sp_after_jsr.wrapping_add(2),
            "SP should increment by 2"
        );
        assert_eq!(cycles, 0, "RTS should not return additional cycles");
    }

    #[test]
    fn test_rts_no_flag_modification() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Push a return address
        cpu.stack_push_u16(&mut bus, 0x0202);

        // Set all flags to known state
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute RTS
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);

        assert_eq!(
            cpu.status, initial_status,
            "RTS should not modify any flags"
        );
    }

    // ========================================
    // JSR/RTS Integration Tests
    // ========================================

    #[test]
    fn test_jsr_rts_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let initial_pc = 0x0203;
        let subroutine_addr = 0x8000;

        cpu.pc = initial_pc;
        let initial_sp = cpu.sp;

        // Execute JSR
        let addr_result = AddressingResult::new(subroutine_addr);
        cpu.jsr(&mut bus, &addr_result);

        assert_eq!(cpu.pc, subroutine_addr, "Should jump to subroutine");

        // Execute RTS
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);

        assert_eq!(cpu.pc, initial_pc, "Should return to original PC");
        assert_eq!(cpu.sp, initial_sp, "SP should be restored");
    }

    #[test]
    fn test_nested_jsr_rts() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Main program
        cpu.pc = 0x0203;
        let main_pc = cpu.pc;
        let initial_sp = cpu.sp;

        // First JSR
        let addr_result = AddressingResult::new(0x8000);
        cpu.jsr(&mut bus, &addr_result);
        assert_eq!(cpu.pc, 0x8000);

        // Nested JSR from first subroutine
        cpu.pc = 0x8005;
        let first_sub_pc = cpu.pc;
        let addr_result = AddressingResult::new(0x9000);
        cpu.jsr(&mut bus, &addr_result);
        assert_eq!(cpu.pc, 0x9000);

        // Return from nested subroutine
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);
        assert_eq!(cpu.pc, first_sub_pc, "Should return to first subroutine");

        // Return from first subroutine
        cpu.rts(&bus, &addr_result);
        assert_eq!(cpu.pc, main_pc, "Should return to main program");
        assert_eq!(cpu.sp, initial_sp, "SP should be fully restored");
    }

    #[test]
    fn test_jsr_rts_with_stack_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Main program
        cpu.pc = 0x0203;
        let main_pc = cpu.pc;

        // JSR to subroutine
        let addr_result = AddressingResult::new(0x8000);
        cpu.jsr(&mut bus, &addr_result);

        // Simulate some stack operations in subroutine (PHA/PLA)
        cpu.stack_push(&mut bus, 0x42);
        cpu.stack_push(&mut bus, 0x13);

        let val1 = cpu.stack_pop(&bus);
        let val2 = cpu.stack_pop(&bus);

        assert_eq!(val1, 0x13);
        assert_eq!(val2, 0x42);

        // RTS should still work correctly
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);

        assert_eq!(cpu.pc, main_pc, "Should return to correct address");
    }

    #[test]
    fn test_jsr_rts_extreme_nesting() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0203;
        let initial_sp = cpu.sp;

        // Nest 10 subroutine calls
        let addresses = [
            0x8000, 0x8100, 0x8200, 0x8300, 0x8400, 0x8500, 0x8600, 0x8700, 0x8800, 0x8900,
        ];

        for &addr in &addresses {
            cpu.pc = cpu.pc.wrapping_add(2); // Simulate reading operands
            let addr_result = AddressingResult::new(addr);
            cpu.jsr(&mut bus, &addr_result);
            assert_eq!(cpu.pc, addr);
        }

        // SP should have moved down by 20 bytes (10 calls × 2 bytes)
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(20));

        // Return from all 10 subroutines
        for _ in 0..10 {
            let addr_result = AddressingResult::new(0);
            cpu.rts(&bus, &addr_result);
        }

        // SP should be back to initial value
        assert_eq!(cpu.sp, initial_sp);
    }

    #[test]
    fn test_jsr_to_zero_page() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x8003;

        // JSR can jump to zero page (though unusual)
        let addr_result = AddressingResult::new(0x0080);
        cpu.jsr(&mut bus, &addr_result);

        assert_eq!(cpu.pc, 0x0080);

        // And return should work
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);

        assert_eq!(cpu.pc, 0x8003);
    }

    #[test]
    fn test_rts_increments_correctly() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test that RTS adds 1 to the pulled address
        // This is critical for correct 6502 behavior

        // Manually push a return address (PC-1)
        cpu.stack_push_u16(&mut bus, 0x1234);

        // Execute RTS
        let addr_result = AddressingResult::new(0);
        cpu.rts(&bus, &addr_result);

        // PC should be 0x1234 + 1 = 0x1235
        assert_eq!(
            cpu.pc, 0x1235,
            "RTS must add 1 to the pulled address (6502 convention)"
        );
    }

    #[test]
    fn test_jsr_pushes_pc_minus_one() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set PC to point to the byte after JSR instruction
        cpu.pc = 0x0203;

        // Execute JSR
        let addr_result = AddressingResult::new(0x8000);
        cpu.jsr(&mut bus, &addr_result);

        // Read the value pushed to stack
        let pushed_addr = cpu.stack_pop_u16(&bus);

        // Should be PC - 1 = 0x0202
        assert_eq!(pushed_addr, 0x0202, "JSR must push PC-1 (6502 convention)");
    }
}
