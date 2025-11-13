// CPU execution and trace logging module

use crate::bus::Bus;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::opcodes::OPCODE_TABLE;
use crate::cpu::Cpu;

impl Cpu {
    /// Execute one CPU instruction
    ///
    /// This function fetches the next opcode, decodes it, executes the instruction,
    /// and updates the cycle counter.
    ///
    /// # Returns
    /// The number of cycles consumed by this instruction
    pub fn step(&mut self, bus: &mut Bus) -> u8 {
        // Fetch opcode from current PC
        let opcode = bus.read(self.pc);
        let opcode_info = &OPCODE_TABLE[opcode as usize];

        // Move PC past the opcode
        self.pc = self.pc.wrapping_add(1);

        // Calculate effective address based on addressing mode
        let addr_result = match opcode_info.mode {
            AddressingMode::Implied => self.addr_implied(),
            AddressingMode::Accumulator => self.addr_accumulator(),
            AddressingMode::Immediate => self.addr_immediate(bus),
            AddressingMode::ZeroPage => self.addr_zero_page(bus),
            AddressingMode::ZeroPageX => self.addr_zero_page_x(bus),
            AddressingMode::ZeroPageY => self.addr_zero_page_y(bus),
            AddressingMode::Relative => self.addr_relative(bus),
            AddressingMode::Absolute => self.addr_absolute(bus),
            AddressingMode::AbsoluteX => self.addr_absolute_x(bus),
            AddressingMode::AbsoluteY => self.addr_absolute_y(bus),
            AddressingMode::Indirect => self.addr_indirect(bus),
            AddressingMode::IndexedIndirect => self.addr_indexed_indirect(bus),
            AddressingMode::IndirectIndexed => self.addr_indirect_indexed(bus),
        };

        // Execute the instruction (may return extra cycles for branches)
        let extra_cycles = self.execute_instruction(opcode, &addr_result, bus);

        // Calculate actual cycles (base + page crossing penalty + branch cycles)
        let mut cycles = opcode_info.cycles;
        if opcode_info.page_cycle
            && addr_result.page_crossed
            && opcode_info.mode != AddressingMode::Relative
        {
            cycles += 1;
        }
        cycles += extra_cycles;

        // Update total cycle counter
        self.cycles = self.cycles.wrapping_add(cycles as u64);

        cycles
    }

