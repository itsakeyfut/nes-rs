// Flag manipulation instructions for 6502 CPU
// These instructions directly set or clear specific processor status flags.

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;
use crate::cpu::flags;
use crate::cpu::Cpu;

impl Cpu {
    // ========================================
    // Carry Flag Instructions
    // ========================================

    /// CLC - Clear Carry Flag
    ///
    /// Clears the carry flag in the processor status register.
    ///
    /// Operation: C = 0
    ///
    /// Flags affected:
    /// - C: Cleared to 0
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// CLC         ; Clear carry flag
    /// ADC #$10    ; Add with carry (C=0)
    /// ```
    ///
    /// # Common Usage
    /// Used before ADC (Add with Carry) operations to ensure a clean addition
    /// without any previous carry. Also used for multi-byte arithmetic.
    pub fn clc(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.clear_flag(flags::CARRY);
        0
    }

    /// SEC - Set Carry Flag
    ///
    /// Sets the carry flag in the processor status register.
    ///
    /// Operation: C = 1
    ///
    /// Flags affected:
    /// - C: Set to 1
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// SEC         ; Set carry flag
    /// SBC #$10    ; Subtract with borrow (C=1, no borrow)
    /// ```
    ///
    /// # Common Usage
    /// Used before SBC (Subtract with Carry) operations. In the 6502, SBC works
    /// with an inverted borrow, so SEC is used to indicate "no borrow".
    pub fn sec(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.set_flag(flags::CARRY);
        0
    }

    // ========================================
    // Interrupt Disable Flag Instructions
    // ========================================

    /// CLI - Clear Interrupt Disable Flag
    ///
    /// Clears the interrupt disable flag, enabling IRQ interrupts.
    ///
    /// Operation: I = 0
    ///
    /// Flags affected:
    /// - I: Cleared to 0 (interrupts enabled)
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// CLI         ; Enable interrupts
    /// ; IRQ interrupts can now occur
    /// ```
    ///
    /// # Important Note
    /// This instruction only affects IRQ interrupts. NMI (Non-Maskable Interrupt)
    /// cannot be disabled and will always trigger regardless of the I flag state.
    pub fn cli(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.clear_flag(flags::INTERRUPT_DISABLE);
        0
    }

    /// SEI - Set Interrupt Disable Flag
    ///
    /// Sets the interrupt disable flag, disabling IRQ interrupts.
    ///
    /// Operation: I = 1
    ///
    /// Flags affected:
    /// - I: Set to 1 (interrupts disabled)
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// SEI         ; Disable interrupts
    /// ; Critical section - IRQ interrupts cannot occur
    /// CLI         ; Re-enable interrupts
    /// ```
    ///
    /// # Important Note
    /// This instruction only affects IRQ interrupts. NMI (Non-Maskable Interrupt)
    /// cannot be disabled and will always trigger regardless of the I flag state.
    /// The I flag is automatically set when an interrupt occurs.
    pub fn sei(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.set_flag(flags::INTERRUPT_DISABLE);
        0
    }

    // ========================================
    // Decimal Mode Flag Instructions
    // ========================================

    /// CLD - Clear Decimal Mode Flag
    ///
    /// Clears the decimal mode flag, setting the CPU to binary mode.
    ///
    /// Operation: D = 0
    ///
    /// Flags affected:
    /// - D: Cleared to 0 (binary mode)
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// CLD         ; Clear decimal mode (binary mode)
    /// ADC #$10    ; Add in binary mode
    /// ```
    ///
    /// # NES-Specific Note
    /// The decimal mode is not functional in the NES/Famicom version of the 6502
    /// (the 2A03/2A07 CPU). The flag can be set and cleared, but ADC and SBC
    /// always operate in binary mode. This instruction is included for compatibility
    /// with the standard 6502 instruction set.
    pub fn cld(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.clear_flag(flags::DECIMAL);
        0
    }

