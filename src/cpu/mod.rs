// CPU module - 6502 processor implementation
// This module will contain the 6502 CPU emulation

/// Processor Status Flags (P register)
///
/// Bit layout:
/// ```text
/// 7  6  5  4  3  2  1  0
/// N  V  -  B  D  I  Z  C
/// ```
///
/// - N: Negative flag (bit 7)
/// - V: Overflow flag (bit 6)
/// - -: Unused flag (bit 5, always 1)
/// - B: Break command flag (bit 4)
/// - D: Decimal mode flag (bit 3, unused in NES)
/// - I: Interrupt disable flag (bit 2)
/// - Z: Zero flag (bit 1)
/// - C: Carry flag (bit 0)
pub mod flags {
    pub const CARRY: u8 = 0b0000_0001; // Bit 0: C
    pub const ZERO: u8 = 0b0000_0010; // Bit 1: Z
    pub const INTERRUPT_DISABLE: u8 = 0b0000_0100; // Bit 2: I
    pub const DECIMAL: u8 = 0b0000_1000; // Bit 3: D (unused in NES)
    pub const BREAK: u8 = 0b0001_0000; // Bit 4: B
    pub const UNUSED: u8 = 0b0010_0000; // Bit 5: - (always 1)
    pub const OVERFLOW: u8 = 0b0100_0000; // Bit 6: V
    pub const NEGATIVE: u8 = 0b1000_0000; // Bit 7: N
}

/// CPU structure representing the 6502 processor state
pub struct Cpu {
    // Registers
    pub a: u8,      // Accumulator
    pub x: u8,      // Index Register X
    pub y: u8,      // Index Register Y
    pub sp: u8,     // Stack Pointer
    pub pc: u16,    // Program Counter
    pub status: u8, // Processor Status flags
}

impl Cpu {
    /// Create a new CPU instance with default values
    ///
    /// Initializes the CPU to the power-on state according to 6502 specifications:
    /// - All registers (A, X, Y) are set to 0
    /// - Stack pointer (SP) is set to 0xFD
    /// - Program counter (PC) is set to 0 (should be loaded from reset vector $FFFC-$FFFD)
    /// - Status flags:
    ///   - Unused flag (bit 5) is always set to 1
    ///   - Interrupt Disable flag (I) is set to 1
    ///   - All other flags are cleared
    pub fn new() -> Self {
        let mut cpu = Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            status: 0,
        };

        // Initialize status register with required flags
        // The UNUSED flag must always be 1
        cpu.set_flag(flags::UNUSED);
        cpu.set_flag(flags::INTERRUPT_DISABLE);

