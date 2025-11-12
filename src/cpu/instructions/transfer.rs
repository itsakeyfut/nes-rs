// Register Transfer instructions for 6502 CPU

use crate::cpu::Cpu;

impl Cpu {
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
    use super::*;

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
