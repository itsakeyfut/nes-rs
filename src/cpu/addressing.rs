// Addressing modes module for 6502 CPU
// Implements all 13 addressing modes used by the 6502 processor

use crate::bus::Bus;

/// Result of an addressing mode calculation
///
/// Contains information about the effective address, whether a page boundary
/// was crossed (which adds an extra cycle), and the operand value if available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressingResult {
    /// The effective address where the data is located
    pub address: u16,

    /// Whether a page boundary was crossed during address calculation
    /// Some instructions add an extra cycle when this occurs
    pub page_crossed: bool,

    /// The operand value (used for immediate mode and accumulator mode)
    /// None for other modes that read from memory
    pub value: Option<u8>,
}

impl AddressingResult {
    /// Create a new addressing result with an address
    pub fn new(address: u16) -> Self {
        Self {
            address,
            page_crossed: false,
            value: None,
        }
    }

    /// Create a new addressing result with an immediate value
    pub fn immediate(value: u8) -> Self {
        Self {
            address: 0, // Not used for immediate mode
            page_crossed: false,
            value: Some(value),
        }
    }

    /// Set the page_crossed flag
    pub fn with_page_cross(mut self, crossed: bool) -> Self {
        self.page_crossed = crossed;
        self
    }
}

/// Addressing modes supported by the 6502
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    /// Implied - No operand (e.g., CLC, NOP)
    Implied,

    /// Accumulator - Operate on accumulator (e.g., LSR A)
    Accumulator,

    /// Immediate - 8-bit constant (e.g., LDA #$01)
    Immediate,

    /// Zero Page - Address in zero page $00-$FF (e.g., LDA $80)
    ZeroPage,

    /// Zero Page,X - Zero page address + X register (e.g., LDA $80,X)
    ZeroPageX,

    /// Zero Page,Y - Zero page address + Y register (e.g., LDX $80,Y)
    ZeroPageY,

    /// Relative - Signed 8-bit offset for branches (e.g., BNE label)
    Relative,

    /// Absolute - 16-bit address (e.g., LDA $8000)
    Absolute,

    /// Absolute,X - 16-bit address + X register (e.g., LDA $8000,X)
    AbsoluteX,

    /// Absolute,Y - 16-bit address + Y register (e.g., LDA $8000,Y)
    AbsoluteY,

    /// Indirect - 16-bit pointer (JMP only) (e.g., JMP ($FFFC))
    Indirect,

    /// Indexed Indirect - Zero page pointer + X (e.g., LDA ($40,X))
    IndexedIndirect,

    /// Indirect Indexed - Zero page pointer + Y (e.g., LDA ($40),Y)
    IndirectIndexed,
}

/// Helper function to check if a page boundary was crossed
///
/// A page boundary is crossed when adding an offset to a base address
/// causes the high byte of the address to change.
#[inline]
fn page_crossed(base: u16, offset: u8) -> bool {
    let addr = base.wrapping_add(offset as u16);
    (base & 0xFF00) != (addr & 0xFF00)
}

impl super::Cpu {
    // ========================================
    // Implied Mode
    // ========================================

    /// Implied addressing mode - No operand needed
    ///
    /// Used by instructions like CLC, SEC, NOP, etc.
    /// Returns a dummy result since no address is needed.
    pub fn addr_implied(&self) -> AddressingResult {
        AddressingResult::new(0)
    }

    // ========================================
    // Accumulator Mode
    // ========================================

    /// Accumulator addressing mode - Operate on the accumulator register
    ///
    /// Used by instructions like LSR A, ROL A, etc.
    /// Returns the current accumulator value.
    pub fn addr_accumulator(&self) -> AddressingResult {
        AddressingResult::immediate(self.a)
    }

    // ========================================
    // Immediate Mode
    // ========================================