        cpu
    }

    /// Reset the CPU to initial state
    ///
    /// This simulates the RESET signal on the 6502 processor.
    /// According to 6502 specifications:
    /// - A, X, Y registers are not affected (but we clear them for consistency)
    /// - SP is decremented by 3 (but we set it to 0xFD as standard practice)
    /// - Interrupt Disable flag is set
    /// - PC should be loaded from reset vector at $FFFC-$FFFD
    /// - UNUSED flag must remain set
    ///
    /// Note: In a complete emulator, PC would be loaded from the reset vector
    /// in memory. For now, we initialize it to 0x0000 to ensure a safe state.
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;

        // Clear all flags except UNUSED (which must always be 1)
        self.status = 0;
        self.set_flag(flags::UNUSED);
        self.set_flag(flags::INTERRUPT_DISABLE);

        // Initialize PC to a safe value
        // TODO: Load the reset vector ($FFFC-$FFFD) from memory once the bus is wired
        self.pc = 0x0000;
    }

    // ========================================
    // Status Flag Manipulation Methods
    // ========================================

    /// Get the value of a specific flag
    #[inline]
    pub fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) != 0
    }

    /// Set a specific flag to 1
    #[inline]
    pub fn set_flag(&mut self, flag: u8) {
        self.status |= flag;
    }

    /// Clear a specific flag (set to 0)
    #[inline]
    pub fn clear_flag(&mut self, flag: u8) {
        self.status &= !flag;
    }

    /// Update a flag based on a condition
    #[inline]
    pub fn update_flag(&mut self, flag: u8, condition: bool) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    // ========================================
    // Individual Flag Getters
    // ========================================

    /// Get Carry flag (C)
    #[inline]
    pub fn get_carry(&self) -> bool {
        self.get_flag(flags::CARRY)
    }

    /// Get Zero flag (Z)
    #[inline]
    pub fn get_zero(&self) -> bool {
        self.get_flag(flags::ZERO)
    }

    /// Get Interrupt Disable flag (I)
    #[inline]
    pub fn get_interrupt_disable(&self) -> bool {
        self.get_flag(flags::INTERRUPT_DISABLE)
    }

    /// Get Decimal mode flag (D) - unused in NES
    #[inline]
    pub fn get_decimal(&self) -> bool {
        self.get_flag(flags::DECIMAL)
    }

    /// Get Break flag (B)
    #[inline]
    pub fn get_break(&self) -> bool {
        self.get_flag(flags::BREAK)
    }

    /// Get Overflow flag (V)
    #[inline]
    pub fn get_overflow(&self) -> bool {
        self.get_flag(flags::OVERFLOW)
    }

    /// Get Negative flag (N)
    #[inline]
    pub fn get_negative(&self) -> bool {
        self.get_flag(flags::NEGATIVE)
    }

    // ========================================
    // Individual Flag Setters
    // ========================================

    /// Set Carry flag (C)
    #[inline]
    pub fn set_carry(&mut self, value: bool) {
        self.update_flag(flags::CARRY, value);
    }

    /// Set Zero flag (Z)
    #[inline]
    pub fn set_zero(&mut self, value: bool) {
        self.update_flag(flags::ZERO, value);
    }

    /// Set Interrupt Disable flag (I)
    #[inline]
    pub fn set_interrupt_disable(&mut self, value: bool) {
        self.update_flag(flags::INTERRUPT_DISABLE, value);
    }

    /// Set Decimal mode flag (D) - unused in NES
    #[inline]
    pub fn set_decimal(&mut self, value: bool) {
        self.update_flag(flags::DECIMAL, value);
    }

    /// Set Break flag (B)
    #[inline]
    pub fn set_break(&mut self, value: bool) {
        self.update_flag(flags::BREAK, value);
    }

    /// Set Overflow flag (V)
    #[inline]
    pub fn set_overflow(&mut self, value: bool) {
        self.update_flag(flags::OVERFLOW, value);
    }

    /// Set Negative flag (N)
    #[inline]
    pub fn set_negative(&mut self, value: bool) {
        self.update_flag(flags::NEGATIVE, value);
    }

    // ========================================
    // Common Flag Update Patterns
    // ========================================

    /// Update Zero and Negative flags based on a value
    /// This is a common pattern after load and arithmetic operations
    #[inline]
    pub fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.set_zero(value == 0);
        self.set_negative((value & 0x80) != 0);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // CPU Initialization Tests
    // ========================================

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();

        // Test register initialization
        assert_eq!(cpu.a, 0, "Accumulator should be initialized to 0");
        assert_eq!(cpu.x, 0, "X register should be initialized to 0");
        assert_eq!(cpu.y, 0, "Y register should be initialized to 0");
        assert_eq!(cpu.sp, 0xFD, "Stack pointer should be initialized to 0xFD");
        assert_eq!(cpu.pc, 0, "Program counter should be initialized to 0");

        // Test status flag initialization
        assert_eq!(
            cpu.status, 0x24,
            "Status should be 0x24 (UNUSED | INTERRUPT_DISABLE)"
        );
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED flag must be set");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt Disable flag should be set on initialization"
        );
        assert!(!cpu.get_carry(), "Carry flag should be clear");
        assert!(!cpu.get_zero(), "Zero flag should be clear");
        assert!(!cpu.get_decimal(), "Decimal flag should be clear");
        assert!(!cpu.get_break(), "Break flag should be clear");
        assert!(!cpu.get_overflow(), "Overflow flag should be clear");
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_cpu_default() {
        let cpu1 = Cpu::new();
        let cpu2 = Cpu::default();

        assert_eq!(cpu1.a, cpu2.a);
        assert_eq!(cpu1.x, cpu2.x);
        assert_eq!(cpu1.y, cpu2.y);
        assert_eq!(cpu1.sp, cpu2.sp);
        assert_eq!(cpu1.pc, cpu2.pc);
        assert_eq!(cpu1.status, cpu2.status);
    }

    #[test]
    fn test_cpu_reset() {
        let mut cpu = Cpu::new();

        // Modify all registers
        cpu.a = 0x42;
        cpu.x = 0x10;
        cpu.y = 0x20;
        cpu.sp = 0x00;
        cpu.pc = 0x1234;
        cpu.status = 0xFF;

        // Reset the CPU
        cpu.reset();

        // Verify registers are reset
        assert_eq!(cpu.a, 0, "Accumulator should be reset to 0");
        assert_eq!(cpu.x, 0, "X register should be reset to 0");
        assert_eq!(cpu.y, 0, "Y register should be reset to 0");
        assert_eq!(cpu.sp, 0xFD, "Stack pointer should be reset to 0xFD");
        assert_eq!(cpu.pc, 0x0000, "Program counter should be reset to 0x0000");

        // Verify status flags
        assert_eq!(
            cpu.status, 0x24,
            "Status should be 0x24 after reset (UNUSED | INTERRUPT_DISABLE)"
        );
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED flag must remain set");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt Disable flag should be set after reset"
        );
    }

    // ========================================
    // Flag Manipulation Tests
    // ========================================

    #[test]
    fn test_flag_set_and_get() {
        let mut cpu = Cpu::new();

        // Clear all flags first (except UNUSED which should always be set)
        cpu.status = 0;
        cpu.set_flag(flags::UNUSED);

        // Test setting each flag
        cpu.set_flag(flags::CARRY);
        assert!(cpu.get_flag(flags::CARRY), "Carry flag should be set");

        cpu.set_flag(flags::ZERO);
        assert!(cpu.get_flag(flags::ZERO), "Zero flag should be set");

        cpu.set_flag(flags::OVERFLOW);
        assert!(cpu.get_flag(flags::OVERFLOW), "Overflow flag should be set");

        cpu.set_flag(flags::NEGATIVE);
        assert!(cpu.get_flag(flags::NEGATIVE), "Negative flag should be set");
    }

    #[test]
    fn test_flag_clear() {
        let mut cpu = Cpu::new();

        // Set all flags
        cpu.status = 0xFF;

        // Clear each flag individually
        cpu.clear_flag(flags::CARRY);
        assert!(!cpu.get_flag(flags::CARRY), "Carry flag should be clear");

        cpu.clear_flag(flags::ZERO);
        assert!(!cpu.get_flag(flags::ZERO), "Zero flag should be clear");

        cpu.clear_flag(flags::OVERFLOW);
        assert!(
            !cpu.get_flag(flags::OVERFLOW),
            "Overflow flag should be clear"
        );

        cpu.clear_flag(flags::NEGATIVE);
        assert!(
            !cpu.get_flag(flags::NEGATIVE),
            "Negative flag should be clear"
        );
    }

    #[test]
    fn test_flag_update() {
        let mut cpu = Cpu::new();

        // Test update_flag with true
        cpu.update_flag(flags::CARRY, true);
        assert!(cpu.get_flag(flags::CARRY), "Flag should be set when true");

        // Test update_flag with false
        cpu.update_flag(flags::CARRY, false);
        assert!(
            !cpu.get_flag(flags::CARRY),
            "Flag should be clear when false"
        );
    }

    // ========================================
    // Individual Flag Getter/Setter Tests
    // ========================================

    #[test]
    fn test_carry_flag() {
        let mut cpu = Cpu::new();

        assert!(!cpu.get_carry(), "Carry flag should initially be clear");

        cpu.set_carry(true);
        assert!(cpu.get_carry(), "Carry flag should be set");
        assert_eq!(
            cpu.status & flags::CARRY,
            flags::CARRY,
            "Status register should have carry bit set"
        );

        cpu.set_carry(false);
        assert!(!cpu.get_carry(), "Carry flag should be clear");
        assert_eq!(
            cpu.status & flags::CARRY,
            0,
            "Status register should have carry bit clear"
        );
    }

    #[test]
    fn test_zero_flag() {
        let mut cpu = Cpu::new();

        assert!(!cpu.get_zero(), "Zero flag should initially be clear");

        cpu.set_zero(true);
        assert!(cpu.get_zero(), "Zero flag should be set");

        cpu.set_zero(false);
        assert!(!cpu.get_zero(), "Zero flag should be clear");
    }

    #[test]
    fn test_interrupt_disable_flag() {
        let mut cpu = Cpu::new();

        // Initially set after initialization
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be set initially"
        );

        cpu.set_interrupt_disable(false);
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should be clear"
        );

        cpu.set_interrupt_disable(true);
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt disable should be set"
        );
    }

    #[test]
    fn test_decimal_flag() {
        let mut cpu = Cpu::new();

        assert!(!cpu.get_decimal(), "Decimal flag should initially be clear");

        cpu.set_decimal(true);
        assert!(cpu.get_decimal(), "Decimal flag should be set");

        cpu.set_decimal(false);
        assert!(!cpu.get_decimal(), "Decimal flag should be clear");
    }

    #[test]
    fn test_break_flag() {
        let mut cpu = Cpu::new();

        assert!(!cpu.get_break(), "Break flag should initially be clear");

        cpu.set_break(true);
        assert!(cpu.get_break(), "Break flag should be set");

        cpu.set_break(false);
        assert!(!cpu.get_break(), "Break flag should be clear");
    }

    #[test]
    fn test_overflow_flag() {
        let mut cpu = Cpu::new();

        assert!(
            !cpu.get_overflow(),
            "Overflow flag should initially be clear"
        );

        cpu.set_overflow(true);
        assert!(cpu.get_overflow(), "Overflow flag should be set");

        cpu.set_overflow(false);
        assert!(!cpu.get_overflow(), "Overflow flag should be clear");
    }

    #[test]
    fn test_negative_flag() {
        let mut cpu = Cpu::new();

        assert!(
            !cpu.get_negative(),
            "Negative flag should initially be clear"
        );

        cpu.set_negative(true);
        assert!(cpu.get_negative(), "Negative flag should be set");

        cpu.set_negative(false);
        assert!(!cpu.get_negative(), "Negative flag should be clear");
    }

    #[test]
    fn test_unused_flag_always_set() {
        let mut cpu = Cpu::new();

        // The UNUSED flag should always be set
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED flag must always be 1");

        // Even after reset
        cpu.reset();
        assert!(
            cpu.get_flag(flags::UNUSED),
            "UNUSED flag must remain 1 after reset"
        );
    }

    // ========================================
    // Common Flag Pattern Tests
    // ========================================

    #[test]
    fn test_update_zero_and_negative_flags_with_zero() {
        let mut cpu = Cpu::new();

        cpu.update_zero_and_negative_flags(0x00);

        assert!(cpu.get_zero(), "Zero flag should be set when value is 0x00");
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when value is 0x00"
        );
    }

    #[test]
    fn test_update_zero_and_negative_flags_with_positive() {
        let mut cpu = Cpu::new();

        cpu.update_zero_and_negative_flags(0x42);

        assert!(
            !cpu.get_zero(),
            "Zero flag should be clear when value is positive"
        );
        assert!(
            !cpu.get_negative(),
            "Negative flag should be clear when bit 7 is 0"
        );
    }

    #[test]
    fn test_update_zero_and_negative_flags_with_negative() {
        let mut cpu = Cpu::new();

        cpu.update_zero_and_negative_flags(0x80);

        assert!(
            !cpu.get_zero(),
            "Zero flag should be clear when value is 0x80"
        );
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when bit 7 is 1"
        );
    }

    #[test]
    fn test_update_zero_and_negative_flags_with_ff() {
        let mut cpu = Cpu::new();

        cpu.update_zero_and_negative_flags(0xFF);

        assert!(
            !cpu.get_zero(),
            "Zero flag should be clear when value is 0xFF"
        );
        assert!(
            cpu.get_negative(),
            "Negative flag should be set when value is 0xFF"
        );
    }

    // ========================================
    // Multiple Flag Operations Tests
    // ========================================

    #[test]
    fn test_multiple_flags_simultaneously() {
        let mut cpu = Cpu::new();

        // Set multiple flags
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_overflow(true);

        // Verify all are set
        assert!(cpu.get_carry(), "Carry should be set");
        assert!(cpu.get_zero(), "Zero should be set");
        assert!(cpu.get_overflow(), "Overflow should be set");

        // Clear one flag, others should remain
        cpu.set_zero(false);
        assert!(cpu.get_carry(), "Carry should still be set");
        assert!(!cpu.get_zero(), "Zero should be clear");
        assert!(cpu.get_overflow(), "Overflow should still be set");
    }

    #[test]
    fn test_status_register_direct_manipulation() {
        let mut cpu = Cpu::new();

        // Set specific status pattern
        cpu.status = 0b11000011; // N, V, C, Z set

        assert!(cpu.get_negative(), "Negative should be set");
        assert!(cpu.get_overflow(), "Overflow should be set");
        assert!(cpu.get_carry(), "Carry should be set");
        assert!(cpu.get_zero(), "Zero should be set");
        assert!(
            !cpu.get_interrupt_disable(),
            "Interrupt disable should be clear"
        );
        assert!(!cpu.get_decimal(), "Decimal should be clear");
    }
}
