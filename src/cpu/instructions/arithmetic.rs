// Arithmetic instructions for 6502 CPU

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Arithmetic Instructions
    // ========================================

    /// ADC - Add with Carry
    ///
    /// Adds the value from memory to the accumulator, plus the carry flag.
    /// This is used for multi-byte addition and arithmetic operations.
    ///
    /// Formula: A = A + M + C
    ///
    /// The Overflow (V) flag is set when the sign of the result is incorrect:
    /// - Adding two positive numbers produces a negative result
    /// - Adding two negative numbers produces a positive result
    ///
    /// Flags affected: C, Z, V, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn adc(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);
        let carry = if self.get_carry() { 1 } else { 0 };

        // Perform addition with carry using u16 to detect overflow
        let sum = self.a as u16 + value as u16 + carry as u16;

        // Set carry flag if result exceeds 255
        self.set_carry(sum > 0xFF);

        // Convert result to u8
        let result = sum as u8;

        // Set overflow flag
        // Overflow occurs when:
        // - Both operands have the same sign (bit 7)
        // - The result has a different sign than the operands
        // This can be checked with: (A^result) & (M^result) & 0x80
        let overflow = (self.a ^ result) & (value ^ result) & 0x80 != 0;
        self.set_overflow(overflow);

        // Update accumulator
        self.a = result;

        // Update zero and negative flags
        self.update_zero_and_negative_flags(result);
    }

    /// SBC - Subtract with Carry
    ///
    /// Subtracts the value from memory from the accumulator, minus (1 - carry).
    /// This is used for multi-byte subtraction and comparison operations.
    ///
    /// Formula: A = A - M - (1 - C)
    /// Equivalent to: A = A + ~M + C (using two's complement)
    ///
    /// The Overflow (V) flag is set when the sign of the result is incorrect.
    ///
    /// Flags affected: C, Z, V, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from
    /// * `addr_result` - The addressing result containing the memory address or immediate value
    pub fn sbc(&mut self, bus: &Bus, addr_result: &AddressingResult) {
        let value = self.read_operand(bus, addr_result);

        // SBC is equivalent to ADC with the one's complement of the value
        // A - M - (1-C) = A + ~M + C
        let inverted = !value;
        let carry = if self.get_carry() { 1 } else { 0 };

        // Perform subtraction using addition with inverted value
        let sum = self.a as u16 + inverted as u16 + carry as u16;

        // Set carry flag if no borrow occurred (result >= 0)
        self.set_carry(sum > 0xFF);

        // Convert result to u8
        let result = sum as u8;

        // Set overflow flag
        // Overflow occurs when:
        // - The operands have different signs
        // - The result has a different sign than the accumulator
        let overflow = (self.a ^ result) & (inverted ^ result) & 0x80 != 0;
        self.set_overflow(overflow);

        // Update accumulator
        self.a = result;

        // Update zero and negative flags
        self.update_zero_and_negative_flags(result);
    }

    /// INC - Increment Memory
    ///
    /// Increments the value at the specified memory location by 1.
    /// Wraps around from 0xFF to 0x00.
    ///
    /// Flags affected: Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from and write to
    /// * `addr_result` - The addressing result containing the memory address
    pub fn inc(&self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = bus.read(addr_result.address);
        let result = value.wrapping_add(1);
        bus.write(addr_result.address, result);
    }

    /// Helper method to update flags for INC instruction
    /// This is needed because INC modifies memory and needs to update CPU flags
    #[inline]
    pub fn inc_update_flags(&mut self, value: u8) {
        self.update_zero_and_negative_flags(value);
    }

    /// DEC - Decrement Memory
    ///
    /// Decrements the value at the specified memory location by 1.
    /// Wraps around from 0x00 to 0xFF.
    ///
    /// Flags affected: Z, N
    ///
    /// # Arguments
    /// * `bus` - The memory bus to read from and write to
    /// * `addr_result` - The addressing result containing the memory address
    pub fn dec(&self, bus: &mut Bus, addr_result: &AddressingResult) {
        let value = bus.read(addr_result.address);
        let result = value.wrapping_sub(1);
        bus.write(addr_result.address, result);
    }

    /// Helper method to update flags for DEC instruction
    /// This is needed because DEC modifies memory and needs to update CPU flags
    #[inline]
    pub fn dec_update_flags(&mut self, value: u8) {
        self.update_zero_and_negative_flags(value);
    }

    // ========================================
    // Register Increment/Decrement Instructions
    // ========================================

    /// INX - Increment X Register
    ///
    /// Increments the X register by 1.
    /// Wraps around from 0xFF to 0x00.
    ///
    /// Flags affected: Z, N
    pub fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.x);
    }

    /// INY - Increment Y Register
    ///
    /// Increments the Y register by 1.
    /// Wraps around from 0xFF to 0x00.
    ///
    /// Flags affected: Z, N
    pub fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.y);
    }

    /// DEX - Decrement X Register
    ///
    /// Decrements the X register by 1.
    /// Wraps around from 0x00 to 0xFF.
    ///
    /// Flags affected: Z, N
    pub fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.x);
    }

    /// DEY - Decrement Y Register
    ///
    /// Decrements the Y register by 1.
    /// Wraps around from 0x00 to 0xFF.
    ///
    /// Flags affected: Z, N
    pub fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::addressing::AddressingResult;

    // ========================================
    // Arithmetic Instruction Tests - ADC
    // ========================================

    #[test]
    fn test_adc_simple() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x10;
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0x20);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x30, "0x10 + 0x20 should equal 0x30");
        assert!(!cpu.get_carry(), "Carry flag should be clear");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
        assert!(!cpu.get_overflow(), "Overflow flag should be clear");
    }

    #[test]
    fn test_adc_with_carry() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x10;
        cpu.set_carry(true);

        let addr_result = AddressingResult::immediate(0x20);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x31, "0x10 + 0x20 + 1 should equal 0x31");
        assert!(!cpu.get_carry(), "Carry flag should be clear");
    }

    #[test]
    fn test_adc_carry_flag() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0xFF;
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0x01);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "0xFF + 0x01 should wrap to 0x00");
        assert!(cpu.get_carry(), "Carry flag should be set");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_adc_overflow_positive() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Adding two positive numbers that overflow into negative
        cpu.a = 0x50; // +80
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0x50); // +80
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0xA0, "Result should be 0xA0 (160, or -96 in signed)");
        assert!(!cpu.get_carry(), "Carry flag should be clear");
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (positive + positive = negative)"
        );
        assert!(
            cpu.get_negative(),
            "Negative flag should be set (bit 7 is 1)"
        );
    }

    #[test]
    fn test_adc_overflow_negative() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Adding two negative numbers that overflow into positive
        cpu.a = 0x80; // -128
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0xFF); // -1
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x7F, "Result should be 0x7F");
        assert!(cpu.get_carry(), "Carry flag should be set");
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (negative + negative = positive)"
        );
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_adc_no_overflow() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Adding positive and negative numbers (different signs) never overflow
        cpu.a = 0x50; // +80
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0xF0); // -16
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x40, "0x50 + 0xF0 should equal 0x40");
        assert!(cpu.get_carry(), "Carry flag should be set");
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (different signs)"
        );
    }

    #[test]
    fn test_adc_zero_result() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x00;
        cpu.set_carry(false);

        let addr_result = AddressingResult::immediate(0x00);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
        assert!(!cpu.get_carry(), "Carry flag should be clear");
    }

    // ========================================
    // Arithmetic Instruction Tests - SBC
    // ========================================

    #[test]
    fn test_sbc_simple() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x50;
        cpu.set_carry(true); // No borrow

        let addr_result = AddressingResult::immediate(0x20);
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x30, "0x50 - 0x20 should equal 0x30");
        assert!(cpu.get_carry(), "Carry flag should be set (no borrow)");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
        assert!(!cpu.get_overflow(), "Overflow flag should be clear");
    }

    #[test]
    fn test_sbc_with_borrow() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x50;
        cpu.set_carry(false); // Borrow

        let addr_result = AddressingResult::immediate(0x20);
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x2F, "0x50 - 0x20 - 1 should equal 0x2F");
        assert!(cpu.get_carry(), "Carry flag should be set");
    }

    #[test]
    fn test_sbc_underflow() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x00;
        cpu.set_carry(true); // No borrow

        let addr_result = AddressingResult::immediate(0x01);
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0xFF, "0x00 - 0x01 should wrap to 0xFF");
        assert!(!cpu.get_carry(), "Carry flag should be clear (borrow occurred)");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_sbc_zero_result() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        cpu.a = 0x50;
        cpu.set_carry(true);

        let addr_result = AddressingResult::immediate(0x50);
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(cpu.get_carry(), "Carry flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_sbc_overflow() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Subtracting a negative from a positive can cause overflow
        cpu.a = 0x50; // +80
        cpu.set_carry(true);

        let addr_result = AddressingResult::immediate(0xB0); // -80
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0xA0);
        assert!(
            cpu.get_overflow(),
            "Overflow flag should be set (positive - negative = negative overflow)"
        );
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_sbc_no_overflow() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Subtracting a positive from a positive (both same sign) doesn't overflow
        cpu.a = 0x50;
        cpu.set_carry(true);

        let addr_result = AddressingResult::immediate(0x30);
        cpu.sbc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x20);
        assert!(
            !cpu.get_overflow(),
            "Overflow flag should be clear (same sign operands)"
        );
    }

    // ========================================
    // Arithmetic Instruction Tests - INC
    // ========================================

    #[test]
    fn test_inc_simple() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0x42);

        let addr_result = AddressingResult::new(0x1234);
        cpu.inc(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0x43, "0x42 + 1 should equal 0x43");

        // Update flags manually for testing (in real execution, this would be done in the instruction handler)
        cpu.inc_update_flags(result);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_inc_wrapping() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0xFF);

        let addr_result = AddressingResult::new(0x1234);
        cpu.inc(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0x00, "0xFF + 1 should wrap to 0x00");

        cpu.inc_update_flags(result);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_inc_negative() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0x7F);

        let addr_result = AddressingResult::new(0x1234);
        cpu.inc(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0x80, "0x7F + 1 should equal 0x80");

        cpu.inc_update_flags(result);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set (bit 7 is 1)");
    }

    // ========================================
    // Arithmetic Instruction Tests - DEC
    // ========================================

    #[test]
    fn test_dec_simple() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0x42);

        let addr_result = AddressingResult::new(0x1234);
        cpu.dec(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0x41, "0x42 - 1 should equal 0x41");

        cpu.dec_update_flags(result);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_dec_wrapping() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0x00);

        let addr_result = AddressingResult::new(0x1234);
        cpu.dec(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0xFF, "0x00 - 1 should wrap to 0xFF");

        cpu.dec_update_flags(result);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_dec_zero() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        bus.write(0x1234, 0x01);

        let addr_result = AddressingResult::new(0x1234);
        cpu.dec(&mut bus, &addr_result);

        let result = bus.read(0x1234);
        assert_eq!(result, 0x00, "0x01 - 1 should equal 0x00");

        cpu.dec_update_flags(result);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // Arithmetic Instruction Tests - INX
    // ========================================

    #[test]
    fn test_inx_simple() {
        let mut cpu = Cpu::new();

        cpu.x = 0x42;
        cpu.inx();

        assert_eq!(cpu.x, 0x43, "X should be incremented from 0x42 to 0x43");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_inx_wrapping() {
        let mut cpu = Cpu::new();

        cpu.x = 0xFF;
        cpu.inx();

        assert_eq!(cpu.x, 0x00, "X should wrap from 0xFF to 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_inx_negative() {
        let mut cpu = Cpu::new();

        cpu.x = 0x7F;
        cpu.inx();

        assert_eq!(cpu.x, 0x80);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // Arithmetic Instruction Tests - INY
    // ========================================

    #[test]
    fn test_iny_simple() {
        let mut cpu = Cpu::new();

        cpu.y = 0x42;
        cpu.iny();

        assert_eq!(cpu.y, 0x43, "Y should be incremented from 0x42 to 0x43");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_iny_wrapping() {
        let mut cpu = Cpu::new();

        cpu.y = 0xFF;
        cpu.iny();

        assert_eq!(cpu.y, 0x00, "Y should wrap from 0xFF to 0x00");
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_iny_negative() {
        let mut cpu = Cpu::new();

        cpu.y = 0x7F;
        cpu.iny();

        assert_eq!(cpu.y, 0x80);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    // ========================================
    // Arithmetic Instruction Tests - DEX
    // ========================================

    #[test]
    fn test_dex_simple() {
        let mut cpu = Cpu::new();

        cpu.x = 0x42;
        cpu.dex();

        assert_eq!(cpu.x, 0x41, "X should be decremented from 0x42 to 0x41");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_dex_wrapping() {
        let mut cpu = Cpu::new();

        cpu.x = 0x00;
        cpu.dex();

        assert_eq!(cpu.x, 0xFF, "X should wrap from 0x00 to 0xFF");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_dex_zero() {
        let mut cpu = Cpu::new();

        cpu.x = 0x01;
        cpu.dex();

        assert_eq!(cpu.x, 0x00);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // Arithmetic Instruction Tests - DEY
    // ========================================

    #[test]
    fn test_dey_simple() {
        let mut cpu = Cpu::new();

        cpu.y = 0x42;
        cpu.dey();

        assert_eq!(cpu.y, 0x41, "Y should be decremented from 0x42 to 0x41");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_dey_wrapping() {
        let mut cpu = Cpu::new();

        cpu.y = 0x00;
        cpu.dey();

        assert_eq!(cpu.y, 0xFF, "Y should wrap from 0x00 to 0xFF");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(cpu.get_negative(), "Negative flag should be set");
    }

    #[test]
    fn test_dey_zero() {
        let mut cpu = Cpu::new();

        cpu.y = 0x01;
        cpu.dey();

        assert_eq!(cpu.y, 0x00);
        assert!(cpu.get_zero(), "Zero flag should be set");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    // ========================================
    // Integration Tests for Arithmetic
    // ========================================

    #[test]
    fn test_multi_byte_addition() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Simulate adding two 16-bit numbers: 0x1234 + 0x5678 = 0x68AC
        // Low byte: 0x34 + 0x78 = 0xAC (carry = 0)
        cpu.a = 0x34;
        cpu.set_carry(false);
        let addr_result = AddressingResult::immediate(0x78);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0xAC, "Low byte should be 0xAC");
        assert!(!cpu.get_carry(), "Carry should be clear");

        // High byte: 0x12 + 0x56 + 0 (no carry) = 0x68
        cpu.a = 0x12;
        let addr_result = AddressingResult::immediate(0x56);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x68, "High byte should be 0x68");
        assert!(!cpu.get_carry(), "Carry should be clear");
    }

    #[test]
    fn test_multi_byte_addition_with_carry() {
        let mut cpu = Cpu::new();
        let bus = Bus::new();

        // Simulate adding two 16-bit numbers: 0x12FF + 0x5601 = 0x6900
        // Low byte: 0xFF + 0x01 = 0x00 (carry = 1)
        cpu.a = 0xFF;
        cpu.set_carry(false);
        let addr_result = AddressingResult::immediate(0x01);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x00, "Low byte should be 0x00");
        assert!(cpu.get_carry(), "Carry should be set");

        // High byte: 0x12 + 0x56 + 1 (carry) = 0x69
        cpu.a = 0x12;
        let addr_result = AddressingResult::immediate(0x56);
        cpu.adc(&bus, &addr_result);

        assert_eq!(cpu.a, 0x69, "High byte should be 0x69");
        assert!(!cpu.get_carry(), "Carry should be clear");
    }

    #[test]
    fn test_counter_pattern_with_inx_dex() {
        let mut cpu = Cpu::new();

        // Initialize counter
        cpu.x = 0x00;

        // Increment 10 times
        for _ in 0..10 {
            cpu.inx();
        }
        assert_eq!(cpu.x, 0x0A);

        // Decrement 5 times
        for _ in 0..5 {
            cpu.dex();
        }
        assert_eq!(cpu.x, 0x05);

        // Decrement to zero
        for _ in 0..5 {
            cpu.dex();
        }
        assert_eq!(cpu.x, 0x00);
        assert!(cpu.get_zero(), "Zero flag should be set");
    }
}