    /// SED - Set Decimal Mode Flag
    ///
    /// Sets the decimal mode flag, instructing the CPU to use BCD arithmetic.
    ///
    /// Operation: D = 1
    ///
    /// Flags affected:
    /// - D: Set to 1 (decimal/BCD mode)
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// SED         ; Set decimal mode (BCD arithmetic)
    /// ADC #$09    ; Add in BCD mode
    /// CLD         ; Clear decimal mode
    /// ```
    ///
    /// # NES-Specific Note
    /// The decimal mode is not functional in the NES/Famicom version of the 6502
    /// (the 2A03/2A07 CPU). The flag can be set and cleared, but ADC and SBC
    /// always operate in binary mode. This instruction is included for compatibility
    /// with the standard 6502 instruction set, but has no effect on calculations
    /// in the NES.
    pub fn sed(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.set_flag(flags::DECIMAL);
        0
    }

    // ========================================
    // Overflow Flag Instructions
    // ========================================

    /// CLV - Clear Overflow Flag
    ///
    /// Clears the overflow flag in the processor status register.
    ///
    /// Operation: V = 0
    ///
    /// Flags affected:
    /// - V: Cleared to 0
    ///
    /// Cycles: 2 cycles
    ///
    /// # Arguments
    /// * `bus` - The memory bus (unused, but included for consistency)
    /// * `addr_result` - Unused (implied addressing mode)
    ///
    /// # Returns
    /// Always returns 0 (no additional cycles)
    ///
    /// # Example
    /// ```text
    /// CLV         ; Clear overflow flag
    /// ADC #$50    ; Add, checking for overflow
    /// BVC no_overflow  ; Branch if no overflow occurred
    /// ```
    ///
    /// # Note
    /// This is the only instruction that can clear the overflow flag. There is
    /// no corresponding "SEV" instruction to set the overflow flag. The overflow
    /// flag is typically set automatically by ADC and SBC operations when a
    /// signed overflow occurs, or by the BIT instruction based on bit 6 of the
    /// tested value.
    pub fn clv(&mut self, _bus: &mut Bus, _addr_result: &AddressingResult) -> u8 {
        self.clear_flag(flags::OVERFLOW);
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
    // CLC (Clear Carry) Tests
    // ========================================

    #[test]
    fn test_clc_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set carry flag
        cpu.set_carry(true);
        assert!(cpu.get_carry(), "Carry should be set initially");

        // Execute CLC
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.clc(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "CLC should not return additional cycles");
        assert!(!cpu.get_carry(), "Carry flag should be cleared");
    }

    #[test]
    fn test_clc_already_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Carry is already clear
        cpu.set_carry(false);
        assert!(!cpu.get_carry());

        // Execute CLC
        cpu.clc(&mut bus, &AddressingResult::new(0));

