// CPU module - 6502 processor implementation
// This module will contain the 6502 CPU emulation

// Sub-modules
pub mod addressing;
pub mod execute;
pub mod instructions;
pub mod opcodes;

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

/// Interrupt Vector Addresses
///
/// The 6502 CPU uses fixed memory addresses to determine where to jump
/// when an interrupt occurs. These vectors are located at the top of the
/// address space.
pub mod vectors {
    /// NMI (Non-Maskable Interrupt) vector address ($FFFA-$FFFB)
    /// NMI is triggered by the PPU at the start of VBlank and cannot be disabled
    pub const NMI: u16 = 0xFFFA;

    /// RESET vector address ($FFFC-$FFFD)
    /// RESET is triggered when the system starts or is reset
    pub const RESET: u16 = 0xFFFC;

    /// IRQ/BRK vector address ($FFFE-$FFFF)
    /// IRQ is triggered by external hardware and can be disabled by the I flag
    /// BRK (software interrupt) also uses this vector
    pub const IRQ: u16 = 0xFFFE;
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

    // Cycle counter
    pub cycles: u64, // Total number of cycles executed
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
            cycles: 0,
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
    /// - PC is loaded from reset vector at $FFFC-$FFFD
    /// - UNUSED flag must remain set
    ///
    /// # Arguments
    /// * `bus` - The memory bus for reading the RESET vector
    ///
    /// # Implementation Note
    /// Unlike NMI and IRQ, RESET does not push any values to the stack.
    /// It simply initializes the CPU state and loads the program counter
    /// from the RESET vector.
    pub fn reset(&mut self, bus: &mut crate::bus::Bus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.cycles = 0;

        // Clear all flags except UNUSED (which must always be 1)
        self.status = 0;
        self.set_flag(flags::UNUSED);
        self.set_flag(flags::INTERRUPT_DISABLE);

        // Load PC from RESET vector ($FFFC-$FFFD)
        let lo = bus.read(vectors::RESET) as u16;
        let hi = bus.read(vectors::RESET.wrapping_add(1)) as u16;
        self.pc = (hi << 8) | lo;

        // RESET takes 7 cycles (actually 8, but we count from 7 for compatibility)
        self.cycles = 7;
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

    // ========================================
    // Interrupt Handling Methods
    // ========================================

    /// Trigger an NMI (Non-Maskable Interrupt)
    ///
    /// NMI is typically triggered by the PPU at the start of VBlank.
    /// Unlike IRQ, NMI cannot be disabled by the Interrupt Disable flag.
    ///
    /// Operation:
    /// 1. Push PC high byte to stack
    /// 2. Push PC low byte to stack
    /// 3. Push status flags with B flag clear to stack
    /// 4. Set I (Interrupt Disable) flag
    /// 5. Load PC from NMI vector at $FFFA-$FFFB
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations and reading NMI vector
    ///
    /// # Implementation Note
    /// - The B flag is pushed as 0 (unlike BRK which pushes it as 1)
    /// - The I flag is set after the interrupt to prevent further IRQs
    /// - NMI cannot be disabled by the I flag
    pub fn nmi(&mut self, bus: &mut crate::bus::Bus) {
        // Push PC to stack (high byte first, then low byte)
        self.stack_push_u16(bus, self.pc);

        // Push status flags with B flag clear and UNUSED flag set
        // NMI pushes status with B=0 (to distinguish from BRK which pushes B=1)
        let status_to_push = (self.status & !flags::BREAK) | flags::UNUSED;
        self.stack_push(bus, status_to_push);

        // Set the Interrupt Disable flag
        self.set_interrupt_disable(true);

        // Load PC from NMI vector ($FFFA-$FFFB)
        let lo = bus.read(vectors::NMI) as u16;
        let hi = bus.read(vectors::NMI.wrapping_add(1)) as u16;
        self.pc = (hi << 8) | lo;
    }

    /// Trigger an IRQ (Interrupt Request)
    ///
    /// IRQ is triggered by external hardware (e.g., cartridge mappers, APU).
    /// IRQ can be disabled by setting the Interrupt Disable flag (I).
    /// If the I flag is set, this method returns immediately without any state changes.
    ///
    /// Operation:
    /// 1. Check if I flag is set; if so, return immediately (IRQ is masked)
    /// 2. Push PC high byte to stack
    /// 3. Push PC low byte to stack
    /// 4. Push status flags with B flag clear to stack
    /// 5. Set I (Interrupt Disable) flag
    /// 6. Load PC from IRQ vector at $FFFE-$FFFF
    ///
    /// # Arguments
    /// * `bus` - The memory bus for stack operations and reading IRQ vector
    ///
    /// # Implementation Note
    /// - The I flag is checked internally; IRQ is ignored when I=1 (maskable interrupt)
    /// - The B flag is pushed as 0 (same as NMI)
    /// - Shares the same vector as BRK ($FFFE-$FFFF)
    /// - The I flag is set after the interrupt to prevent nested IRQs
    pub fn irq(&mut self, bus: &mut crate::bus::Bus) {
        // Check if interrupts are disabled; if so, ignore this IRQ
        if self.get_interrupt_disable() {
            return;
        }

        // Push PC to stack (high byte first, then low byte)
        self.stack_push_u16(bus, self.pc);

        // Push status flags with B flag clear and UNUSED flag set
        // IRQ pushes status with B=0 (to distinguish from BRK which pushes B=1)
        let status_to_push = (self.status & !flags::BREAK) | flags::UNUSED;
        self.stack_push(bus, status_to_push);

        // Set the Interrupt Disable flag
        self.set_interrupt_disable(true);

        // Load PC from IRQ vector ($FFFE-$FFFF)
        let lo = bus.read(vectors::IRQ) as u16;
        let hi = bus.read(vectors::IRQ.wrapping_add(1)) as u16;
        self.pc = (hi << 8) | lo;
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
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up RESET vector at $FFFC-$FFFD to point to $8000
        bus.write(0xFFFC, 0x00); // Low byte
        bus.write(0xFFFD, 0x80); // High byte

        // Modify all registers
        cpu.a = 0x42;
        cpu.x = 0x10;
        cpu.y = 0x20;
        cpu.sp = 0x00;
        cpu.pc = 0x1234;
        cpu.status = 0xFF;

        // Reset the CPU
        cpu.reset(&mut bus);

        // Verify registers are reset
        assert_eq!(cpu.a, 0, "Accumulator should be reset to 0");
        assert_eq!(cpu.x, 0, "X register should be reset to 0");
        assert_eq!(cpu.y, 0, "Y register should be reset to 0");
        assert_eq!(cpu.sp, 0xFD, "Stack pointer should be reset to 0xFD");
        assert_eq!(
            cpu.pc, 0x8000,
            "Program counter should be loaded from RESET vector"
        );

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
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // The UNUSED flag should always be set
        assert!(cpu.get_flag(flags::UNUSED), "UNUSED flag must always be 1");

        // Even after reset
        cpu.reset(&mut bus);
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

    // ========================================
    // Interrupt Handling Tests
    // ========================================

    #[test]
    fn test_reset_loads_pc_from_vector() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up RESET vector at $FFFC-$FFFD
        let reset_addr: u16 = 0x8000;
        bus.write(0xFFFC, (reset_addr & 0xFF) as u8); // Low byte
        bus.write(0xFFFD, (reset_addr >> 8) as u8); // High byte

        // Reset the CPU
        cpu.reset(&mut bus);

        assert_eq!(cpu.pc, reset_addr, "PC should be loaded from RESET vector");
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt Disable flag should be set"
        );
        assert_eq!(cpu.sp, 0xFD, "Stack pointer should be 0xFD");
        assert_eq!(cpu.a, 0, "Accumulator should be 0");
        assert_eq!(cpu.x, 0, "X register should be 0");
        assert_eq!(cpu.y, 0, "Y register should be 0");
    }