    /// Immediate addressing mode - 8-bit constant operand
    ///
    /// The operand is the byte immediately following the opcode.
    /// Format: LDA #$01
    /// Returns the immediate value and increments PC.
    pub fn addr_immediate(&mut self, bus: &Bus) -> AddressingResult {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        AddressingResult::immediate(value)
    }

    // ========================================
    // Zero Page Mode
    // ========================================

    /// Zero Page addressing mode - Address in page 0 ($00-$FF)
    ///
    /// Uses only one byte for the address, limiting it to page 0.
    /// Format: LDA $80 (reads from $0080)
    /// Faster than absolute addressing (2 bytes vs 3 bytes).
    pub fn addr_zero_page(&mut self, bus: &Bus) -> AddressingResult {
        let addr = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        AddressingResult::new(addr)
    }

    // ========================================
    // Zero Page,X Mode
    // ========================================

    /// Zero Page,X addressing mode - Zero page address + X register
    ///
    /// Adds X register to zero page address with wrapping.
    /// Format: LDA $80,X (if X=5, reads from $0085)
    /// Wraps within zero page: $FF + 2 = $01 (not $0101).
    pub fn addr_zero_page_x(&mut self, bus: &Bus) -> AddressingResult {
        let base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // Add X register and wrap within zero page (0x00-0xFF)
        let addr = base.wrapping_add(self.x) as u16;
        AddressingResult::new(addr)
    }

    // ========================================
    // Zero Page,Y Mode
    // ========================================

    /// Zero Page,Y addressing mode - Zero page address + Y register
    ///
    /// Adds Y register to zero page address with wrapping.
    /// Format: LDX $80,Y (if Y=5, reads from $0085)
    /// Wraps within zero page: $FF + 2 = $01 (not $0101).
    pub fn addr_zero_page_y(&mut self, bus: &Bus) -> AddressingResult {
        let base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // Add Y register and wrap within zero page (0x00-0xFF)
        let addr = base.wrapping_add(self.y) as u16;
        AddressingResult::new(addr)
    }

    // ========================================
    // Relative Mode
    // ========================================

    /// Relative addressing mode - Signed 8-bit offset for branch instructions
    ///
    /// Used only by branch instructions (BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS).
    /// The offset is added to PC to calculate the branch target.
    /// Format: BNE $1234 (offset is calculated by assembler)
    /// Range: -128 to +127 bytes from the instruction after the branch.
    pub fn addr_relative(&mut self, bus: &Bus) -> AddressingResult {
        let offset = bus.read(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1);

        // Calculate the target address by adding signed offset to current PC
        let target = if offset >= 0 {
            self.pc.wrapping_add(offset as u16)
        } else {
            self.pc.wrapping_sub((-offset) as u16)
        };

        // Check if page boundary was crossed
        let crossed = (self.pc & 0xFF00) != (target & 0xFF00);

        AddressingResult::new(target).with_page_cross(crossed)
    }

    // ========================================
    // Absolute Mode
    // ========================================

    /// Absolute addressing mode - 16-bit address
    ///
    /// Uses a full 16-bit address to access any location in memory.
    /// Format: LDA $8000 (reads from $8000)
    /// Address is stored in little-endian format (low byte first).
    pub fn addr_absolute(&mut self, bus: &Bus) -> AddressingResult {
        let lo = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let hi = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let addr = (hi << 8) | lo;
        AddressingResult::new(addr)
    }

    // ========================================
    // Absolute,X Mode
    // ========================================

    /// Absolute,X addressing mode - 16-bit address + X register
    ///
    /// Adds X register to a 16-bit base address.
    /// Format: LDA $8000,X (if X=5, reads from $8005)
    /// Page boundary crossing adds an extra cycle for some instructions.
    pub fn addr_absolute_x(&mut self, bus: &Bus) -> AddressingResult {
        let lo = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let hi = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.x as u16);

        // Check if page boundary was crossed
        let crossed = page_crossed(base, self.x);