    /// Execute a specific instruction based on its opcode
    /// Returns the number of extra cycles consumed (used by branch instructions)
    #[allow(clippy::too_many_lines)]
    fn execute_instruction(
        &mut self,
        opcode: u8,
        addr_result: &crate::cpu::addressing::AddressingResult,
        bus: &mut Bus,
    ) -> u8 {
        match opcode {
            // Load/Store instructions
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(bus, addr_result),
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(bus, addr_result),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(bus, addr_result),
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(bus, addr_result),
            0x86 | 0x96 | 0x8E => self.stx(bus, addr_result),
            0x84 | 0x94 | 0x8C => self.sty(bus, addr_result),

            // Arithmetic instructions
            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.adc(bus, addr_result),
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.sbc(bus, addr_result),
            0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(bus, addr_result),
            0xE8 => self.inx(),
            0xC8 => self.iny(),
            0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(bus, addr_result),
            0xCA => self.dex(),
            0x88 => self.dey(),

            // Logical instructions
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(bus, addr_result),
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ora(bus, addr_result),
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.eor(bus, addr_result),
            0x24 | 0x2C => self.bit(bus, addr_result),

            // Shift/Rotate instructions
            0x0A => self.asl(bus, addr_result, true),
            0x06 | 0x16 | 0x0E | 0x1E => self.asl(bus, addr_result, false),
            0x4A => self.lsr(bus, addr_result, true),
            0x46 | 0x56 | 0x4E | 0x5E => self.lsr(bus, addr_result, false),
            0x2A => self.rol(bus, addr_result, true),
            0x26 | 0x36 | 0x2E | 0x3E => self.rol(bus, addr_result, false),
            0x6A => self.ror(bus, addr_result, true),
            0x66 | 0x76 | 0x6E | 0x7E => self.ror(bus, addr_result, false),

            // Compare instructions
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => self.cmp(bus, addr_result),
            0xE0 | 0xE4 | 0xEC => self.cpx(bus, addr_result),
            0xC0 | 0xC4 | 0xCC => self.cpy(bus, addr_result),

            // Branch instructions (return extra cycles)
            0x90 => return self.bcc(bus, addr_result),
            0xB0 => return self.bcs(bus, addr_result),
            0xF0 => return self.beq(bus, addr_result),
            0x30 => return self.bmi(bus, addr_result),
            0xD0 => return self.bne(bus, addr_result),
            0x10 => return self.bpl(bus, addr_result),
            0x50 => return self.bvc(bus, addr_result),
            0x70 => return self.bvs(bus, addr_result),

            // Jump/Subroutine instructions
            0x4C | 0x6C => return self.jmp(bus, addr_result),
            0x20 => return self.jsr(bus, addr_result),
            0x60 => return self.rts(bus, addr_result),

            // Stack instructions
            0x48 => return self.pha(bus, addr_result),
            0x68 => return self.pla(bus, addr_result),
            0x08 => return self.php(bus, addr_result),
            0x28 => return self.plp(bus, addr_result),
            0x9A => self.txs(),
            0xBA => self.tsx(),

            // Transfer instructions
            0xAA => self.tax(),
            0xA8 => self.tay(),
            0x8A => self.txa(),
            0x98 => self.tya(),

            // Flag instructions
            0x18 => return self.clc(bus, addr_result),
            0xD8 => return self.cld(bus, addr_result),
            0x58 => return self.cli(bus, addr_result),
            0xB8 => return self.clv(bus, addr_result),
            0x38 => return self.sec(bus, addr_result),
            0xF8 => return self.sed(bus, addr_result),
            0x78 => return self.sei(bus, addr_result),

            // Miscellaneous instructions
            0x00 => return self.brk(bus, addr_result),
            0x40 => return self.rti(bus, addr_result),
            0xEA => return self.nop(bus, addr_result),

            // Unofficial/unimplemented opcodes - treat as NOP for now
            _ => {
                // For unimplemented opcodes, just do nothing
                // In a real emulator, you might want to log a warning here
            }
        }
        0 // No extra cycles for non-branch instructions
    }

    /// Generate a trace log line in Nestest format
    ///
    /// Format: PC  OP OP OP  MNEMONIC $ADDR    A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXXX
    /// Example: C000  4C F5 C5  JMP $C5F5       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7
    pub fn trace(&self, bus: &mut Bus) -> String {
        let pc = self.pc;
        let opcode = bus.read(pc);
        let opcode_info = &OPCODE_TABLE[opcode as usize];

        // Read instruction bytes (opcode + operands)
        let byte1 = opcode;
        let byte2 = if opcode_info.bytes >= 2 {
            bus.read(pc.wrapping_add(1))
        } else {
            0
        };
        let byte3 = if opcode_info.bytes >= 3 {
            bus.read(pc.wrapping_add(2))
        } else {
            0
        };

        // Format the hex bytes with proper spacing (9 characters total)
        let hex_bytes = match opcode_info.bytes {
            1 => format!("{:02X}      ", byte1),
            2 => format!("{:02X} {:02X}   ", byte1, byte2),
            3 => format!("{:02X} {:02X} {:02X}", byte1, byte2, byte3),
            _ => format!("{:02X}      ", byte1),
        };

        // Disassemble the instruction with operand
        let disassembly = self.disassemble_instruction(pc, bus, opcode_info, byte2, byte3);

        // Format the trace line (pad disassembly to 32 characters from start)
        // The format is: "XXXX  HH HH HH  " (16 chars) + disassembly (padded to 48 chars total)
        format!(
            "{:04X}  {}  {:<32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            pc, hex_bytes, disassembly, self.a, self.x, self.y, self.status, self.sp, self.cycles
        )
    }

