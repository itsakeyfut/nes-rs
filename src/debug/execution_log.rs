// Execution Log - Track instruction execution, memory access, and PPU events
//
// Provides:
// - Instruction trace logging
// - Memory access tracking (reads and writes)
// - PPU event logging (VBlank, NMI, register changes, sprite 0 hit)
// - Circular buffer with configurable size
// - Search and filter functionality

use super::disassembler::DisassembledInstruction;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// PPU event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PpuEventType {
    /// VBlank started
    VBlankStart { frame: u64 },
    /// VBlank ended
    VBlankEnd { frame: u64 },
    /// NMI triggered
    NmiTriggered { cycle: u64 },
    /// PPUCTRL register changed
    PpuCtrlChange { old: u8, new: u8 },
    /// PPUMASK register changed
    PpuMaskChange { old: u8, new: u8 },
    /// Sprite 0 hit occurred
    Sprite0Hit { scanline: u16, cycle: u16 },
    /// Scanline milestone reached
    ScanlineMilestone { scanline: u16 },
}

impl std::fmt::Display for PpuEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PpuEventType::VBlankStart { frame } => write!(f, "VBlank Start (Frame {})", frame),
            PpuEventType::VBlankEnd { frame } => write!(f, "VBlank End (Frame {})", frame),
            PpuEventType::NmiTriggered { cycle } => write!(f, "NMI Triggered (Cycle {})", cycle),
            PpuEventType::PpuCtrlChange { old, new } => {
                write!(f, "PPUCTRL: ${:02X} -> ${:02X}", old, new)
            }
            PpuEventType::PpuMaskChange { old, new } => {
                write!(f, "PPUMASK: ${:02X} -> ${:02X}", old, new)
            }
            PpuEventType::Sprite0Hit { scanline, cycle } => {
                write!(f, "Sprite 0 Hit (SL:{}, CY:{})", scanline, cycle)
            }
            PpuEventType::ScanlineMilestone { scanline } => {
                write!(f, "Scanline {}", scanline)
            }
        }
    }
}

/// Execution log entry
#[derive(Debug, Clone)]
pub enum ExecutionLogEntry {
    /// CPU instruction execution
    Instruction {
        cycle: u64,
        pc: u16,
        instruction: DisassembledInstruction,
        a: u8,
        x: u8,
        y: u8,
        p: u8,
        sp: u8,
    },
    /// Memory read
    MemoryRead {
        cycle: u64,
        address: u16,
        value: u8,
        pc: u16,
    },
    /// Memory write
    MemoryWrite {
        cycle: u64,
        address: u16,
        value: u8,
        pc: u16,
    },
    /// PPU event
    PpuEvent { cycle: u64, event: PpuEventType },
}

impl ExecutionLogEntry {
    /// Get the cycle count for this entry
    pub fn cycle(&self) -> u64 {
        match self {
            ExecutionLogEntry::Instruction { cycle, .. } => *cycle,
            ExecutionLogEntry::MemoryRead { cycle, .. } => *cycle,
            ExecutionLogEntry::MemoryWrite { cycle, .. } => *cycle,
            ExecutionLogEntry::PpuEvent { cycle, .. } => *cycle,
        }
    }

    /// Check if this entry matches the search query
    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();

        match self {
            ExecutionLogEntry::Instruction { instruction, .. } => {
                instruction.mnemonic.to_lowercase().contains(&query_lower)
                    || instruction
                        .format_assembly()
                        .to_lowercase()
                        .contains(&query_lower)
            }
            ExecutionLogEntry::MemoryRead { address, .. } => {
                format!("{:04X}", address).contains(&query_lower)
            }
            ExecutionLogEntry::MemoryWrite { address, .. } => {
                format!("{:04X}", address).contains(&query_lower)
            }
            ExecutionLogEntry::PpuEvent { event, .. } => {
                format!("{}", event).to_lowercase().contains(&query_lower)
            }
        }
    }
}

