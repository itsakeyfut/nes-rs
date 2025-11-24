// Disassembler - 6502 instruction disassembly
//
// Converts raw bytes into human-readable 6502 assembly instructions.

use crate::bus::Bus;
use crate::cpu::opcodes::OPCODE_TABLE;

/// Disassembled instruction
///
/// Contains all information about a disassembled instruction.
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    /// Address where the instruction is located
    pub address: u16,

    /// Opcode byte
    pub opcode: u8,

    /// Mnemonic (e.g., "LDA", "STA", "JMP")
    pub mnemonic: String,

    /// Addressing mode (e.g., "Immediate", "Absolute")
    pub addressing_mode: String,

    /// Operand bytes
    pub operands: Vec<u8>,

    /// Total instruction length in bytes
    pub length: u8,
}

impl DisassembledInstruction {
    /// Format the instruction as assembly code
    ///
    /// # Returns
    ///
    /// A string like "LDA #$42" or "JMP $8000"
    pub fn format_assembly(&self) -> String {
        let operand_str = match self.addressing_mode.as_str() {
            "Implied" | "Accumulator" => String::new(),
            "Immediate" => {
                if !self.operands.is_empty() {
                    format!(" #${:02X}", self.operands[0])
                } else {
                    String::new()
                }
            }
            "ZeroPage" => {
                if !self.operands.is_empty() {
                    format!(" ${:02X}", self.operands[0])
                } else {
                    String::new()
                }
            }
            "ZeroPageX" => {
                if !self.operands.is_empty() {
                    format!(" ${:02X},X", self.operands[0])
                } else {
                    String::new()
                }
            }
            "ZeroPageY" => {
                if !self.operands.is_empty() {
                    format!(" ${:02X},Y", self.operands[0])
                } else {
                    String::new()
                }
            }
            "Absolute" => {
                if self.operands.len() >= 2 {
                    let addr = (self.operands[1] as u16) << 8 | (self.operands[0] as u16);
                    format!(" ${:04X}", addr)
                } else {
                    String::new()
                }
            }
            "AbsoluteX" => {
                if self.operands.len() >= 2 {
                    let addr = (self.operands[1] as u16) << 8 | (self.operands[0] as u16);
                    format!(" ${:04X},X", addr)
                } else {
                    String::new()
                }
            }
            "AbsoluteY" => {
                if self.operands.len() >= 2 {
                    let addr = (self.operands[1] as u16) << 8 | (self.operands[0] as u16);
                    format!(" ${:04X},Y", addr)
                } else {
                    String::new()
                }
            }
            "Indirect" => {
                if self.operands.len() >= 2 {
                    let addr = (self.operands[1] as u16) << 8 | (self.operands[0] as u16);
                    format!(" (${:04X})", addr)
                } else {
                    String::new()
                }
            }
            "IndexedIndirect" => {
                if !self.operands.is_empty() {
                    format!(" (${:02X},X)", self.operands[0])
                } else {
                    String::new()
                }
            }
            "IndirectIndexed" => {
                if !self.operands.is_empty() {
                    format!(" (${:02X}),Y", self.operands[0])
                } else {
                    String::new()
                }
            }
            "Relative" => {
                if !self.operands.is_empty() {
                    let offset = self.operands[0] as i8;
                    let target = self.address.wrapping_add(2).wrapping_add(offset as u16);
                    format!(" ${:04X}", target)
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        format!("{}{}", self.mnemonic, operand_str)
    }

    /// Format the instruction bytes as hex
    ///
    /// # Returns
    ///
    /// A string like "A9 42" or "4C 00 80"
    pub fn format_bytes(&self) -> String {
        let mut result = format!("{:02X}", self.opcode);

        for operand in &self.operands {
            result.push_str(&format!(" {:02X}", operand));
        }

        result
    }
}

impl std::fmt::Display for DisassembledInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "${:04X}  {:8}  {}",
            self.address,
            self.format_bytes(),
            self.format_assembly()
        )
    }
}