        AddressingResult::new(addr).with_page_cross(crossed)
    }

    // ========================================
    // Absolute,Y Mode
    // ========================================

    /// Absolute,Y addressing mode - 16-bit address + Y register
    ///
    /// Adds Y register to a 16-bit base address.
    /// Format: LDA $8000,Y (if Y=5, reads from $8005)
    /// Page boundary crossing adds an extra cycle for some instructions.
    pub fn addr_absolute_y(&mut self, bus: &Bus) -> AddressingResult {
        let lo = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let hi = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.y as u16);

        // Check if page boundary was crossed
        let crossed = page_crossed(base, self.y);

        AddressingResult::new(addr).with_page_cross(crossed)
    }

    // ========================================
    // Indirect Mode
    // ========================================

    /// Indirect addressing mode - 16-bit pointer (JMP only)
    ///
    /// Reads a 16-bit address from the specified pointer location.
    /// Format: JMP ($FFFC) - jumps to address stored at $FFFC-$FFFD
    ///
    /// IMPORTANT BUG: The 6502 has a bug with page boundary wrapping.
    /// If the pointer is at $xxFF, the high byte is read from $xx00 instead of $(xx+1)00.
    /// For example: JMP ($02FF) reads low byte from $02FF and high byte from $0200 (not $0300).
    pub fn addr_indirect(&mut self, bus: &Bus) -> AddressingResult {
        let ptr_lo = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let ptr_hi = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let ptr = (ptr_hi << 8) | ptr_lo;

        // Read the target address from the pointer location
        let lo = bus.read(ptr) as u16;

        // Hardware bug: If pointer is at page boundary ($xxFF),
        // the high byte wraps to $xx00 instead of $(xx+1)00
        let hi_addr = if ptr & 0x00FF == 0x00FF {
            ptr & 0xFF00 // Wrap to start of same page
        } else {
            ptr + 1
        };

        let hi = bus.read(hi_addr) as u16;
        let addr = (hi << 8) | lo;

        AddressingResult::new(addr)
    }

    // ========================================
    // Indexed Indirect Mode (Indirect,X)
    // ========================================

    /// Indexed Indirect addressing mode - ($nn,X)
    ///
    /// Adds X register to zero page address, then reads 16-bit pointer.
    /// Format: LDA ($40,X) - if X=5, reads pointer from $45-$46
    ///
    /// Steps:
    /// 1. Add X to zero page address (with wrapping)
    /// 2. Read 16-bit pointer from that location
    /// 3. Use pointer as the effective address
    ///
    /// Used primarily with X register for table lookups.
    pub fn addr_indexed_indirect(&mut self, bus: &Bus) -> AddressingResult {
        let base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // Add X register to base address and wrap within zero page
        let ptr = base.wrapping_add(self.x);

        // Read 16-bit pointer from zero page (with wrapping)
        let lo = bus.read(ptr as u16) as u16;

        // Wrap within zero page: if ptr is $FF, next byte is at $00
        let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;

        let addr = (hi << 8) | lo;
        AddressingResult::new(addr)
    }

    // ========================================
    // Indirect Indexed Mode (Indirect),Y
    // ========================================

    /// Indirect Indexed addressing mode - ($nn),Y
    ///
    /// Reads 16-bit pointer from zero page, then adds Y register.
    /// Format: LDA ($40),Y - reads pointer from $40-$41, then adds Y
    ///
    /// Steps:
    /// 1. Read 16-bit pointer from zero page address
    /// 2. Add Y register to the pointer value
    /// 3. Use result as the effective address
    ///
    /// Used for accessing data structures with a base pointer.
    /// Page boundary crossing adds an extra cycle for some instructions.
    pub fn addr_indirect_indexed(&mut self, bus: &Bus) -> AddressingResult {
        let ptr = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // Read 16-bit base address from zero page (with wrapping)
        let lo = bus.read(ptr as u16) as u16;
        let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;

        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.y as u16);

        // Check if page boundary was crossed
        let crossed = page_crossed(base, self.y);

        AddressingResult::new(addr).with_page_cross(crossed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    /// Helper function to create a test bus with data
    fn create_test_bus(data: &[(u16, u8)]) -> Bus {
        let mut bus = Bus::new();
        for &(addr, value) in data {
            bus.write(addr, value);
        }
        bus
    }

    // ========================================
    // Implied Mode Tests
    // ========================================

    #[test]
    fn test_addr_implied() {
        let cpu = Cpu::new();
        let result = cpu.addr_implied();

        assert_eq!(result.address, 0);
        assert!(!result.page_crossed);
        assert_eq!(result.value, None);
    }

    // ========================================
    // Accumulator Mode Tests
    // ========================================

    #[test]
    fn test_addr_accumulator() {
        let mut cpu = Cpu::new();
        cpu.a = 0x42;

        let result = cpu.addr_accumulator();

        assert_eq!(result.value, Some(0x42));
        assert!(!result.page_crossed);
    }

    // ========================================
    // Immediate Mode Tests
    // ========================================

    #[test]
    fn test_addr_immediate() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100; // Use RAM address

        let bus = create_test_bus(&[(0x0100, 0x42)]);
        let result = cpu.addr_immediate(&bus);

        assert_eq!(result.value, Some(0x42));
        assert_eq!(cpu.pc, 0x0101);
        assert!(!result.page_crossed);
    }

    // ========================================
    // Zero Page Mode Tests
    // ========================================

    #[test]
    fn test_addr_zero_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;

        let bus = create_test_bus(&[(0x0100, 0x80)]);
        let result = cpu.addr_zero_page(&bus);

        assert_eq!(result.address, 0x0080);
        assert_eq!(cpu.pc, 0x0101);
        assert!(!result.page_crossed);
    }

    // ========================================
    // Zero Page,X Mode Tests
    // ========================================

    #[test]
    fn test_addr_zero_page_x() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;
        cpu.x = 0x05;

        let bus = create_test_bus(&[(0x0100, 0x80)]);
        let result = cpu.addr_zero_page_x(&bus);

        assert_eq!(result.address, 0x0085);
        assert_eq!(cpu.pc, 0x0101);
    }

    #[test]
    fn test_addr_zero_page_x_wrapping() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;
        cpu.x = 0x10;

        // $FF + $10 = $0F (wraps within zero page)
        let bus = create_test_bus(&[(0x0100, 0xFF)]);
        let result = cpu.addr_zero_page_x(&bus);

        assert_eq!(result.address, 0x000F);
    }

    // ========================================
    // Zero Page,Y Mode Tests
    // ========================================

    #[test]
    fn test_addr_zero_page_y() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;
        cpu.y = 0x05;

        let bus = create_test_bus(&[(0x0100, 0x80)]);
        let result = cpu.addr_zero_page_y(&bus);

        assert_eq!(result.address, 0x0085);
        assert_eq!(cpu.pc, 0x0101);
    }

    #[test]
    fn test_addr_zero_page_y_wrapping() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;
        cpu.y = 0x10;

        // $FF + $10 = $0F (wraps within zero page)
        let bus = create_test_bus(&[(0x0100, 0xFF)]);
        let result = cpu.addr_zero_page_y(&bus);

        assert_eq!(result.address, 0x000F);
    }

    // ========================================
    // Relative Mode Tests
    // ========================================

    #[test]
    fn test_addr_relative_forward() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;

        // Positive offset: +10
        let bus = create_test_bus(&[(0x0200, 0x0A)]);
        let result = cpu.addr_relative(&bus);

        assert_eq!(result.address, 0x020B); // 0x0201 + 0x0A
        assert_eq!(cpu.pc, 0x0201);
        assert!(!result.page_crossed);
    }

    #[test]
    fn test_addr_relative_backward() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0250;

        // Negative offset: -16 (0xF0 in two's complement)
        let bus = create_test_bus(&[(0x0250, 0xF0)]);
        let result = cpu.addr_relative(&bus);

        assert_eq!(result.address, 0x0241); // 0x0251 - 0x10
        assert_eq!(cpu.pc, 0x0251);
        assert!(!result.page_crossed);
    }

    #[test]
    fn test_addr_relative_page_crossing() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x01FE;

        // Jump forward across page boundary
        let bus = create_test_bus(&[(0x01FE, 0x05)]);
        let result = cpu.addr_relative(&bus);

        assert_eq!(result.address, 0x0204); // 0x01FF + 0x05
        assert!(result.page_crossed);
    }

    // ========================================
    // Absolute Mode Tests
    // ========================================

    #[test]
    fn test_addr_absolute() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;

        // Address $0734 stored as little-endian: $34 $07
        let bus = create_test_bus(&[(0x0200, 0x34), (0x0201, 0x07)]);
        let result = cpu.addr_absolute(&bus);

        assert_eq!(result.address, 0x0734);
        assert_eq!(cpu.pc, 0x0202);
        assert!(!result.page_crossed);
    }

    // ========================================
    // Absolute,X Mode Tests
    // ========================================

    #[test]
    fn test_addr_absolute_x() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.x = 0x05;

        // Base address $0434 + X(5) = $0439
        let bus = create_test_bus(&[(0x0200, 0x34), (0x0201, 0x04)]);
        let result = cpu.addr_absolute_x(&bus);

        assert_eq!(result.address, 0x0439);
        assert_eq!(cpu.pc, 0x0202);
        assert!(!result.page_crossed);
    }

    #[test]
    fn test_addr_absolute_x_page_crossing() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.x = 0x10;

        // Base address $04FF + X(16) = $050F (crosses page boundary)
        let bus = create_test_bus(&[(0x0200, 0xFF), (0x0201, 0x04)]);
        let result = cpu.addr_absolute_x(&bus);

        assert_eq!(result.address, 0x050F);
        assert!(result.page_crossed);
    }

    // ========================================
    // Absolute,Y Mode Tests
    // ========================================

    #[test]
    fn test_addr_absolute_y() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.y = 0x05;

        // Base address $0434 + Y(5) = $0439
        let bus = create_test_bus(&[(0x0200, 0x34), (0x0201, 0x04)]);
        let result = cpu.addr_absolute_y(&bus);

        assert_eq!(result.address, 0x0439);
        assert_eq!(cpu.pc, 0x0202);
        assert!(!result.page_crossed);
    }

    #[test]
    fn test_addr_absolute_y_page_crossing() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.y = 0x10;

        // Base address $04FF + Y(16) = $050F (crosses page boundary)
        let bus = create_test_bus(&[(0x0200, 0xFF), (0x0201, 0x04)]);
        let result = cpu.addr_absolute_y(&bus);

        assert_eq!(result.address, 0x050F);
        assert!(result.page_crossed);
    }

    // ========================================
    // Indirect Mode Tests
    // ========================================

    #[test]
    fn test_addr_indirect() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;

        // Pointer at $0120 contains address $0634
        let bus = create_test_bus(&[
            (0x0200, 0x20), // Pointer low byte
            (0x0201, 0x01), // Pointer high byte
            (0x0120, 0x34), // Target address low byte
            (0x0121, 0x06), // Target address high byte
        ]);
        let result = cpu.addr_indirect(&bus);

        assert_eq!(result.address, 0x0634);
        assert_eq!(cpu.pc, 0x0202);
    }

    #[test]
    fn test_addr_indirect_page_boundary_bug() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;

        // Test the 6502 page boundary bug
        // Pointer at $01FF should read high byte from $0100 (not $0200)
        let bus = create_test_bus(&[
            (0x0200, 0xFF), // Pointer low byte
            (0x0201, 0x01), // Pointer high byte
            (0x01FF, 0x34), // Target address low byte
            (0x0100, 0x06), // Target address high byte (wraps to start of page)
        ]);
        let result = cpu.addr_indirect(&bus);

        assert_eq!(result.address, 0x0634);
    }

    // ========================================
    // Indexed Indirect Mode Tests
    // ========================================

    #[test]
    fn test_addr_indexed_indirect() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.x = 0x05;

        // Base $40 + X(5) = $45
        // Pointer at $45 contains address $0634
        let bus = create_test_bus(&[
            (0x0200, 0x40),   // Base zero page address
            (0x0045, 0x34),   // Target address low byte
            (0x0046, 0x06),   // Target address high byte
        ]);
        let result = cpu.addr_indexed_indirect(&bus);

        assert_eq!(result.address, 0x0634);
        assert_eq!(cpu.pc, 0x0201);
    }

    #[test]
    fn test_addr_indexed_indirect_wrapping() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.x = 0x10;

        // Base $FF + X(16) = $0F (wraps within zero page)
        let bus = create_test_bus(&[
            (0x0200, 0xFF),   // Base zero page address
            (0x000F, 0x34),   // Target address low byte
            (0x0010, 0x06),   // Target address high byte
        ]);
        let result = cpu.addr_indexed_indirect(&bus);

        assert_eq!(result.address, 0x0634);
    }

    #[test]
    fn test_addr_indexed_indirect_pointer_wrapping() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.x = 0x00;

        // Pointer at $FF should read high byte from $00 (wraps within zero page)
        let bus = create_test_bus(&[
            (0x0200, 0xFF),   // Base zero page address
            (0x00FF, 0x34),   // Target address low byte
            (0x0000, 0x06),   // Target address high byte (wraps to $00)
        ]);
        let result = cpu.addr_indexed_indirect(&bus);

        assert_eq!(result.address, 0x0634);
    }

    // ========================================
    // Indirect Indexed Mode Tests
    // ========================================

    #[test]
    fn test_addr_indirect_indexed() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.y = 0x05;

        // Pointer at $40 contains $0434
        // $0434 + Y(5) = $0439
        let bus = create_test_bus(&[
            (0x0200, 0x40),   // Zero page pointer address
            (0x0040, 0x34),   // Base address low byte
            (0x0041, 0x04),   // Base address high byte
        ]);
        let result = cpu.addr_indirect_indexed(&bus);

        assert_eq!(result.address, 0x0439);
        assert_eq!(cpu.pc, 0x0201);
        assert!(!result.page_crossed);
    }

    #[test]
    fn test_addr_indirect_indexed_page_crossing() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.y = 0x10;

        // Pointer at $40 contains $04FF
        // $04FF + Y(16) = $050F (crosses page boundary)
        let bus = create_test_bus(&[
            (0x0200, 0x40),   // Zero page pointer address
            (0x0040, 0xFF),   // Base address low byte
            (0x0041, 0x04),   // Base address high byte
        ]);
        let result = cpu.addr_indirect_indexed(&bus);

        assert_eq!(result.address, 0x050F);
        assert!(result.page_crossed);
    }

    #[test]
    fn test_addr_indirect_indexed_pointer_wrapping() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0200;
        cpu.y = 0x05;

        // Pointer at $FF should read high byte from $00 (wraps within zero page)
        let bus = create_test_bus(&[
            (0x0200, 0xFF),   // Zero page pointer address
            (0x00FF, 0x34),   // Base address low byte
            (0x0000, 0x04),   // Base address high byte (wraps to $00)
        ]);
        let result = cpu.addr_indirect_indexed(&bus);

        assert_eq!(result.address, 0x0439);
    }

    // ========================================
    // Page Crossing Helper Tests
    // ========================================

    #[test]
    fn test_page_crossed_helper() {
        // No page crossing
        assert!(!page_crossed(0x1234, 0x05));

        // Page crossing
        assert!(page_crossed(0x12FF, 0x01));
        assert!(page_crossed(0x12FF, 0x10));

        // Boundary case
        assert!(!page_crossed(0x12FE, 0x01));
        assert!(page_crossed(0x12FE, 0x02));
    }
}