        assert!(!cpu.get_carry(), "Carry should remain clear");
    }

    #[test]
    fn test_clc_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(true);
        cpu.set_overflow(true);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute CLC
        cpu.clc(&mut bus, &AddressingResult::new(0));

        // Only carry flag should change
        assert_eq!(
            cpu.status,
            initial_status & !flags::CARRY,
            "Only carry flag should be modified"
        );
        assert!(cpu.get_zero(), "Zero flag should be unchanged");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt flag should be unchanged"
        );
        assert!(cpu.get_decimal(), "Decimal flag should be unchanged");
        assert!(cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(cpu.get_negative(), "Negative flag should be unchanged");
    }

    // ========================================
    // SEC (Set Carry) Tests
    // ========================================

    #[test]
    fn test_sec_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear carry flag
        cpu.set_carry(false);
        assert!(!cpu.get_carry(), "Carry should be clear initially");

        // Execute SEC
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.sec(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "SEC should not return additional cycles");
        assert!(cpu.get_carry(), "Carry flag should be set");
    }

    #[test]
    fn test_sec_already_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Carry is already set
        cpu.set_carry(true);
        assert!(cpu.get_carry());

        // Execute SEC
        cpu.sec(&mut bus, &AddressingResult::new(0));

        assert!(cpu.get_carry(), "Carry should remain set");
    }

    #[test]
    fn test_sec_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(true);
        cpu.set_overflow(false);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute SEC
        cpu.sec(&mut bus, &AddressingResult::new(0));

        // Only carry flag should change
        assert_eq!(
            cpu.status,
            initial_status | flags::CARRY,
            "Only carry flag should be modified"
        );
        assert!(cpu.get_zero(), "Zero flag should be unchanged");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt flag should be unchanged"
        );
        assert!(cpu.get_decimal(), "Decimal flag should be unchanged");
        assert!(!cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(cpu.get_negative(), "Negative flag should be unchanged");
    }

    #[test]
    fn test_clc_sec_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set carry
        cpu.sec(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_carry());

        // Clear carry
        cpu.clc(&mut bus, &AddressingResult::new(0));
        assert!(!cpu.get_carry());

        // Set carry again
        cpu.sec(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_carry());
    }

    // ========================================
    // CLI (Clear Interrupt Disable) Tests
    // ========================================

    #[test]
    fn test_cli_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set interrupt disable flag
        cpu.set_interrupt_disable(true);
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be set initially"
        );

        // Execute CLI
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.cli(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "CLI should not return additional cycles");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable flag should be cleared"
        );
    }

    #[test]
    fn test_cli_already_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Interrupt disable is already clear
        cpu.set_interrupt_disable(false);
        assert!(!cpu.get_interrupt_disable());

        // Execute CLI
        cpu.cli(&mut bus, &AddressingResult::new(0));

        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should remain clear"
        );
    }

    #[test]
    fn test_cli_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(true);
        cpu.set_overflow(false);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute CLI
        cpu.cli(&mut bus, &AddressingResult::new(0));

        // Only interrupt disable flag should change
        assert_eq!(
            cpu.status,
            initial_status & !flags::INTERRUPT_DISABLE,
            "Only interrupt disable flag should be modified"
        );
        assert!(cpu.get_carry(), "Carry flag should be unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be unchanged");
        assert!(cpu.get_decimal(), "Decimal flag should be unchanged");
        assert!(!cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(cpu.get_negative(), "Negative flag should be unchanged");
    }

    // ========================================
    // SEI (Set Interrupt Disable) Tests
    // ========================================

    #[test]
    fn test_sei_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear interrupt disable flag
        cpu.set_interrupt_disable(false);
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should be clear initially"
        );

        // Execute SEI
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.sei(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "SEI should not return additional cycles");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable flag should be set"
        );
    }

    #[test]
    fn test_sei_already_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Interrupt disable is already set
        cpu.set_interrupt_disable(true);
        assert!(cpu.get_interrupt_disable());

        // Execute SEI
        cpu.sei(&mut bus, &AddressingResult::new(0));

        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should remain set"
        );
    }

    #[test]
    fn test_sei_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute SEI
        cpu.sei(&mut bus, &AddressingResult::new(0));

        // Only interrupt disable flag should change
        assert_eq!(
            cpu.status,
            initial_status | flags::INTERRUPT_DISABLE,
            "Only interrupt disable flag should be modified"
        );
        assert!(!cpu.get_carry(), "Carry flag should be unchanged");
        assert!(cpu.get_zero(), "Zero flag should be unchanged");
        assert!(!cpu.get_decimal(), "Decimal flag should be unchanged");
        assert!(cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(!cpu.get_negative(), "Negative flag should be unchanged");
    }

    #[test]
    fn test_cli_sei_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set interrupt disable
        cpu.sei(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_interrupt_disable());

        // Clear interrupt disable
        cpu.cli(&mut bus, &AddressingResult::new(0));
        assert!(!cpu.get_interrupt_disable());

        // Set interrupt disable again
        cpu.sei(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_interrupt_disable());
    }

    // ========================================
    // CLD (Clear Decimal) Tests
    // ========================================

    #[test]
    fn test_cld_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set decimal flag
        cpu.set_decimal(true);
        assert!(cpu.get_decimal(), "Decimal should be set initially");

        // Execute CLD
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.cld(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "CLD should not return additional cycles");
        assert!(!cpu.get_decimal(), "Decimal flag should be cleared");
    }

    #[test]
    fn test_cld_already_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Decimal is already clear
        cpu.set_decimal(false);
        assert!(!cpu.get_decimal());

        // Execute CLD
        cpu.cld(&mut bus, &AddressingResult::new(0));

        assert!(!cpu.get_decimal(), "Decimal should remain clear");
    }

    #[test]
    fn test_cld_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(true);
        cpu.set_overflow(false);
        cpu.set_negative(true);

        let initial_status = cpu.status;

        // Execute CLD
        cpu.cld(&mut bus, &AddressingResult::new(0));

        // Only decimal flag should change
        assert_eq!(
            cpu.status,
            initial_status & !flags::DECIMAL,
            "Only decimal flag should be modified"
        );
        assert!(cpu.get_carry(), "Carry flag should be unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be unchanged");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt flag should be unchanged"
        );
        assert!(!cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(cpu.get_negative(), "Negative flag should be unchanged");
    }

    // ========================================
    // SED (Set Decimal) Tests
    // ========================================

    #[test]
    fn test_sed_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear decimal flag
        cpu.set_decimal(false);
        assert!(!cpu.get_decimal(), "Decimal should be clear initially");

        // Execute SED
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.sed(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "SED should not return additional cycles");
        assert!(cpu.get_decimal(), "Decimal flag should be set");
    }

    #[test]
    fn test_sed_already_set() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Decimal is already set
        cpu.set_decimal(true);
        assert!(cpu.get_decimal());

        // Execute SED
        cpu.sed(&mut bus, &AddressingResult::new(0));

        assert!(cpu.get_decimal(), "Decimal should remain set");
    }

    #[test]
    fn test_sed_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute SED
        cpu.sed(&mut bus, &AddressingResult::new(0));

        // Only decimal flag should change
        assert_eq!(
            cpu.status,
            initial_status | flags::DECIMAL,
            "Only decimal flag should be modified"
        );
        assert!(!cpu.get_carry(), "Carry flag should be unchanged");
        assert!(cpu.get_zero(), "Zero flag should be unchanged");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt flag should be unchanged"
        );
        assert!(cpu.get_overflow(), "Overflow flag should be unchanged");
        assert!(!cpu.get_negative(), "Negative flag should be unchanged");
    }

    #[test]
    fn test_cld_sed_pair() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set decimal
        cpu.sed(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_decimal());

        // Clear decimal
        cpu.cld(&mut bus, &AddressingResult::new(0));
        assert!(!cpu.get_decimal());

        // Set decimal again
        cpu.sed(&mut bus, &AddressingResult::new(0));
        assert!(cpu.get_decimal());
    }

    // ========================================
    // CLV (Clear Overflow) Tests
    // ========================================

    #[test]
    fn test_clv_basic() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set overflow flag
        cpu.set_overflow(true);
        assert!(cpu.get_overflow(), "Overflow should be set initially");

        // Execute CLV
        let addr_result = AddressingResult::new(0);
        let cycles = cpu.clv(&mut bus, &addr_result);

        assert_eq!(cycles, 0, "CLV should not return additional cycles");
        assert!(!cpu.get_overflow(), "Overflow flag should be cleared");
    }

    #[test]
    fn test_clv_already_clear() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Overflow is already clear
        cpu.set_overflow(false);
        assert!(!cpu.get_overflow());

        // Execute CLV
        cpu.clv(&mut bus, &AddressingResult::new(0));

        assert!(!cpu.get_overflow(), "Overflow should remain clear");
    }

    #[test]
    fn test_clv_no_other_flags_modified() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags to known state
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(false);
        cpu.set_overflow(true);
        cpu.set_negative(false);

        let initial_status = cpu.status;

        // Execute CLV
        cpu.clv(&mut bus, &AddressingResult::new(0));

        // Only overflow flag should change
        assert_eq!(
            cpu.status,
            initial_status & !flags::OVERFLOW,
            "Only overflow flag should be modified"
        );
        assert!(cpu.get_carry(), "Carry flag should be unchanged");
        assert!(!cpu.get_zero(), "Zero flag should be unchanged");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt flag should be unchanged"
        );
        assert!(!cpu.get_decimal(), "Decimal flag should be unchanged");
        assert!(!cpu.get_negative(), "Negative flag should be unchanged");
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_multiple_flag_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear all flags
        cpu.clc(&mut bus, &AddressingResult::new(0));
        cpu.cli(&mut bus, &AddressingResult::new(0));
        cpu.cld(&mut bus, &AddressingResult::new(0));
        cpu.clv(&mut bus, &AddressingResult::new(0));

        assert!(!cpu.get_carry(), "Carry should be clear");
        assert!(!cpu.get_interrupt_disable(), "Interrupt should be enabled");
        assert!(!cpu.get_decimal(), "Decimal should be clear");
        assert!(!cpu.get_overflow(), "Overflow should be clear");

        // Set all flags
        cpu.sec(&mut bus, &AddressingResult::new(0));
        cpu.sei(&mut bus, &AddressingResult::new(0));
        cpu.sed(&mut bus, &AddressingResult::new(0));

        assert!(cpu.get_carry(), "Carry should be set");
        assert!(cpu.get_interrupt_disable(), "Interrupt should be disabled");
        assert!(cpu.get_decimal(), "Decimal should be set");
        // Overflow remains clear (no SEV instruction)
        assert!(!cpu.get_overflow(), "Overflow should still be clear");
    }

    #[test]
    fn test_flag_instructions_independent() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set some flags
        cpu.sec(&mut bus, &AddressingResult::new(0));
        cpu.sei(&mut bus, &AddressingResult::new(0));

        // Clear one flag
        cpu.clc(&mut bus, &AddressingResult::new(0));

        // Only the cleared flag should be affected
        assert!(!cpu.get_carry(), "Carry should be clear");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should remain set"
        );
    }

    #[test]
    fn test_all_clear_instructions() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set all flags that can be cleared
        cpu.set_carry(true);
        cpu.set_interrupt_disable(true);
        cpu.set_decimal(true);
        cpu.set_overflow(true);

        // Execute all clear instructions
        cpu.clc(&mut bus, &AddressingResult::new(0));
        cpu.cli(&mut bus, &AddressingResult::new(0));
        cpu.cld(&mut bus, &AddressingResult::new(0));
        cpu.clv(&mut bus, &AddressingResult::new(0));

        // All should be cleared
        assert!(!cpu.get_carry(), "Carry should be cleared");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should be cleared"
        );
        assert!(!cpu.get_decimal(), "Decimal should be cleared");
        assert!(!cpu.get_overflow(), "Overflow should be cleared");
    }

    #[test]
    fn test_all_set_instructions() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Clear all flags that can be set
        cpu.set_carry(false);
        cpu.set_interrupt_disable(false);
        cpu.set_decimal(false);

        // Execute all set instructions (no SEV for overflow)
        cpu.sec(&mut bus, &AddressingResult::new(0));
        cpu.sei(&mut bus, &AddressingResult::new(0));
        cpu.sed(&mut bus, &AddressingResult::new(0));

        // All should be set
        assert!(cpu.get_carry(), "Carry should be set");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be set"
        );
        assert!(cpu.get_decimal(), "Decimal should be set");
    }

    #[test]
    fn test_unused_flag_unaffected() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Unused flag should always be 1
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED flag must be 1");

        // Execute all flag manipulation instructions
        cpu.clc(&mut bus, &AddressingResult::new(0));
        cpu.sec(&mut bus, &AddressingResult::new(0));
        cpu.cli(&mut bus, &AddressingResult::new(0));
        cpu.sei(&mut bus, &AddressingResult::new(0));
        cpu.cld(&mut bus, &AddressingResult::new(0));
        cpu.sed(&mut bus, &AddressingResult::new(0));
        cpu.clv(&mut bus, &AddressingResult::new(0));

        // UNUSED flag should still be 1
        assert!(
            cpu.get_flag(flags::UNUSED),
            "UNUSED flag must remain 1 after flag operations"
        );
    }
}
