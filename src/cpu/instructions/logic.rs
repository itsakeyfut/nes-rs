// Logic and bit operation instructions for 6502 CPU

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Logical Instructions
    // ========================================

    /// AND - Logical AND
    ///
    /// Performs a bitwise AND operation between the accumulator and a value from memory,
    /// storing the result in the accumulator.
    ///
    /// Formula: A = A & M
    ///
    /// This instruction is commonly used for:
    /// - Masking bits (clearing specific bits while preserving others)
    /// - Testing if specific bits are set
    /// - Isolating bit patterns
    ///
    /// Flags affected: Z, N
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn and(&mut self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.a &= value;
        self.update_zero_and_negative_flags(self.a);
    }

    /// ORA - Logical OR
    ///
    /// Performs a bitwise OR operation between the accumulator and a value from memory,
    /// storing the result in the accumulator.
    ///
    /// Formula: A = A | M
    ///
    /// This instruction is commonly used for:
    /// - Setting specific bits while preserving others
    /// - Combining bit patterns
    /// - Merging flags or options
    ///
    /// Flags affected: Z, N
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn ora(&mut self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.a |= value;
        self.update_zero_and_negative_flags(self.a);
    }

    /// EOR - Exclusive OR
    ///
    /// Performs a bitwise exclusive OR (XOR) operation between the accumulator and a value
    /// from memory, storing the result in the accumulator.
    ///
    /// Formula: A = A ^ M
    ///
    /// This instruction is commonly used for:
    /// - Toggling specific bits
    /// - Simple encryption/decryption
    /// - Comparing values (A EOR A = 0)
    /// - Swapping values in combination with other operations
    ///
    /// Flags affected: Z, N
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn eor(&mut self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.a ^= value;
        self.update_zero_and_negative_flags(self.a);
    }

    // ========================================
    // Bit Testing Instructions
    // ========================================

    /// BIT - Bit Test
    ///
    /// Tests bits in memory with the accumulator without modifying the accumulator.
    /// This instruction has unique flag behavior compared to other logical operations.
    ///
    /// The BIT instruction is special because:
    /// - It performs A & M but doesn't store the result
    /// - It directly copies bits 6 and 7 from memory to V and N flags
    /// - It sets Z flag based on the AND result
    ///
    /// This is commonly used for:
    /// - Testing the status of hardware registers
    /// - Checking specific bit patterns without modifying the accumulator
    /// - Reading overflow and negative flags from memory-mapped I/O
    ///
    /// Flags affected: Z, V, N
    /// - Z: Set if (A AND M) == 0 (result of AND is zero)
    /// - V: Set to bit 6 of the memory value (not the AND result)
    /// - N: Set to bit 7 of the memory value (not the AND result)
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address
    ///
    /// # Note
    /// BIT instruction does not support immediate addressing mode.
    /// It always reads from memory, never from an immediate value.
    pub fn bit(&mut self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);

        // Test if A & M is zero
        let result = self.a & value;
        self.set_zero(result == 0);

        // Copy bit 6 of memory value to V flag
        self.set_overflow((value & 0x40) != 0);

        // Copy bit 7 of memory value to N flag
        self.set_negative((value & 0x80) != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;

    // ========================================
    // Logical Instruction Tests - AND
    // ========================================

    #[test]
    fn test_and_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1111_0000;
        let addr_result = AddressingResult::immediate(0b1010_1010);
        cpu.and(&mut bus, &addr_result);

        assert_eq!(
            cpu.a, 0b1010_0000,
            "0b1111_0000 AND 0b1010_1010 should equal 0b1010_0000"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 is 1)"
        );
    }

    #[test]
    fn test_and_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0000_1111;
        let addr_result = AddressingResult::immediate(0b1111_0000);
        cpu.and(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "0b0000_1111 AND 0b1111_0000 should equal 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_and_masking() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Common use case: Masking lower 4 bits
        cpu.a = 0xAB;
        let addr_result = AddressingResult::immediate(0x0F);
        cpu.and(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x0B, "Should mask lower 4 bits");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_and_all_ones() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.and(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "AND with 0xFF should preserve value");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_and_all_zeros() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.and(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "AND with 0x00 should clear accumulator");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // Logical Instruction Tests - ORA
    // ========================================

    #[test]
    fn test_ora_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1111_0000;
        let addr_result = AddressingResult::immediate(0b0000_1111);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(
            cpu.a, 0b1111_1111,
            "0b1111_0000 OR 0b0000_1111 should equal 0b1111_1111"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 is 1)"
        );
    }

    #[test]
    fn test_ora_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "0x00 OR 0x00 should equal 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ora_set_bits() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Common use case: Setting specific bits
        cpu.a = 0b0000_0001;
        let addr_result = AddressingResult::immediate(0b0000_0010);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0b0000_0011, "Should set both bits");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ora_with_zero() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "OR with 0x00 should preserve value");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ora_with_all_ones() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0xFF, "OR with 0xFF should set all bits");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_ora_combining_values() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1010_0000;
        let addr_result = AddressingResult::immediate(0b0101_0101);
        cpu.ora(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0b1111_0101, "Should combine bit patterns");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // Logical Instruction Tests - EOR
    // ========================================

    #[test]
    fn test_eor_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1111_0000;
        let addr_result = AddressingResult::immediate(0b1010_1010);
        cpu.eor(&mut bus, &addr_result);

        assert_eq!(
            cpu.a, 0b0101_1010,
            "0b1111_0000 XOR 0b1010_1010 should equal 0b0101_1010"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_eor_same_value() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0x42);
        cpu.eor(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "XOR with same value should equal 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_eor_toggle_bits() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Common use case: Toggling specific bits
        cpu.a = 0b1111_0000;
        let addr_result = AddressingResult::immediate(0b0000_1111);
        cpu.eor(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0b1111_1111, "Should toggle lower 4 bits");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_eor_with_zero() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.eor(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "XOR with 0x00 should preserve value");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_eor_with_all_ones() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1010_1010;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.eor(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0b0101_0101, "XOR with 0xFF should invert all bits");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_eor_double_toggle() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // XOR twice with same value returns to original
        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0xAA);

        cpu.eor(&mut bus, &addr_result);
        let intermediate = cpu.a;
        cpu.eor(&mut bus, &addr_result);

        assert_ne!(intermediate, 0x42, "First XOR should change value");
        assert_eq!(cpu.a, 0x42, "Second XOR should restore original value");
    }

    // ========================================
    // Bit Testing Instruction Tests - BIT
    // ========================================

    #[test]
    fn test_bit_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test when A & M is non-zero
        cpu.a = 0b1111_1111;
        let addr_result = AddressingResult::immediate(0b1010_1010);
        cpu.bit(&mut bus, &addr_result);

        assert!(!cpu.get_zero(), "Zero flag should be clear (A & M != 0)");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 of M is 1)"
        );
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (bit 6 of M is 0)"
        );
    }

    #[test]
    fn test_bit_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test when A & M equals zero
        cpu.a = 0b0000_1111;
        let addr_result = AddressingResult::immediate(0b1111_0000);
        cpu.bit(&mut bus, &addr_result);

        assert!(cpu.get_zero(), "Zero flag should be set (A & M == 0)");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 of M is 1)"
        );
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (bit 6 of M is 1)"
        );
    }

    #[test]
    fn test_bit_flags_from_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test that V and N are set from memory, not from AND result
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0b1100_0000); // Bits 7 and 6 set
        cpu.bit(&mut bus, &addr_result);

        assert!(!cpu.get_zero(), "Zero flag should be clear (A & M != 0)");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 of memory is 1)"
        );
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (bit 6 of memory is 1)"
        );
    }

    #[test]
    fn test_bit_preserves_accumulator() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.bit(&mut bus, &addr_result);

        assert_eq!(cpu.a, 0x42, "BIT should not modify accumulator");
    }

    #[test]
    fn test_bit_all_flags() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test all flag combinations
        // Memory: 0b0011_1111 (bits 7 and 6 are 0)
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0b0011_1111);
        cpu.bit(&mut bus, &addr_result);

        assert!(!cpu.get_zero(), "Zero flag should be clear (A & M != 0)");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear (bit 7 of M is 0)"
        );
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (bit 6 of M is 0)"
        );
    }

    #[test]
    fn test_bit_only_bit_7() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Memory: 0b1000_0000 (only bit 7 set)
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0b1000_0000);
        cpu.bit(&mut bus, &addr_result);

        assert!(!cpu.get_zero(), "Zero flag should be clear (A & M != 0)");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 of M is 1)"
        );
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (bit 6 of M is 0)"
        );
    }

    #[test]
    fn test_bit_only_bit_6() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Memory: 0b0100_0000 (only bit 6 set)
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0b0100_0000);
        cpu.bit(&mut bus, &addr_result);

        assert!(!cpu.get_zero(), "Zero flag should be clear (A & M != 0)");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear (bit 7 of M is 0)"
        );
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (bit 6 of M is 1)"
        );
    }

    #[test]
    fn test_bit_zero_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.bit(&mut bus, &addr_result);

        assert!(cpu.get_zero(), "Zero flag should be set (A & M == 0)");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear (bit 7 of M is 0)"
        );
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (bit 6 of M is 0)"
        );
    }

    // ========================================
    // Integration and Combination Tests
    // ========================================

    #[test]
    fn test_logical_operations_combination() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Test a sequence of logical operations
        cpu.a = 0b0000_0000;

        // Set some bits with ORA
        let addr_result = AddressingResult::immediate(0b1111_0000);
        cpu.ora(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b1111_0000);

        // Set more bits with ORA
        let addr_result = AddressingResult::immediate(0b0000_1111);
        cpu.ora(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b1111_1111);

        // Clear some bits with AND
        let addr_result = AddressingResult::immediate(0b1010_1010);
        cpu.and(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b1010_1010);

        // Toggle bits with EOR
        let addr_result = AddressingResult::immediate(0b1111_1111);
        cpu.eor(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b0101_0101);
    }

    #[test]
    fn test_bit_masking_pattern() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Common pattern: Check if specific bits are set using BIT
        let status_register = 0b1100_0001;
        cpu.a = 0b1000_0000; // Testing bit 7

        let addr_result = AddressingResult::immediate(status_register);
        cpu.bit(&mut bus, &addr_result);

        // A & status_register = 0b1000_0000 & 0b1100_0001 = 0b1000_0000 (non-zero)
        assert!(!cpu.get_zero(), "Should detect bit 7 is set");
        assert!(cpu.get_negative(), "N flag from bit 7 of memory");
        assert!(cpu.get_overflow(), "V flag from bit 6 of memory");

        // Original accumulator unchanged
        assert_eq!(cpu.a, 0b1000_0000);
    }

    #[test]
    fn test_clear_vs_toggle() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Start with a value
        cpu.a = 0b1111_1111;

        // Clear specific bits with AND
        let mask = 0b1111_0000;
        let addr_result = AddressingResult::immediate(mask);
        cpu.and(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b1111_0000, "Lower bits should be cleared");

        // Reset for toggle test
        cpu.a = 0b1111_1111;

        // Toggle specific bits with EOR
        let toggle = 0b0000_1111;
        let addr_result = AddressingResult::immediate(toggle);
        cpu.eor(&mut bus, &addr_result);
        assert_eq!(cpu.a, 0b1111_0000, "Lower bits should be toggled (cleared)");
    }
}
