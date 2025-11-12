// Comparison instructions for 6502 CPU
// These instructions perform subtraction without storing the result,
// only updating the processor status flags.

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Comparison Instructions
    // ========================================

    /// CMP - Compare Accumulator
    ///
    /// Compares the accumulator with a value from memory by performing
    /// subtraction (A - M) without storing the result.
    ///
    /// The comparison is performed by subtracting the memory value from the
    /// accumulator and setting flags based on the result:
    /// - Carry (C): Set if A >= M (no borrow needed)
    /// - Zero (Z): Set if A == M (result is zero)
    /// - Negative (N): Set if bit 7 of the result is 1
    ///
    /// The accumulator is not modified.
    ///
    /// Flags affected: C, Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    ///
    /// # Example
    /// ```text
    /// A = 0x50, M = 0x30
    /// CMP: A - M = 0x50 - 0x30 = 0x20
    /// Result: C=1 (A >= M), Z=0 (A != M), N=0 (bit 7 is 0)
    /// ```
    pub fn cmp(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.compare(self.a, value);
    }

    /// CPX - Compare X Register
    ///
    /// Compares the X register with a value from memory by performing
    /// subtraction (X - M) without storing the result.
    ///
    /// The comparison is performed by subtracting the memory value from the
    /// X register and setting flags based on the result:
    /// - Carry (C): Set if X >= M (no borrow needed)
    /// - Zero (Z): Set if X == M (result is zero)
    /// - Negative (N): Set if bit 7 of the result is 1
    ///
    /// The X register is not modified.
    ///
    /// Flags affected: C, Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    ///
    /// # Example
    /// ```text
    /// X = 0x30, M = 0x50
    /// CPX: X - M = 0x30 - 0x50 = 0xE0 (wraps around)
    /// Result: C=0 (X < M), Z=0 (X != M), N=1 (bit 7 is 1)
    /// ```
    pub fn cpx(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.compare(self.x, value);
    }

    /// CPY - Compare Y Register
    ///
    /// Compares the Y register with a value from memory by performing
    /// subtraction (Y - M) without storing the result.
    ///
    /// The comparison is performed by subtracting the memory value from the
    /// Y register and setting flags based on the result:
    /// - Carry (C): Set if Y >= M (no borrow needed)
    /// - Zero (Z): Set if Y == M (result is zero)
    /// - Negative (N): Set if bit 7 of the result is 1
    ///
    /// The Y register is not modified.
    ///
    /// Flags affected: C, Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    ///
    /// # Example
    /// ```text
    /// Y = 0x50, M = 0x50
    /// CPY: Y - M = 0x50 - 0x50 = 0x00
    /// Result: C=1 (Y >= M), Z=1 (Y == M), N=0 (bit 7 is 0)
    /// ```
    pub fn cpy(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        self.compare(self.y, value);
    }

    // ========================================
    // Helper Methods
    // ========================================

    /// Internal comparison helper method
    ///
    /// Performs the actual comparison logic used by CMP, CPX, and CPY.
    /// Subtracts the memory value from the register value and sets flags
    /// accordingly.
    ///
    /// # Arguments
    /// * `register_value` - The value from the register (A, X, or Y)
    /// * `memory_value` - The value from memory to compare against
    #[inline]
    fn compare(&mut self, register_value: u8, memory_value: u8) {
        // Perform subtraction: register - memory
        let result = register_value.wrapping_sub(memory_value);

        // Set Carry flag if register >= memory (no borrow needed)
        // This is true when the subtraction doesn't underflow
        self.set_carry(register_value >= memory_value);

        // Set Zero flag if register == memory (result is 0)
        self.set_zero(result == 0);

        // Set Negative flag if bit 7 of result is 1
        self.set_negative((result & 0x80) != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;

    // ========================================
    // CMP Instruction Tests
    // ========================================

    #[test]
    fn test_cmp_equal() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x50;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cmp(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when A == M (A >= M)"
        );
        assert!(cpu.get_zero(), "Zero flag should be set when A == M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when result is 0"
        );
    }

    #[test]
    fn test_cmp_greater() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x50;
        let addr_result = AddressingResult::immediate(0x30);
        cpu.cmp(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when A > M (A >= M)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when A != M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when A > M (positive result)"
        );
    }

    #[test]
    fn test_cmp_less() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x30;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cmp(&bus, &addr_result);

        assert!(
            !cpu.get_carry(),
            "Carry flag should be clear when A < M (borrow needed)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when A != M");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when A < M (underflow, bit 7 is 1)"
        );
    }

    #[test]
    fn test_cmp_zero_equal() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.cmp(&bus, &addr_result);

        assert!(cpu.get_carry(), "Carry flag should be set when A == M");
        assert!(cpu.get_zero(), "Zero flag should be set when A == M");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_cmp_accumulator_not_modified() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x42;
        let addr_result = AddressingResult::immediate(0x10);
        cpu.cmp(&bus, &addr_result);

        assert_eq!(
            cpu.a, 0x42,
            "Accumulator should not be modified by CMP instruction"
        );
    }

    #[test]
    fn test_cmp_max_values() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.cmp(&bus, &addr_result);

        assert!(cpu.get_carry(), "Carry flag should be set when equal");
        assert!(cpu.get_zero(), "Zero flag should be set when equal");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_cmp_boundary_case_80() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // 0x80 - 0x00 = 0x80 (bit 7 is set)
        cpu.a = 0x80;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.cmp(&bus, &addr_result);

        assert!(cpu.get_carry(), "Carry flag should be set when A > M");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (result 0x80 has bit 7 set)"
        );
    }

    // ========================================
    // CPX Instruction Tests
    // ========================================

    #[test]
    fn test_cpx_equal() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.x = 0x50;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cpx(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when X == M (X >= M)"
        );
        assert!(cpu.get_zero(), "Zero flag should be set when X == M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when result is 0"
        );
    }

    #[test]
    fn test_cpx_greater() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.x = 0x50;
        let addr_result = AddressingResult::immediate(0x30);
        cpu.cpx(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when X > M (X >= M)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when X != M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when X > M (positive result)"
        );
    }

    #[test]
    fn test_cpx_less() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.x = 0x30;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cpx(&bus, &addr_result);

        assert!(
            !cpu.get_carry(),
            "Carry flag should be clear when X < M (borrow needed)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when X != M");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when X < M (underflow, bit 7 is 1)"
        );
    }

    #[test]
    fn test_cpx_register_not_modified() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.x = 0x42;
        let addr_result = AddressingResult::immediate(0x10);
        cpu.cpx(&bus, &addr_result);

        assert_eq!(
            cpu.x, 0x42,
            "X register should not be modified by CPX instruction"
        );
    }

    #[test]
    fn test_cpx_zero_values() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.x = 0x00;
        let addr_result = AddressingResult::immediate(0x00);
        cpu.cpx(&bus, &addr_result);

        assert!(cpu.get_carry(), "Carry flag should be set when X == M");
        assert!(cpu.get_zero(), "Zero flag should be set when X == M");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // CPY Instruction Tests
    // ========================================

    #[test]
    fn test_cpy_equal() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.y = 0x50;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cpy(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when Y == M (Y >= M)"
        );
        assert!(cpu.get_zero(), "Zero flag should be set when Y == M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when result is 0"
        );
    }

    #[test]
    fn test_cpy_greater() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.y = 0x50;
        let addr_result = AddressingResult::immediate(0x30);
        cpu.cpy(&bus, &addr_result);

        assert!(
            cpu.get_carry(),
            "Carry flag should be set when Y > M (Y >= M)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when Y != M");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when Y > M (positive result)"
        );
    }

    #[test]
    fn test_cpy_less() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.y = 0x30;
        let addr_result = AddressingResult::immediate(0x50);
        cpu.cpy(&bus, &addr_result);

        assert!(
            !cpu.get_carry(),
            "Carry flag should be clear when Y < M (borrow needed)"
        );
        assert!(!cpu.get_zero(), "Zero flag should be clear when Y != M");
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when Y < M (underflow, bit 7 is 1)"
        );
    }

    #[test]
    fn test_cpy_register_not_modified() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.y = 0x42;
        let addr_result = AddressingResult::immediate(0x10);
        cpu.cpy(&bus, &addr_result);

        assert_eq!(
            cpu.y, 0x42,
            "Y register should not be modified by CPY instruction"
        );
    }

    #[test]
    fn test_cpy_max_values() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.y = 0xFF;
        let addr_result = AddressingResult::immediate(0xFF);
        cpu.cpy(&bus, &addr_result);

        assert!(cpu.get_carry(), "Carry flag should be set when Y == M");
        assert!(cpu.get_zero(), "Zero flag should be set when Y == M");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // Edge Cases and Integration Tests
    // ========================================

    #[test]
    fn test_compare_all_registers() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Set all registers to same value
        cpu.a = 0x42;
        cpu.x = 0x42;
        cpu.y = 0x42;

        // Compare all with same value
        let addr_result = AddressingResult::immediate(0x42);

        cpu.cmp(&bus, &addr_result);
        assert!(cpu.get_carry(), "CMP: Carry should be set");
        assert!(cpu.get_zero(), "CMP: Zero should be set");
        assert!(!cpu.get_negative(), "CMP: Negative should be clear");

        cpu.cpx(&bus, &addr_result);
        assert!(cpu.get_carry(), "CPX: Carry should be set");
        assert!(cpu.get_zero(), "CPX: Zero should be set");
        assert!(!cpu.get_negative(), "CPX: Negative should be clear");

        cpu.cpy(&bus, &addr_result);
        assert!(cpu.get_carry(), "CPY: Carry should be set");
        assert!(cpu.get_zero(), "CPY: Zero should be set");
        assert!(!cpu.get_negative(), "CPY: Negative should be clear");

        // Verify registers are not modified
        assert_eq!(cpu.a, 0x42, "Accumulator should not be modified");
        assert_eq!(cpu.x, 0x42, "X register should not be modified");
        assert_eq!(cpu.y, 0x42, "Y register should not be modified");
    }

    #[test]
    fn test_comparison_loop_simulation() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Simulate a loop counter comparison (common pattern in 6502 code)
        // Loop from 0 to 10
        cpu.x = 0x00;
        let target = 0x0A;

        for i in 0..=10 {
            cpu.x = i;
            let addr_result = AddressingResult::immediate(target);
            cpu.cpx(&bus, &addr_result);

            if i < target {
                assert!(
                    !cpu.get_carry(),
                    "Carry should be clear when counter < target"
                );
                assert!(!cpu.get_zero(), "Zero should be clear when not equal");
            } else if i == target {
                assert!(
                    cpu.get_carry(),
                    "Carry should be set when counter == target"
                );
                assert!(cpu.get_zero(), "Zero should be set when equal");
            } else {
                assert!(cpu.get_carry(), "Carry should be set when counter > target");
                assert!(!cpu.get_zero(), "Zero should be clear when not equal");
            }
        }
    }

    #[test]
    fn test_comparison_signed_interpretation() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Test comparison with values that represent negative numbers in signed interpretation
        // 0x80 = -128 in signed, 128 in unsigned
        cpu.a = 0x80;
        let addr_result = AddressingResult::immediate(0x7F); // 127 in both signed and unsigned
        cpu.cmp(&bus, &addr_result);

        // In unsigned comparison: 0x80 (128) > 0x7F (127)
        assert!(cpu.get_carry(), "Carry should be set (unsigned: 128 > 127)");
        assert!(!cpu.get_zero(), "Zero should be clear");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear (0x80 - 0x7F = 0x01)"
        );
    }

    #[test]
    fn test_comparison_wrapping_behavior() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Test subtraction that wraps around
        // 0x00 - 0x01 = 0xFF (wraps around)
        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(0x01);
        cpu.cmp(&bus, &addr_result);

        assert!(
            !cpu.get_carry(),
            "Carry should be clear (0 < 1, borrow needed)"
        );
        assert!(!cpu.get_zero(), "Zero should be clear");
        assert!(
            cpu.get_negative(),
            "Negative should be set (result 0xFF has bit 7 set)"
        );
    }
}
