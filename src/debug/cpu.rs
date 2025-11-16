// CPU Debugger - Debug information for the 6502 CPU
//
// Provides:
// - Register dump
// - Stack dump
// - Current instruction disassembly
// - CPU state capture

use super::disassembler::{disassemble_instruction, DisassembledInstruction};
use crate::bus::Bus;
use crate::cpu::Cpu;

/// CPU state snapshot
///
/// Contains a complete snapshot of the CPU state at a specific point in time.
/// Used for debugging and logging.
#[derive(Debug, Clone)]
pub struct CpuState {
    /// Program Counter
    pub pc: u16,

    /// Accumulator
    pub a: u8,

    /// X register
    pub x: u8,

    /// Y register
    pub y: u8,

    /// Stack Pointer
    pub sp: u8,

    /// Status flags
    pub status: u8,

    /// Cycle count
    pub cycles: u64,

    /// Current instruction (disassembled)
    pub instruction: DisassembledInstruction,

    /// Stack contents (top 16 bytes)
    pub stack: Vec<u8>,
}

impl CpuState {
    /// Format the status flags as a string
    ///
    /// # Returns
    ///
    /// A string representation of the status flags (e.g., "NV-BDIZC")
    pub fn format_status(&self) -> String {
        let mut result = String::with_capacity(8);

        result.push(if self.status & 0x80 != 0 { 'N' } else { 'n' });
        result.push(if self.status & 0x40 != 0 { 'V' } else { 'v' });
        result.push('-'); // Unused flag (always 1)
        result.push(if self.status & 0x10 != 0 { 'B' } else { 'b' });
        result.push(if self.status & 0x08 != 0 { 'D' } else { 'd' });
        result.push(if self.status & 0x04 != 0 { 'I' } else { 'i' });
        result.push(if self.status & 0x02 != 0 { 'Z' } else { 'z' });
        result.push(if self.status & 0x01 != 0 { 'C' } else { 'c' });

        result
    }

    /// Format the register state as a string
    ///
    /// # Returns
    ///
    /// A string representation of all registers
    pub fn format_registers(&self) -> String {
        format!(
            "PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} SP:{:02X} P:{:02X} [{}]",
            self.pc,
            self.a,
            self.x,
            self.y,
            self.sp,
            self.status,
            self.format_status()
        )
    }

    /// Format the stack contents as a string
    ///
    /// # Returns
    ///
    /// A hex dump of the stack
    pub fn format_stack(&self) -> String {
        let mut result = String::new();
        result.push_str("Stack: ");

        for (i, byte) in self.stack.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(&format!("{:02X}", byte));
        }

        result
    }
}

impl std::fmt::Display for CpuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | Cycles: {}",
            self.format_registers(),
            self.instruction,
            self.cycles
        )
    }
}

/// CPU Debugger
///
/// Provides debugging functionality for the 6502 CPU including:
/// - State capture
/// - Register dumps
/// - Stack inspection
pub struct CpuDebugger {
    /// Whether to capture full state (including stack)
    capture_stack: bool,
}

impl CpuDebugger {
    /// Create a new CPU debugger
    ///
    /// # Returns
    ///
    /// A new CPU debugger instance
    pub fn new() -> Self {
        CpuDebugger {
            capture_stack: true,
        }
    }

    /// Enable/disable stack capture
    ///
    /// Disabling stack capture can improve performance when logging
    /// many CPU states.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to capture stack contents
    pub fn set_capture_stack(&mut self, enabled: bool) {
        self.capture_stack = enabled;
    }

    /// Capture the current CPU state
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// A snapshot of the CPU state
    pub fn capture_state(&self, cpu: &Cpu, bus: &mut Bus) -> CpuState {
        // Disassemble the current instruction
        let instruction = disassemble_instruction(cpu.pc, bus);

        // Capture stack contents (top 16 bytes)
        let stack = if self.capture_stack {
            self.capture_stack_contents(cpu, bus)
        } else {
            Vec::new()
        };

        CpuState {
            pc: cpu.pc,
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            sp: cpu.sp,
            status: cpu.status,
            cycles: cpu.cycles,
            instruction,
            stack,
        }
    }

    /// Capture stack contents
    ///
    /// Reads the top 16 bytes from the stack.
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// A vector containing stack contents
    fn capture_stack_contents(&self, cpu: &Cpu, bus: &mut Bus) -> Vec<u8> {
        let mut stack = Vec::new();
        let stack_top = cpu.sp;

        // Read up to 16 bytes from the stack
        for i in 0..16 {
            let addr = 0x0100 | ((stack_top.wrapping_add(i + 1)) as u16);
            if addr <= 0x01FF {
                stack.push(bus.read(addr));
            } else {
                break;
            }
        }

        stack
    }