impl std::fmt::Display for ExecutionLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionLogEntry::Instruction {
                cycle,
                pc,
                instruction,
                a,
                x,
                y,
                p,
                sp,
            } => {
                write!(
                    f,
                    "[{:08}] ${:04X}: {:20} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                    cycle,
                    pc,
                    instruction.format_assembly(),
                    a,
                    x,
                    y,
                    p,
                    sp
                )
            }
            ExecutionLogEntry::MemoryRead {
                cycle,
                address,
                value,
                pc,
            } => {
                write!(
                    f,
                    "[{:08}] MEM READ  ${:04X} = ${:02X} (PC: ${:04X})",
                    cycle, address, value, pc
                )
            }
            ExecutionLogEntry::MemoryWrite {
                cycle,
                address,
                value,
                pc,
            } => {
                write!(
                    f,
                    "[{:08}] MEM WRITE ${:04X} = ${:02X} (PC: ${:04X})",
                    cycle, address, value, pc
                )
            }
            ExecutionLogEntry::PpuEvent { cycle, event } => {
                write!(f, "[{:08}] PPU: {}", cycle, event)
            }
        }
    }
}

/// Filter settings for execution log
#[derive(Debug, Clone, Copy)]
pub struct LogFilter {
    /// Show instruction execution
    pub show_instructions: bool,
    /// Show memory reads
    pub show_memory_reads: bool,
    /// Show memory writes
    pub show_memory_writes: bool,
    /// Show PPU events
    pub show_ppu_events: bool,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            show_instructions: true,
            show_memory_reads: false,
            show_memory_writes: false,
            show_ppu_events: false,
        }
    }
}

impl LogFilter {
    /// Check if an entry passes the filter
    pub fn passes(&self, entry: &ExecutionLogEntry) -> bool {
        match entry {
            ExecutionLogEntry::Instruction { .. } => self.show_instructions,
            ExecutionLogEntry::MemoryRead { .. } => self.show_memory_reads,
            ExecutionLogEntry::MemoryWrite { .. } => self.show_memory_writes,
            ExecutionLogEntry::PpuEvent { .. } => self.show_ppu_events,
        }
    }
}

/// Execution log
///
/// Tracks instruction execution, memory access, and PPU events in a circular buffer.
pub struct ExecutionLog {
    /// Log entries (circular buffer)
    entries: VecDeque<ExecutionLogEntry>,
    /// Maximum number of entries (0 = unlimited)
    max_entries: usize,
    /// Enable instruction logging
    log_instructions: bool,
    /// Enable memory read logging
    log_memory_reads: bool,
    /// Enable memory write logging
    log_memory_writes: bool,
    /// Enable PPU event logging
    log_ppu_events: bool,
    /// Memory address filter (None = all addresses)
    memory_filter_start: Option<u16>,
    memory_filter_end: Option<u16>,
}