/// Disassemble an instruction at the specified address
///
/// # Arguments
///
/// * `addr` - Address to disassemble from
/// * `bus` - Reference to the bus for reading memory
///
/// # Returns
///
/// A disassembled instruction
///
/// # Example
///
/// ```ignore
/// use nes_rs::debug::disassemble_instruction;
/// use nes_rs::bus::Bus;
///
/// let mut bus = Bus::new();
/// let instruction = disassemble_instruction(0x8000, &mut bus);
/// println!("{}", instruction.format_assembly());
/// ```
pub fn disassemble_instruction(addr: u16, bus: &mut Bus) -> DisassembledInstruction {
    let opcode = bus.read(addr);
    let opcode_info = &OPCODE_TABLE[opcode as usize];

    let mut operands = Vec::new();
    for i in 1..opcode_info.bytes {
        operands.push(bus.read(addr.wrapping_add(i as u16)));
    }

    let addressing_mode = format!("{:?}", opcode_info.mode);

    DisassembledInstruction {
        address: addr,
        opcode,
        mnemonic: opcode_info.mnemonic.to_string(),
        addressing_mode,
        operands,
        length: opcode_info.bytes,
    }
}

/// Disassemble a range of memory
///
/// # Arguments
///
/// * `start` - Start address
/// * `end` - End address (inclusive)
/// * `bus` - Reference to the bus for reading memory
///
/// # Returns
///
/// A vector of disassembled instructions
///
/// # Example
///
/// ```ignore
/// use nes_rs::debug::disassemble_range;
/// use nes_rs::bus::Bus;
///
/// let mut bus = Bus::new();
/// let instructions = disassemble_range(0x8000, 0x8010, &mut bus);
/// for instr in instructions {
///     println!("{}", instr);
/// }
/// ```
pub fn disassemble_range(start: u16, end: u16, bus: &mut Bus) -> Vec<DisassembledInstruction> {
    let mut instructions = Vec::new();
    let mut addr = start;

    while addr <= end {
        let instruction = disassemble_instruction(addr, bus);
        addr = addr.wrapping_add(instruction.length as u16);
        instructions.push(instruction);

        // Prevent infinite loop if we wrap around
        if addr < start {
            break;
        }
    }

    instructions
}

