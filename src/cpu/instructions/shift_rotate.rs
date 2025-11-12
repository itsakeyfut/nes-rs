// Shift and rotate instructions for 6502 CPU

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Shift Instructions
    // ========================================

    /// ASL - Arithmetic Shift Left
    ///
    /// Shifts all bits left one position. Bit 0 is filled with 0.
    /// Bit 7 is shifted into the Carry flag.
    ///
    /// Formula: C <- [76543210] <- 0
    ///
    /// This instruction is commonly used for:
    /// - Multiplying by 2 (in binary)
    /// - Extracting the high bit
    /// - Building bit patterns
    ///
    /// Flags affected: C, Z, N
    /// - C: Set to the value of bit 7 before the shift
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading/writing
    /// * `addr_result` - The addressing result containing the address or accumulator value
    /// * `is_accumulator` - True if operating on accumulator, false if operating on memory
    ///
    /// # Note
    /// This instruction can operate on either the accumulator or a memory location.
    /// When operating on the accumulator, is_accumulator should be true.
    pub fn asl(&mut self, bus: &mut Bus, addr_result: &AddressingResult, is_accumulator: bool) {
        let value = if is_accumulator {
            self.a
        } else {
            bus.read(addr_result.address)
        };

        // Bit 7 goes to Carry flag
        self.set_carry((value & 0x80) != 0);

        // Shift left by 1, bit 0 becomes 0
        let result = value << 1;

        // Update flags based on result
        self.update_zero_and_negative_flags(result);

        // Write back the result
        if is_accumulator {
            self.a = result;
        } else {
            bus.write(addr_result.address, result);
        }
    }

    /// LSR - Logical Shift Right
    ///
    /// Shifts all bits right one position. Bit 7 is filled with 0.
    /// Bit 0 is shifted into the Carry flag.
    ///
    /// Formula: 0 -> [76543210] -> C
    ///
    /// This instruction is commonly used for:
    /// - Dividing by 2 (unsigned)
    /// - Extracting the low bit
    /// - Processing bit patterns
    ///
    /// Flags affected: C, Z, N
    /// - C: Set to the value of bit 0 before the shift
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set (always 0 for LSR)
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading/writing
    /// * `addr_result` - The addressing result containing the address or accumulator value
    /// * `is_accumulator` - True if operating on accumulator, false if operating on memory
    ///
    /// # Note
    /// This instruction can operate on either the accumulator or a memory location.
    /// After LSR, the Negative flag is always clear since bit 7 is always 0.
    pub fn lsr(&mut self, bus: &mut Bus, addr_result: &AddressingResult, is_accumulator: bool) {
        let value = if is_accumulator {
            self.a
        } else {
            bus.read(addr_result.address)
        };

        // Bit 0 goes to Carry flag
        self.set_carry((value & 0x01) != 0);

        // Shift right by 1, bit 7 becomes 0
        let result = value >> 1;

        // Update flags based on result
        self.update_zero_and_negative_flags(result);

        // Write back the result
        if is_accumulator {
            self.a = result;
        } else {
            bus.write(addr_result.address, result);
        }
    }

    // ========================================
    // Rotate Instructions
    // ========================================

    /// ROL - Rotate Left
    ///
    /// Rotates all bits left one position through the Carry flag.
    /// Bit 0 is filled with the current Carry flag value.
    /// Bit 7 is shifted into the Carry flag.
    ///
    /// Formula: C <- [76543210] <- C
    ///
    /// This instruction is commonly used for:
    /// - Rotating multi-byte values
    /// - Implementing circular shifts
    /// - Multiplying by 2 while preserving the overflow
    ///
    /// Flags affected: C, Z, N
    /// - C: Set to the value of bit 7 before the rotate
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading/writing
    /// * `addr_result` - The addressing result containing the address or accumulator value
    /// * `is_accumulator` - True if operating on accumulator, false if operating on memory
    ///
    /// # Note
    /// ROL differs from ASL in that it rotates the old Carry flag into bit 0,
    /// while ASL always sets bit 0 to 0.
    pub fn rol(&mut self, bus: &mut Bus, addr_result: &AddressingResult, is_accumulator: bool) {
        let value = if is_accumulator {
            self.a
        } else {
            bus.read(addr_result.address)
        };

        // Save the current Carry flag
        let old_carry = if self.get_carry() { 1 } else { 0 };

        // Bit 7 goes to Carry flag
        self.set_carry((value & 0x80) != 0);

        // Shift left by 1, and set bit 0 to old Carry
        let result = (value << 1) | old_carry;

        // Update flags based on result
        self.update_zero_and_negative_flags(result);

        // Write back the result
        if is_accumulator {
            self.a = result;
        } else {
            bus.write(addr_result.address, result);
        }
    }

    /// ROR - Rotate Right
    ///
    /// Rotates all bits right one position through the Carry flag.
    /// Bit 7 is filled with the current Carry flag value.
    /// Bit 0 is shifted into the Carry flag.
    ///
    /// Formula: C -> [76543210] -> C
    ///
    /// This instruction is commonly used for:
    /// - Rotating multi-byte values
    /// - Implementing circular shifts
    /// - Dividing by 2 while preserving the remainder
    ///
    /// Flags affected: C, Z, N
    /// - C: Set to the value of bit 0 before the rotate
    /// - Z: Set if the result is zero
    /// - N: Set if bit 7 of the result is set (equal to old Carry flag)
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading/writing
    /// * `addr_result` - The addressing result containing the address or accumulator value
    /// * `is_accumulator` - True if operating on accumulator, false if operating on memory
    ///
    /// # Note
    /// ROR differs from LSR in that it rotates the old Carry flag into bit 7,
    /// while LSR always sets bit 7 to 0.
    pub fn ror(&mut self, bus: &mut Bus, addr_result: &AddressingResult, is_accumulator: bool) {
        let value = if is_accumulator {
            self.a
        } else {
            bus.read(addr_result.address)
        };

        // Save the current Carry flag
        let old_carry = if self.get_carry() { 0x80 } else { 0 };

        // Bit 0 goes to Carry flag
        self.set_carry((value & 0x01) != 0);

        // Shift right by 1, and set bit 7 to old Carry
        let result = (value >> 1) | old_carry;

        // Update flags based on result
        self.update_zero_and_negative_flags(result);

        // Write back the result
        if is_accumulator {
            self.a = result;
        } else {
            bus.write(addr_result.address, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;

    // ========================================
    // ASL - Arithmetic Shift Left Tests
    // ========================================

    #[test]
    fn test_asl_accumulator_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0000_0010; // 2
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.asl(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0100, "2 << 1 should equal 4");
        assert!(!cpu.get_carry(), "Carry should be clear (bit 7 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_asl_accumulator_carry_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1000_0001; // Bit 7 is set
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.asl(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0010);
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_asl_accumulator_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1000_0000;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.asl(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_asl_accumulator_negative_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0100_0000;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.asl(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b1000_0000);
        assert!(!cpu.get_carry(), "Carry should be clear (bit 7 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set (bit 7 is 1)");
    }

    #[test]
    fn test_asl_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let addr = 0x0200;
        bus.write(addr, 0b0000_0010);

        let addr_result = AddressingResult::new(addr);
        cpu.asl(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b0000_0100, "Memory value should be shifted");
        assert!(!cpu.get_carry(), "Carry should be clear");
    }

    #[test]
    fn test_asl_memory_with_carry() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let addr = 0x0200;
        bus.write(addr, 0b1100_0000);

        let addr_result = AddressingResult::new(addr);
        cpu.asl(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b1000_0000);
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // LSR - Logical Shift Right Tests
    // ========================================

    #[test]
    fn test_lsr_accumulator_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0000_0100; // 4
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0010, "4 >> 1 should equal 2");
        assert!(!cpu.get_carry(), "Carry should be clear (bit 0 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_lsr_accumulator_carry_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0000_0101; // Bit 0 is set
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0010);
        assert!(cpu.get_carry(), "Carry should be set (bit 0 was 1)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_lsr_accumulator_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b0000_0001;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_carry(), "Carry should be set (bit 0 was 1)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_lsr_accumulator_high_bit_cleared() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0b1000_0000;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0100_0000);
        assert!(!cpu.get_carry(), "Carry should be clear (bit 0 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear (bit 7 is 0)");
    }

    #[test]
    fn test_lsr_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let addr = 0x0200;
        bus.write(addr, 0b0000_0100);

        let addr_result = AddressingResult::new(addr);
        cpu.lsr(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b0000_0010, "Memory value should be shifted");
        assert!(!cpu.get_carry(), "Carry should be clear");
    }

    #[test]
    fn test_lsr_memory_with_carry() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        let addr = 0x0200;
        bus.write(addr, 0b1100_0011);

        let addr_result = AddressingResult::new(addr);
        cpu.lsr(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b0110_0001);
        assert!(cpu.get_carry(), "Carry should be set (bit 0 was 1)");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // ROL - Rotate Left Tests
    // ========================================

    #[test]
    fn test_rol_accumulator_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b0000_0010;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0100);
        assert!(!cpu.get_carry(), "Carry should be clear (bit 7 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_rol_accumulator_with_carry_in() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        cpu.a = 0b0000_0010;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0101, "Old carry should be rotated into bit 0");
        assert!(!cpu.get_carry(), "Carry should be clear (bit 7 was 0)");
    }

    #[test]
    fn test_rol_accumulator_carry_out() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b1000_0001;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0010);
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
    }

    #[test]
    fn test_rol_accumulator_full_rotation() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        cpu.a = 0b1010_1010;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0101_0101, "Pattern should rotate with carry");
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
    }

    #[test]
    fn test_rol_accumulator_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b1000_0000;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_carry(), "Carry should be set (bit 7 was 1)");
        assert!(cpu.get_zero(), "Zero flag should be set");
    }

    #[test]
    fn test_rol_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        let addr = 0x0200;
        bus.write(addr, 0b0100_0000);

        let addr_result = AddressingResult::new(addr);
        cpu.rol(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b1000_0001, "Should rotate with carry");
        assert!(!cpu.get_carry(), "Carry should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // ROR - Rotate Right Tests
    // ========================================

    #[test]
    fn test_ror_accumulator_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b0000_0100;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.ror(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0000_0010);
        assert!(!cpu.get_carry(), "Carry should be clear (bit 0 was 0)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ror_accumulator_with_carry_in() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        cpu.a = 0b0000_0100;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.ror(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b1000_0010, "Old carry should be rotated into bit 7");
        assert!(!cpu.get_carry(), "Carry should be clear (bit 0 was 0)");
        assert!(cpu.get_negative(), "Negative flag should be set (bit 7 is 1)");
    }

    #[test]
    fn test_ror_accumulator_carry_out() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b1000_0001;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.ror(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0100_0000);
        assert!(cpu.get_carry(), "Carry should be set (bit 0 was 1)");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ror_accumulator_full_rotation() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        cpu.a = 0b1010_1010;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.ror(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b1101_0101, "Pattern should rotate with carry");
        assert!(!cpu.get_carry(), "Carry should be clear (bit 0 was 0)");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_ror_accumulator_zero_result() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(false);
        cpu.a = 0b0000_0001;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.ror(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_carry(), "Carry should be set (bit 0 was 1)");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_ror_memory() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.set_carry(true);
        let addr = 0x0200;
        bus.write(addr, 0b0000_0010);

        let addr_result = AddressingResult::new(addr);
        cpu.ror(&mut bus, &addr_result, false);

        assert_eq!(bus.read(addr), 0b1000_0001, "Should rotate with carry");
        assert!(!cpu.get_carry(), "Carry should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // Integration and Edge Case Tests
    // ========================================

    #[test]
    fn test_asl_vs_rol_difference() {
        let mut cpu1 = Cpu::new();
        let mut cpu2 = Cpu::new();
        let mut bus1 = Bus::new();
        let mut bus2 = Bus::new();

        // ASL always shifts in 0
        cpu1.a = 0b0000_0001;
        let addr_result = AddressingResult::immediate(cpu1.a);
        cpu1.asl(&mut bus1, &addr_result, true);
        assert_eq!(cpu1.a, 0b0000_0010, "ASL shifts in 0");

        // ROL shifts in the carry flag
        cpu2.set_carry(true);
        cpu2.a = 0b0000_0001;
        let addr_result = AddressingResult::immediate(cpu2.a);
        cpu2.rol(&mut bus2, &addr_result, true);
        assert_eq!(cpu2.a, 0b0000_0011, "ROL shifts in carry");
    }

    #[test]
    fn test_lsr_vs_ror_difference() {
        let mut cpu1 = Cpu::new();
        let mut cpu2 = Cpu::new();
        let mut bus1 = Bus::new();
        let mut bus2 = Bus::new();

        // LSR always shifts in 0
        cpu1.a = 0b1000_0000;
        let addr_result = AddressingResult::immediate(cpu1.a);
        cpu1.lsr(&mut bus1, &addr_result, true);
        assert_eq!(cpu1.a, 0b0100_0000, "LSR shifts in 0");

        // ROR shifts in the carry flag
        cpu2.set_carry(true);
        cpu2.a = 0b1000_0000;
        let addr_result = AddressingResult::immediate(cpu2.a);
        cpu2.ror(&mut bus2, &addr_result, true);
        assert_eq!(cpu2.a, 0b1100_0000, "ROR shifts in carry");
    }

    #[test]
    fn test_multi_byte_rotation_pattern() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Simulate rotating a 16-bit value using ROL
        // Low byte: 0b1010_0101, High byte: 0b0101_1010
        cpu.set_carry(false);
        cpu.a = 0b1010_0101; // Low byte
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b0100_1010, "Low byte rotated");
        assert!(cpu.get_carry(), "Carry from low byte");

        // Now rotate high byte with carry
        cpu.a = 0b0101_1010; // High byte
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.rol(&mut bus, &addr_result, true);

        assert_eq!(cpu.a, 0b1011_0101, "High byte rotated with carry from low");
    }

    #[test]
    fn test_shift_all_zeros() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(cpu.a);

        // ASL of zero
        cpu.asl(&mut bus, &addr_result, true);
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_zero(), "Should be zero");
        assert!(!cpu.get_carry(), "No carry from zero");

        // LSR of zero
        cpu.a = 0x00;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_zero(), "Should be zero");
        assert!(!cpu.get_carry(), "No carry from zero");
    }

    #[test]
    fn test_shift_all_ones() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // ASL of 0xFF
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.asl(&mut bus, &addr_result, true);
        assert_eq!(cpu.a, 0xFE);
        assert!(cpu.get_carry(), "Bit 7 should go to carry");
        assert!(cpu.get_negative(), "Bit 7 should be set in result");

        // LSR of 0xFF
        cpu.a = 0xFF;
        let addr_result = AddressingResult::immediate(cpu.a);
        cpu.lsr(&mut bus, &addr_result, true);
        assert_eq!(cpu.a, 0x7F);
        assert!(cpu.get_carry(), "Bit 0 should go to carry");
        assert!(!cpu.get_negative(), "Bit 7 should be clear in result");
    }
}