impl ExecutionLog {
    /// Create a new execution log
    ///
    /// # Returns
    ///
    /// A new execution log with default settings
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 10000,
            log_instructions: true,
            log_memory_reads: false,
            log_memory_writes: false,
            log_ppu_events: false,
            memory_filter_start: None,
            memory_filter_end: None,
        }
    }

    /// Set maximum number of entries
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum number of entries (0 = unlimited)
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        self.trim_if_needed();
    }

    /// Get maximum number of entries
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Enable instruction logging
    pub fn enable_instruction_logging(&mut self) {
        self.log_instructions = true;
    }

    /// Disable instruction logging
    pub fn disable_instruction_logging(&mut self) {
        self.log_instructions = false;
    }

    /// Check if instruction logging is enabled
    pub fn is_instruction_logging_enabled(&self) -> bool {
        self.log_instructions
    }

    /// Enable memory read logging
    pub fn enable_memory_read_logging(&mut self) {
        self.log_memory_reads = true;
    }

    /// Disable memory read logging
    pub fn disable_memory_read_logging(&mut self) {
        self.log_memory_reads = false;
    }

    /// Check if memory read logging is enabled
    pub fn is_memory_read_logging_enabled(&self) -> bool {
        self.log_memory_reads
    }

    /// Enable memory write logging
    pub fn enable_memory_write_logging(&mut self) {
        self.log_memory_writes = true;
    }

    /// Disable memory write logging
    pub fn disable_memory_write_logging(&mut self) {
        self.log_memory_writes = false;
    }

    /// Check if memory write logging is enabled
    pub fn is_memory_write_logging_enabled(&self) -> bool {
        self.log_memory_writes
    }

    /// Enable PPU event logging
    pub fn enable_ppu_event_logging(&mut self) {
        self.log_ppu_events = true;
    }

    /// Disable PPU event logging
    pub fn disable_ppu_event_logging(&mut self) {
        self.log_ppu_events = false;
    }

    /// Check if PPU event logging is enabled
    pub fn is_ppu_event_logging_enabled(&self) -> bool {
        self.log_ppu_events
    }

    /// Set memory address filter
    ///
    /// # Arguments
    ///
    /// * `start` - Start address (inclusive)
    /// * `end` - End address (inclusive)
    pub fn set_memory_filter(&mut self, start: u16, end: u16) {
        self.memory_filter_start = Some(start);
        self.memory_filter_end = Some(end);
    }

    /// Clear memory address filter
    pub fn clear_memory_filter(&mut self) {
        self.memory_filter_start = None;
        self.memory_filter_end = None;
    }

    /// Check if an address passes the memory filter
    fn passes_memory_filter(&self, address: u16) -> bool {
        match (self.memory_filter_start, self.memory_filter_end) {
            (Some(start), Some(end)) => address >= start && address <= end,
            _ => true,
        }
    }

    /// Log an instruction execution
    ///
    /// # Arguments
    ///
    /// * `cycle` - Current CPU cycle count
    /// * `pc` - Program counter
    /// * `instruction` - Disassembled instruction
    /// * `a` - Accumulator
    /// * `x` - X register
    /// * `y` - Y register
    /// * `p` - Processor status
    /// * `sp` - Stack pointer
    #[allow(clippy::too_many_arguments)]
    pub fn log_instruction(
        &mut self,
        cycle: u64,
        pc: u16,
        instruction: DisassembledInstruction,
        a: u8,
        x: u8,
        y: u8,
        p: u8,
        sp: u8,
    ) {
        if !self.log_instructions {
            return;
        }

        let entry = ExecutionLogEntry::Instruction {
            cycle,
            pc,
            instruction,
            a,
            x,
            y,
            p,
            sp,
        };
        self.add_entry(entry);
    }

    /// Log a memory read
    ///
    /// # Arguments
    ///
    /// * `cycle` - Current CPU cycle count
    /// * `address` - Memory address
    /// * `value` - Value read
    /// * `pc` - Program counter
    pub fn log_memory_read(&mut self, cycle: u64, address: u16, value: u8, pc: u16) {
        if !self.log_memory_reads || !self.passes_memory_filter(address) {
            return;
        }

        let entry = ExecutionLogEntry::MemoryRead {
            cycle,
            address,
            value,
            pc,
        };
        self.add_entry(entry);
    }

    /// Log a memory write
    ///
    /// # Arguments
    ///
    /// * `cycle` - Current CPU cycle count
    /// * `address` - Memory address
    /// * `value` - Value written
    /// * `pc` - Program counter
    pub fn log_memory_write(&mut self, cycle: u64, address: u16, value: u8, pc: u16) {
        if !self.log_memory_writes || !self.passes_memory_filter(address) {
            return;
        }

        let entry = ExecutionLogEntry::MemoryWrite {
            cycle,
            address,
            value,
            pc,
        };
        self.add_entry(entry);
    }

    /// Log a PPU event
    ///
    /// # Arguments
    ///
    /// * `cycle` - Current CPU cycle count
    /// * `event` - PPU event type
    pub fn log_ppu_event(&mut self, cycle: u64, event: PpuEventType) {
        if !self.log_ppu_events {
            return;
        }

        let entry = ExecutionLogEntry::PpuEvent { cycle, event };
        self.add_entry(entry);
    }

    /// Add an entry to the log
    fn add_entry(&mut self, entry: ExecutionLogEntry) {
        self.entries.push_back(entry);
        self.trim_if_needed();
    }

    /// Trim the log if it exceeds max size
    fn trim_if_needed(&mut self) {
        if self.max_entries > 0 {
            while self.entries.len() > self.max_entries {
                self.entries.pop_front();
            }
        }
    }

    /// Get all entries
    ///
    /// # Returns
    ///
    /// A slice of all log entries
    pub fn entries(&self) -> &VecDeque<ExecutionLogEntry> {
        &self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export the log to a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    /// * `filter` - Optional filter to apply
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err` otherwise
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        filter: Option<&LogFilter>,
    ) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        for entry in &self.entries {
            if let Some(filter) = filter {
                if !filter.passes(entry) {
                    continue;
                }
            }

            writeln!(file, "{}", entry)?;
        }

        Ok(())
    }

    /// Get filtered entries matching a search query
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `filter` - Filter settings
    ///
    /// # Returns
    ///
    /// A vector of entries matching the query and filter
    pub fn get_filtered_entries(&self, query: &str, filter: &LogFilter) -> Vec<&ExecutionLogEntry> {
        self.entries
            .iter()
            .filter(|entry| filter.passes(entry) && entry.matches_search(query))
            .collect()
    }
}

impl Default for ExecutionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::debug::disassembler::disassemble_instruction;

    #[test]
    fn test_execution_log_creation() {
        let log = ExecutionLog::new();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
        assert_eq!(log.max_entries(), 10000);
    }

    #[test]
    fn test_log_instruction() {
        let mut log = ExecutionLog::new();
        let mut bus = Bus::new();

        bus.write(0x8000, 0xEA); // NOP
        let instruction = disassemble_instruction(0x8000, &mut bus);

        log.log_instruction(100, 0x8000, instruction, 0x00, 0x00, 0x00, 0x24, 0xFD);

        assert_eq!(log.len(), 1);
        match log.entries().front().unwrap() {
            ExecutionLogEntry::Instruction { cycle, pc, .. } => {
                assert_eq!(*cycle, 100);
                assert_eq!(*pc, 0x8000);
            }
            _ => panic!("Expected Instruction entry"),
        }
    }

    #[test]
    fn test_log_memory_read() {
        let mut log = ExecutionLog::new();
        log.enable_memory_read_logging();

        log.log_memory_read(100, 0x2002, 0x80, 0x8000);

        assert_eq!(log.len(), 1);
        match log.entries().front().unwrap() {
            ExecutionLogEntry::MemoryRead {
                cycle,
                address,
                value,
                pc,
            } => {
                assert_eq!(*cycle, 100);
                assert_eq!(*address, 0x2002);
                assert_eq!(*value, 0x80);
                assert_eq!(*pc, 0x8000);
            }
            _ => panic!("Expected MemoryRead entry"),
        }
    }

    #[test]
    fn test_log_memory_write() {
        let mut log = ExecutionLog::new();
        log.enable_memory_write_logging();

        log.log_memory_write(100, 0x2000, 0x42, 0x8000);

        assert_eq!(log.len(), 1);
        match log.entries().front().unwrap() {
            ExecutionLogEntry::MemoryWrite {
                cycle,
                address,
                value,
                pc,
            } => {
                assert_eq!(*cycle, 100);
                assert_eq!(*address, 0x2000);
                assert_eq!(*value, 0x42);
                assert_eq!(*pc, 0x8000);
            }
            _ => panic!("Expected MemoryWrite entry"),
        }
    }

    #[test]
    fn test_log_ppu_event() {
        let mut log = ExecutionLog::new();
        log.enable_ppu_event_logging();

        log.log_ppu_event(100, PpuEventType::VBlankStart { frame: 0 });

        assert_eq!(log.len(), 1);
        match log.entries().front().unwrap() {
            ExecutionLogEntry::PpuEvent { cycle, event } => {
                assert_eq!(*cycle, 100);
                assert_eq!(*event, PpuEventType::VBlankStart { frame: 0 });
            }
            _ => panic!("Expected PpuEvent entry"),
        }
    }

    #[test]
    fn test_max_entries() {
        let mut log = ExecutionLog::new();
        log.set_max_entries(3);
        log.enable_ppu_event_logging();

        log.log_ppu_event(1, PpuEventType::VBlankStart { frame: 0 });
        log.log_ppu_event(2, PpuEventType::VBlankStart { frame: 1 });
        log.log_ppu_event(3, PpuEventType::VBlankStart { frame: 2 });
        log.log_ppu_event(4, PpuEventType::VBlankStart { frame: 3 });

        assert_eq!(log.len(), 3);

        // First entry should be removed
        let first = log.entries().front().unwrap();
        assert_eq!(first.cycle(), 2);
    }

    #[test]
    fn test_clear() {
        let mut log = ExecutionLog::new();
        log.enable_ppu_event_logging();

        log.log_ppu_event(100, PpuEventType::VBlankStart { frame: 0 });
        assert_eq!(log.len(), 1);

        log.clear();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_memory_filter() {
        let mut log = ExecutionLog::new();
        log.enable_memory_read_logging();
        log.set_memory_filter(0x2000, 0x2007);

        // Should be logged (within filter range)
        log.log_memory_read(100, 0x2002, 0x80, 0x8000);

        // Should not be logged (outside filter range)
        log.log_memory_read(101, 0x0800, 0x00, 0x8001);

        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_log_filter() {
        let filter = LogFilter {
            show_instructions: true,
            show_memory_reads: false,
            ..Default::default()
        };

        let mut bus = Bus::new();
        bus.write(0x8000, 0xEA); // NOP
        let instruction = disassemble_instruction(0x8000, &mut bus);

        let instr_entry = ExecutionLogEntry::Instruction {
            cycle: 100,
            pc: 0x8000,
            instruction,
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            sp: 0,
        };

        let mem_entry = ExecutionLogEntry::MemoryRead {
            cycle: 101,
            address: 0x2002,
            value: 0x80,
            pc: 0x8001,
        };

        assert!(filter.passes(&instr_entry));
        assert!(!filter.passes(&mem_entry));
    }

    #[test]
    fn test_search() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xA9); // LDA #$42
        bus.write(0x8001, 0x42);
        let instruction = disassemble_instruction(0x8000, &mut bus);

        let entry = ExecutionLogEntry::Instruction {
            cycle: 100,
            pc: 0x8000,
            instruction,
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            sp: 0,
        };

        assert!(entry.matches_search("LDA"));
        assert!(entry.matches_search("lda"));
        assert!(entry.matches_search("#$42"));
        assert!(!entry.matches_search("STA"));
    }

    #[test]
    fn test_ppu_event_display() {
        let event = PpuEventType::VBlankStart { frame: 5 };
        assert_eq!(format!("{}", event), "VBlank Start (Frame 5)");

        let event = PpuEventType::NmiTriggered { cycle: 12345 };
        assert_eq!(format!("{}", event), "NMI Triggered (Cycle 12345)");

        let event = PpuEventType::PpuCtrlChange {
            old: 0x80,
            new: 0x90,
        };
        assert_eq!(format!("{}", event), "PPUCTRL: $80 -> $90");
    }

    #[test]
    fn test_entry_display_format() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xEA); // NOP
        let instruction = disassemble_instruction(0x8000, &mut bus);

        let entry = ExecutionLogEntry::Instruction {
            cycle: 123,
            pc: 0x8000,
            instruction,
            a: 0x42,
            x: 0x10,
            y: 0x20,
            p: 0x24,
            sp: 0xFD,
        };

        let display = format!("{}", entry);
        assert!(display.contains("[00000123]"));
        assert!(display.contains("$8000"));
        assert!(display.contains("NOP"));
        assert!(display.contains("A:42"));
        assert!(display.contains("X:10"));
        assert!(display.contains("Y:20"));
        assert!(display.contains("P:24"));
        assert!(display.contains("SP:FD"));
    }
}