    /// Format register dump
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    ///
    /// # Returns
    ///
    /// A formatted string showing all CPU registers
    pub fn dump_registers(&self, cpu: &Cpu) -> String {
        let mut output = String::new();

        output.push_str(&format!("PC: ${:04X}\n", cpu.pc));
        output.push_str(&format!("A:  ${:02X} ({})\n", cpu.a, cpu.a));
        output.push_str(&format!("X:  ${:02X} ({})\n", cpu.x, cpu.x));
        output.push_str(&format!("Y:  ${:02X} ({})\n", cpu.y, cpu.y));
        output.push_str(&format!("SP: ${:02X}\n", cpu.sp));
        output.push_str(&format!("P:  ${:02X} [", cpu.status));

        // Format flags
        output.push(if cpu.status & 0x80 != 0 { 'N' } else { 'n' });
        output.push(if cpu.status & 0x40 != 0 { 'V' } else { 'v' });
        output.push('-');
        output.push(if cpu.status & 0x10 != 0 { 'B' } else { 'b' });
        output.push(if cpu.status & 0x08 != 0 { 'D' } else { 'd' });
        output.push(if cpu.status & 0x04 != 0 { 'I' } else { 'i' });
        output.push(if cpu.status & 0x02 != 0 { 'Z' } else { 'z' });
        output.push(if cpu.status & 0x01 != 0 { 'C' } else { 'c' });
        output.push_str("]\n");

        output.push_str(&format!("Cycles: {}\n", cpu.cycles));

        output
    }

    /// Format stack dump
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// A hex dump of the stack contents
    pub fn dump_stack(&self, cpu: &Cpu, bus: &mut Bus) -> String {
        let mut output = String::new();

        output.push_str(&format!("Stack (SP = ${:02X}):\n", cpu.sp));

        // Show 64 bytes of stack memory
        for row in 0..4 {
            let base_addr = 0x0100 + (row * 16);
            output.push_str(&format!("  ${:04X}: ", base_addr));

            for col in 0..16 {
                let addr = base_addr + col;
                let value = bus.read(addr);

                // Highlight the current stack pointer position
                if (addr & 0xFF) == cpu.sp as u16 {
                    output.push_str(&format!("[{:02X}] ", value));
                } else {
                    output.push_str(&format!("{:02X} ", value));
                }
            }

            output.push('\n');
        }

        output
    }
}

impl Default for CpuDebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_debugger_creation() {
        let debugger = CpuDebugger::new();
        assert!(debugger.capture_stack);
    }

    #[test]
    fn test_capture_stack_toggle() {
        let mut debugger = CpuDebugger::new();

        debugger.set_capture_stack(false);
        assert!(!debugger.capture_stack);

        debugger.set_capture_stack(true);
        assert!(debugger.capture_stack);
    }

    #[test]
    fn test_cpu_state_format_status() {
        let state = CpuState {
            pc: 0x8000,
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            status: 0b11000011, // N, V, C, Z set
            cycles: 0,
            instruction: DisassembledInstruction {
                address: 0x8000,
                opcode: 0xEA,
                mnemonic: "NOP".to_string(),
                addressing_mode: "Implied".to_string(),
                operands: Vec::new(),
                length: 1,
            },
            stack: Vec::new(),
        };

        let formatted = state.format_status();
        assert_eq!(formatted, "NV-bdiZC");
    }

    #[test]
    fn test_cpu_state_format_registers() {
        let state = CpuState {
            pc: 0x8000,
            a: 0x42,
            x: 0x10,
            y: 0x20,
            sp: 0xFD,
            status: 0x24,
            cycles: 7,
            instruction: DisassembledInstruction {
                address: 0x8000,
                opcode: 0xEA,
                mnemonic: "NOP".to_string(),
                addressing_mode: "Implied".to_string(),
                operands: Vec::new(),
                length: 1,
            },
            stack: Vec::new(),
        };

        let formatted = state.format_registers();
        assert!(formatted.contains("PC:8000"));
        assert!(formatted.contains("A:42"));
        assert!(formatted.contains("X:10"));
        assert!(formatted.contains("Y:20"));
        assert!(formatted.contains("SP:FD"));
    }
}