    /// Disassemble an instruction into human-readable format
    #[allow(clippy::too_many_lines)]
    fn disassemble_instruction(
        &self,
        pc: u16,
        bus: &mut Bus,
        opcode_info: &crate::cpu::opcodes::OpcodeInfo,
        byte2: u8,
        byte3: u8,
    ) -> String {
        let mnemonic = opcode_info.mnemonic;

        match opcode_info.mode {
            AddressingMode::Implied | AddressingMode::Accumulator => {
                if opcode_info.mode == AddressingMode::Accumulator {
                    format!("{} A", mnemonic)
                } else {
                    mnemonic.to_string()
                }
            }
            AddressingMode::Immediate => {
                format!("{} #${:02X}", mnemonic, byte2)
            }
            AddressingMode::ZeroPage => {
                let addr = byte2;
                let value = bus.read(addr as u16);
                format!("{} ${:02X} = {:02X}", mnemonic, addr, value)
            }
            AddressingMode::ZeroPageX => {
                let base = byte2;
                let addr = base.wrapping_add(self.x);
                let value = bus.read(addr as u16);
                format!(
                    "{} ${:02X},X @ {:02X} = {:02X}",
                    mnemonic, base, addr, value
                )
            }
            AddressingMode::ZeroPageY => {
                let base = byte2;
                let addr = base.wrapping_add(self.y);
                let value = bus.read(addr as u16);
                format!(
                    "{} ${:02X},Y @ {:02X} = {:02X}",
                    mnemonic, base, addr, value
                )
            }
            AddressingMode::Relative => {
                let offset = byte2 as i8;
                let target = if offset >= 0 {
                    pc.wrapping_add(2).wrapping_add(offset as u16)
                } else {
                    pc.wrapping_add(2).wrapping_sub((-offset) as u16)
                };
                format!("{} ${:04X}", mnemonic, target)
            }
            AddressingMode::Absolute => {
                let addr = u16::from_le_bytes([byte2, byte3]);
                if mnemonic == "JMP" || mnemonic == "JSR" {
                    format!("{} ${:04X}", mnemonic, addr)
                } else {
                    let value = bus.read(addr);
                    format!("{} ${:04X} = {:02X}", mnemonic, addr, value)
                }
            }
            AddressingMode::AbsoluteX => {
                let base = u16::from_le_bytes([byte2, byte3]);
                let addr = base.wrapping_add(self.x as u16);
                let value = bus.read(addr);
                format!(
                    "{} ${:04X},X @ {:04X} = {:02X}",
                    mnemonic, base, addr, value
                )
            }
            AddressingMode::AbsoluteY => {
                let base = u16::from_le_bytes([byte2, byte3]);
                let addr = base.wrapping_add(self.y as u16);
                let value = bus.read(addr);
                format!(
                    "{} ${:04X},Y @ {:04X} = {:02X}",
                    mnemonic, base, addr, value
                )
            }
            AddressingMode::Indirect => {
                let ptr = u16::from_le_bytes([byte2, byte3]);
                let lo = bus.read(ptr);
                let hi_addr = if ptr & 0x00FF == 0x00FF {
                    ptr & 0xFF00
                } else {
                    ptr + 1
                };
                let hi = bus.read(hi_addr);
                let target = u16::from_le_bytes([lo, hi]);
                format!("{} (${:04X}) = {:04X}", mnemonic, ptr, target)
            }
            AddressingMode::IndexedIndirect => {
                let base = byte2;
                let ptr = base.wrapping_add(self.x);
                let lo = bus.read(ptr as u16);
                let hi = bus.read(ptr.wrapping_add(1) as u16);
                let addr = u16::from_le_bytes([lo, hi]);
                let value = bus.read(addr);
                format!(
                    "{} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    mnemonic, base, ptr, addr, value
                )
            }
            AddressingMode::IndirectIndexed => {
                let ptr = byte2;
                let lo = bus.read(ptr as u16);
                let hi = bus.read(ptr.wrapping_add(1) as u16);
                let base = u16::from_le_bytes([lo, hi]);
                let addr = base.wrapping_add(self.y as u16);
                let value = bus.read(addr);
                format!(
                    "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    mnemonic, ptr, base, addr, value
                )
            }
        }
    }
}