    #[test]
    fn test_reset_does_not_push_to_stack() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up RESET vector
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let initial_sp = cpu.sp;

        // Reset the CPU
        cpu.reset(&mut bus);

        // Stack pointer should not change (RESET doesn't push anything)
        assert_eq!(cpu.sp, initial_sp, "Stack pointer should not change");
    }

    #[test]
    fn test_nmi_basic() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up NMI vector at $FFFA-$FFFB
        let nmi_handler_addr: u16 = 0x9000;
        bus.write(0xFFFA, (nmi_handler_addr & 0xFF) as u8); // Low byte
        bus.write(0xFFFB, (nmi_handler_addr >> 8) as u8); // High byte

        // Set initial PC and status
        cpu.pc = 0x1234;
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_interrupt_disable(false); // NMI should work even if I flag is clear
        let initial_sp = cpu.sp;

        // Trigger NMI
        cpu.nmi(&mut bus);

        // Verify PC was loaded from NMI vector
        assert_eq!(
            cpu.pc, nmi_handler_addr,
            "PC should be loaded from NMI vector"
        );

        // Verify Interrupt Disable flag is set
        assert!(
            cpu.get_interrupt_disable(),
            "Interrupt Disable flag should be set"
        );

        // Verify stack operations: PC and status pushed (3 bytes total)
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(3),
            "SP should decrement by 3"
        );

        // Verify PC was pushed to stack
        let pushed_pc_hi = bus.read(0x0100 | (initial_sp as u16));
        let pushed_pc_lo = bus.read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
        let pushed_pc = ((pushed_pc_hi as u16) << 8) | (pushed_pc_lo as u16);
        assert_eq!(pushed_pc, 0x1234, "PC should be pushed to stack");

        // Verify status was pushed with B flag clear
        let pushed_status = bus.read(0x0100 | (initial_sp.wrapping_sub(2) as u16));
        assert_eq!(
            pushed_status & flags::BREAK,
            0,
            "Pushed status should have B flag clear (NMI)"
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
    fn test_nmi_preserves_flags_in_pushed_status() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up NMI vector
        bus.write(0xFFFA, 0x00);
        bus.write(0xFFFB, 0x90);

        // Set various flags
        cpu.set_carry(true);
        cpu.set_zero(true);
        cpu.set_overflow(true);
        cpu.set_negative(true);
        cpu.set_decimal(true);

        let initial_sp = cpu.sp;

        // Trigger NMI
        cpu.nmi(&mut bus);

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

    #[test]
    fn test_nmi_sets_interrupt_disable() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up NMI vector
        bus.write(0xFFFA, 0x00);
        bus.write(0xFFFB, 0x90);

        // Clear interrupt disable flag
        cpu.set_interrupt_disable(false);
        assert!(
            !cpu.get_interrupt_disable(),
            "I flag should initially be clear"
        );

        // Trigger NMI
        cpu.nmi(&mut bus);

        // Verify I flag is now set
        assert!(
            cpu.get_interrupt_disable(),
            "I flag should be set after NMI"
        );
    }

    #[test]
    fn test_irq_basic() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector at $FFFE-$FFFF
        let irq_handler_addr: u16 = 0xA000;
        bus.write(0xFFFE, (irq_handler_addr & 0xFF) as u8); // Low byte
        bus.write(0xFFFF, (irq_handler_addr >> 8) as u8); // High byte

        // Set initial PC and status
        cpu.pc = 0x5678;
        cpu.set_carry(false);
        cpu.set_zero(true);
        cpu.set_interrupt_disable(false); // IRQ requires I flag to be clear
        let initial_sp = cpu.sp;

        // Trigger IRQ
        cpu.irq(&mut bus);

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

        // Verify stack operations: PC and status pushed (3 bytes total)
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(3),
            "SP should decrement by 3"
        );

        // Verify PC was pushed to stack
        let pushed_pc_hi = bus.read(0x0100 | (initial_sp as u16));
        let pushed_pc_lo = bus.read(0x0100 | (initial_sp.wrapping_sub(1) as u16));
        let pushed_pc = ((pushed_pc_hi as u16) << 8) | (pushed_pc_lo as u16);
        assert_eq!(pushed_pc, 0x5678, "PC should be pushed to stack");

        // Verify status was pushed with B flag clear
        let pushed_status = bus.read(0x0100 | (initial_sp.wrapping_sub(2) as u16));
        assert_eq!(
            pushed_status & flags::BREAK,
            0,
            "Pushed status should have B flag clear (IRQ)"
        );
        assert_eq!(
            pushed_status & flags::UNUSED,
            flags::UNUSED,
            "Pushed status should have UNUSED flag set"
        );
    }

    #[test]
    fn test_irq_shares_vector_with_brk() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Both IRQ and BRK use $FFFE-$FFFF
        let handler_addr: u16 = 0xB000;
        bus.write(0xFFFE, (handler_addr & 0xFF) as u8);
        bus.write(0xFFFF, (handler_addr >> 8) as u8);

        cpu.pc = 0x1000;
        cpu.set_interrupt_disable(false); // Ensure I flag is clear

        // Trigger IRQ
        cpu.irq(&mut bus);

        // Verify PC was loaded from shared vector
        assert_eq!(
            cpu.pc, handler_addr,
            "IRQ should use the same vector as BRK"
        );
    }

    #[test]
    fn test_irq_respects_i_flag() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        let irq_handler_addr: u16 = 0xA000;
        bus.write(0xFFFE, (irq_handler_addr & 0xFF) as u8);
        bus.write(0xFFFF, (irq_handler_addr >> 8) as u8);

        // Set initial state
        cpu.pc = 0x5000;
        cpu.set_interrupt_disable(true); // Set I flag to disable IRQ
        let initial_sp = cpu.sp;
        let initial_pc = cpu.pc;

        // Trigger IRQ - should be ignored because I flag is set
        cpu.irq(&mut bus);

        // Verify CPU state is unchanged
        assert_eq!(
            cpu.pc, initial_pc,
            "PC should not change when I flag is set"
        );
        assert_eq!(
            cpu.sp, initial_sp,
            "Stack pointer should not change when I flag is set"
        );
        assert!(cpu.get_interrupt_disable(), "I flag should remain set");
    }

    #[test]
    fn test_irq_works_when_i_flag_clear() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up IRQ vector
        let irq_handler_addr: u16 = 0xA000;
        bus.write(0xFFFE, (irq_handler_addr & 0xFF) as u8);
        bus.write(0xFFFF, (irq_handler_addr >> 8) as u8);

        // Set initial state with I flag clear
        cpu.pc = 0x5000;
        cpu.set_interrupt_disable(false); // Clear I flag to enable IRQ
        let initial_sp = cpu.sp;

        // Trigger IRQ - should be processed because I flag is clear
        cpu.irq(&mut bus);

        // Verify interrupt was processed
        assert_eq!(
            cpu.pc, irq_handler_addr,
            "PC should jump to IRQ handler when I flag is clear"
        );
        assert_eq!(
            cpu.sp,
            initial_sp.wrapping_sub(3),
            "Stack should have PC and status pushed"
        );
        assert!(
            cpu.get_interrupt_disable(),
            "I flag should be set after IRQ"
        );
    }

    #[test]
    fn test_nmi_and_irq_both_push_b_flag_clear() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up vectors
        bus.write(0xFFFA, 0x00); // NMI vector
        bus.write(0xFFFB, 0x90);
        bus.write(0xFFFE, 0x00); // IRQ vector
        bus.write(0xFFFF, 0xA0);

        cpu.pc = 0x1000;

        // Test NMI
        let sp_before_nmi = cpu.sp;
        cpu.nmi(&mut bus);
        let nmi_pushed_status = bus.read(0x0100 | (sp_before_nmi.wrapping_sub(2) as u16));

        // Reset for IRQ test
        cpu.sp = 0xFD;
        cpu.pc = 0x2000;
        cpu.set_interrupt_disable(false); // Ensure I flag is clear for IRQ

        // Test IRQ
        let sp_before_irq = cpu.sp;
        cpu.irq(&mut bus);
        let irq_pushed_status = bus.read(0x0100 | (sp_before_irq.wrapping_sub(2) as u16));

        // Both should have B flag clear (unlike BRK which sets it)
        assert_eq!(
            nmi_pushed_status & flags::BREAK,
            0,
            "NMI should push status with B flag clear"
        );
        assert_eq!(
            irq_pushed_status & flags::BREAK,
            0,
            "IRQ should push status with B flag clear"
        );
    }

    #[test]
    fn test_interrupt_rti_roundtrip_nmi() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up NMI vector
        let nmi_handler = 0x9000;
        bus.write(0xFFFA, (nmi_handler & 0xFF) as u8);
        bus.write(0xFFFB, (nmi_handler >> 8) as u8);

        // Set initial state
        let original_pc = 0x1234;
        cpu.pc = original_pc;
        cpu.set_carry(true);
        cpu.set_zero(false);
        cpu.set_overflow(true);
        cpu.set_interrupt_disable(false);

        let original_carry = cpu.get_carry();
        let original_zero = cpu.get_zero();
        let original_overflow = cpu.get_overflow();

        // Trigger NMI
        cpu.nmi(&mut bus);

        assert_eq!(cpu.pc, nmi_handler, "Should jump to NMI handler");
        assert!(cpu.get_interrupt_disable(), "I flag should be set");

        // Return from interrupt with RTI
        // RTI is implemented in instructions/miscellaneous.rs
        // We need to simulate it here
        let status_from_stack = cpu.stack_pop(&mut bus);
        let current_b_flag = cpu.get_flag(flags::BREAK);
        cpu.status = status_from_stack | flags::UNUSED;
        cpu.update_flag(flags::BREAK, current_b_flag);
        cpu.pc = cpu.stack_pop_u16(&mut bus);

        // Verify PC is restored
        assert_eq!(cpu.pc, original_pc, "PC should be restored");

        // Verify flags are restored
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
    }

    #[test]
    fn test_nested_interrupts() {
        use crate::bus::Bus;
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up vectors
        bus.write(0xFFFA, 0x00); // NMI vector -> $9000
        bus.write(0xFFFB, 0x90);
        bus.write(0xFFFE, 0x00); // IRQ vector -> $A000
        bus.write(0xFFFF, 0xA0);

        let initial_sp = cpu.sp;

        // First interrupt (NMI)
        cpu.pc = 0x1000;
        cpu.nmi(&mut bus);
        assert_eq!(cpu.pc, 0x9000);
        let sp_after_first = cpu.sp;

        // Second interrupt (another NMI, simulating nested handling)
        cpu.pc = 0x9100;
        cpu.nmi(&mut bus);
        assert_eq!(cpu.pc, 0x9000); // Jumps to handler again

        // Stack should have 6 bytes pushed (3 per NMI)
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(6));

        // Return from second interrupt (simulate RTI)
        cpu.sp = cpu.sp.wrapping_add(1);
        let _status = bus.read(0x0100 | (cpu.sp as u16));
        cpu.sp = cpu.sp.wrapping_add(1);
        let lo = bus.read(0x0100 | (cpu.sp as u16)) as u16;
        cpu.sp = cpu.sp.wrapping_add(1);
        let hi = bus.read(0x0100 | (cpu.sp as u16)) as u16;
        cpu.pc = (hi << 8) | lo;

        assert_eq!(cpu.pc, 0x9100, "Should return from second interrupt");
        assert_eq!(cpu.sp, sp_after_first);

        // Return from first interrupt
        cpu.sp = cpu.sp.wrapping_add(1);
        let _status = bus.read(0x0100 | (cpu.sp as u16));
        cpu.sp = cpu.sp.wrapping_add(1);
        let lo = bus.read(0x0100 | (cpu.sp as u16)) as u16;
        cpu.sp = cpu.sp.wrapping_add(1);
        let hi = bus.read(0x0100 | (cpu.sp as u16)) as u16;
        cpu.pc = (hi << 8) | lo;

        assert_eq!(cpu.pc, 0x1000, "Should return from first interrupt");
        assert_eq!(cpu.sp, initial_sp);
    }

    #[test]
    fn test_all_interrupt_vectors_are_different() {
        use crate::cpu::vectors;

        // Verify all three interrupt vectors are at different addresses
        assert_ne!(vectors::NMI, vectors::RESET, "NMI and RESET vectors differ");
        assert_ne!(vectors::NMI, vectors::IRQ, "NMI and IRQ vectors differ");
        assert_ne!(vectors::RESET, vectors::IRQ, "RESET and IRQ vectors differ");

        // Verify they are in the expected locations
        assert_eq!(vectors::NMI, 0xFFFA, "NMI vector at $FFFA");
        assert_eq!(vectors::RESET, 0xFFFC, "RESET vector at $FFFC");
        assert_eq!(vectors::IRQ, 0xFFFE, "IRQ vector at $FFFE");
    }
}