/// Disassemble a specific number of instructions
///
/// # Arguments
///
/// * `start` - Start address
/// * `count` - Number of instructions to disassemble
/// * `bus` - Reference to the bus for reading memory
///
/// # Returns
///
/// A vector of disassembled instructions
///
/// # Example
///
/// ```ignore
/// use nes_rs::debug::disassemble_count;
/// use nes_rs::bus::Bus;
///
/// let mut bus = Bus::new();
/// let instructions = disassemble_count(0x8000, 10, &mut bus);
/// for instr in instructions {
///     println!("{}", instr);
/// }
/// ```
pub fn disassemble_count(start: u16, count: usize, bus: &mut Bus) -> Vec<DisassembledInstruction> {
    let mut instructions = Vec::new();
    let mut addr = start;

    for _ in 0..count {
        let instruction = disassemble_instruction(addr, bus);
        addr = addr.wrapping_add(instruction.length as u16);
        instructions.push(instruction);
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_nop() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xEA); // NOP

        let instr = disassemble_instruction(0x8000, &mut bus);

        assert_eq!(instr.opcode, 0xEA);
        assert_eq!(instr.mnemonic, "NOP");
        assert_eq!(instr.length, 1);
        assert!(instr.operands.is_empty());
    }

    #[test]
    fn test_disassemble_lda_immediate() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xA9); // LDA #$42
        bus.write(0x8001, 0x42);

        let instr = disassemble_instruction(0x8000, &mut bus);

        assert_eq!(instr.opcode, 0xA9);
        assert_eq!(instr.mnemonic, "LDA");
        assert_eq!(instr.length, 2);
        assert_eq!(instr.operands, vec![0x42]);

        let assembly = instr.format_assembly();
        assert!(assembly.contains("LDA"));
        assert!(assembly.contains("#$42"));
    }

    #[test]
    fn test_disassemble_jmp_absolute() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x4C); // JMP $1234
        bus.write(0x8001, 0x34);
        bus.write(0x8002, 0x12);

        let instr = disassemble_instruction(0x8000, &mut bus);

        assert_eq!(instr.opcode, 0x4C);
        assert_eq!(instr.mnemonic, "JMP");
        assert_eq!(instr.length, 3);
        assert_eq!(instr.operands, vec![0x34, 0x12]);

        let assembly = instr.format_assembly();
        assert!(assembly.contains("JMP"));
        assert!(assembly.contains("$1234"));
    }

    #[test]
    fn test_disassemble_format_bytes() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x4C); // JMP $1234
        bus.write(0x8001, 0x34);
        bus.write(0x8002, 0x12);

        let instr = disassemble_instruction(0x8000, &mut bus);
        let bytes = instr.format_bytes();

        assert_eq!(bytes, "4C 34 12");
    }

    #[test]
    fn test_disassemble_count() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xEA); // NOP
        bus.write(0x8001, 0xEA); // NOP
        bus.write(0x8002, 0xA9); // LDA #$42
        bus.write(0x8003, 0x42);

        let instructions = disassemble_count(0x8000, 3, &mut bus);

        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].mnemonic, "NOP");
        assert_eq!(instructions[1].mnemonic, "NOP");
        assert_eq!(instructions[2].mnemonic, "LDA");
    }

    #[test]
    fn test_disassemble_range() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xEA); // NOP
        bus.write(0x8001, 0xA9); // LDA #$42
        bus.write(0x8002, 0x42);
        bus.write(0x8003, 0xEA); // NOP

        let instructions = disassemble_range(0x8000, 0x8003, &mut bus);

        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].address, 0x8000);
        assert_eq!(instructions[1].address, 0x8001);
        assert_eq!(instructions[2].address, 0x8003);
    }

    #[test]
    fn test_disassemble_illegal_opcode() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x02); // ??? (illegal opcode)

        let instr = disassemble_instruction(0x8000, &mut bus);

        assert_eq!(instr.opcode, 0x02);
        assert_eq!(instr.mnemonic, "???");
        assert_eq!(instr.length, 1);
    }

    #[test]
    fn test_disassemble_display_format() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x78); // SEI

        let instr = disassemble_instruction(0x8000, &mut bus);
        let display = format!("{}", instr);

        // Should match format: "$8000  78        SEI"
        assert!(display.contains("$8000"));
        assert!(display.contains("78"));
        assert!(display.contains("SEI"));
    }

    #[test]
    fn test_all_addressing_modes() {
        let mut bus = Bus::new();

        // Implied
        bus.write(0x8000, 0x18); // CLC
        let instr = disassemble_instruction(0x8000, &mut bus);
        assert_eq!(instr.format_assembly(), "CLC");

        // Accumulator
        bus.write(0x8010, 0x0A); // ASL A
        let instr = disassemble_instruction(0x8010, &mut bus);
        assert_eq!(instr.format_assembly(), "ASL");

        // Immediate
        bus.write(0x8020, 0xA9); // LDA #$42
        bus.write(0x8021, 0x42);
        let instr = disassemble_instruction(0x8020, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA #$42");

        // Zero Page
        bus.write(0x8030, 0xA5); // LDA $80
        bus.write(0x8031, 0x80);
        let instr = disassemble_instruction(0x8030, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA $80");

        // Zero Page,X
        bus.write(0x8040, 0xB5); // LDA $80,X
        bus.write(0x8041, 0x80);
        let instr = disassemble_instruction(0x8040, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA $80,X");

        // Zero Page,Y
        bus.write(0x8050, 0xB6); // LDX $80,Y
        bus.write(0x8051, 0x80);
        let instr = disassemble_instruction(0x8050, &mut bus);
        assert_eq!(instr.format_assembly(), "LDX $80,Y");

        // Relative
        bus.write(0x8060, 0xF0); // BEQ +10
        bus.write(0x8061, 0x0A);
        let instr = disassemble_instruction(0x8060, &mut bus);
        let target = 0x8062 + 0x0A;
        assert_eq!(instr.format_assembly(), format!("BEQ ${:04X}", target));

        // Absolute
        bus.write(0x8070, 0xAD); // LDA $2002
        bus.write(0x8071, 0x02);
        bus.write(0x8072, 0x20);
        let instr = disassemble_instruction(0x8070, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA $2002");

        // Absolute,X
        bus.write(0x8080, 0xBD); // LDA $2000,X
        bus.write(0x8081, 0x00);
        bus.write(0x8082, 0x20);
        let instr = disassemble_instruction(0x8080, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA $2000,X");

        // Absolute,Y
        bus.write(0x8090, 0xB9); // LDA $2000,Y
        bus.write(0x8091, 0x00);
        bus.write(0x8092, 0x20);
        let instr = disassemble_instruction(0x8090, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA $2000,Y");

        // Indirect
        bus.write(0x80A0, 0x6C); // JMP ($1234)
        bus.write(0x80A1, 0x34);
        bus.write(0x80A2, 0x12);
        let instr = disassemble_instruction(0x80A0, &mut bus);
        assert_eq!(instr.format_assembly(), "JMP ($1234)");

        // Indexed Indirect
        bus.write(0x80B0, 0xA1); // LDA ($40,X)
        bus.write(0x80B1, 0x40);
        let instr = disassemble_instruction(0x80B0, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA ($40,X)");

        // Indirect Indexed
        bus.write(0x80C0, 0xB1); // LDA ($40),Y
        bus.write(0x80C1, 0x40);
        let instr = disassemble_instruction(0x80C0, &mut bus);
        assert_eq!(instr.format_assembly(), "LDA ($40),Y");
    }

    #[test]
    fn test_branch_instructions_all_directions() {
        let mut bus = Bus::new();

        // Forward branch
        bus.write(0x8000, 0xD0); // BNE +10
        bus.write(0x8001, 0x0A);
        let instr = disassemble_instruction(0x8000, &mut bus);
        assert_eq!(instr.format_assembly(), "BNE $800C");

        // Backward branch (negative offset)
        bus.write(0x8010, 0xF0); // BEQ -16
        bus.write(0x8011, 0xF0);
        let instr = disassemble_instruction(0x8010, &mut bus);
        assert_eq!(instr.format_assembly(), "BEQ $8002");
    }

    #[test]
    fn test_example_output_format() {
        // Test the example output from the issue
        let mut bus = Bus::new();

        // $8000: 78        SEI
        bus.write(0x8000, 0x78);
        // $8001: D8        CLD
        bus.write(0x8001, 0xD8);
        // $8002: A2 FF    LDX #$FF
        bus.write(0x8002, 0xA2);
        bus.write(0x8003, 0xFF);
        // $8004: 9A        TXS
        bus.write(0x8004, 0x9A);
        // $8005: AD 02 20  LDA $2002
        bus.write(0x8005, 0xAD);
        bus.write(0x8006, 0x02);
        bus.write(0x8007, 0x20);

        let instructions = disassemble_count(0x8000, 5, &mut bus);

        assert_eq!(instructions[0].format_assembly(), "SEI");
        assert_eq!(instructions[0].format_bytes(), "78");

        assert_eq!(instructions[1].format_assembly(), "CLD");
        assert_eq!(instructions[1].format_bytes(), "D8");

        assert_eq!(instructions[2].format_assembly(), "LDX #$FF");
        assert_eq!(instructions[2].format_bytes(), "A2 FF");

        assert_eq!(instructions[3].format_assembly(), "TXS");
        assert_eq!(instructions[3].format_bytes(), "9A");

        assert_eq!(instructions[4].format_assembly(), "LDA $2002");
        assert_eq!(instructions[4].format_bytes(), "AD 02 20");
    }

    #[test]
    fn test_instruction_length_calculation() {
        let mut bus = Bus::new();

        // 1-byte instructions
        bus.write(0x8000, 0xEA); // NOP
        let instr = disassemble_instruction(0x8000, &mut bus);
        assert_eq!(instr.length, 1);

        // 2-byte instructions
        bus.write(0x8010, 0xA9); // LDA #$42
        bus.write(0x8011, 0x42);
        let instr = disassemble_instruction(0x8010, &mut bus);
        assert_eq!(instr.length, 2);

        // 3-byte instructions
        bus.write(0x8020, 0xAD); // LDA $2002
        bus.write(0x8021, 0x02);
        bus.write(0x8022, 0x20);
        let instr = disassemble_instruction(0x8020, &mut bus);
        assert_eq!(instr.length, 3);
    }

    #[test]
    fn test_disassemble_wrapping() {
        let mut bus = Bus::new();

        // Test disassembly near the end of address space
        bus.write(0xFFFE, 0xEA); // NOP
        bus.write(0xFFFF, 0xEA); // NOP

        let instr = disassemble_instruction(0xFFFE, &mut bus);
        assert_eq!(instr.address, 0xFFFE);
        assert_eq!(instr.mnemonic, "NOP");
    }
}
