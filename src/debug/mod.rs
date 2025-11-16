// Debug module - Debugging tools for NES emulator
//
// This module provides debugging capabilities including:
// - CPU debugger (step execution, breakpoints, register dump, disassembly)
// - Memory viewer (CPU and PPU memory space, hex dump)
// - PPU debugger (nametable, pattern table, palette, OAM viewers)
// - Logging (CPU trace, PPU trace, configurable log levels)
//
// All debugging features are optional and designed to have minimal
// performance impact when disabled.

pub mod cpu;
pub mod disassembler;
pub mod logger;
pub mod memory;
pub mod ppu;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use std::collections::HashSet;

pub use cpu::{CpuDebugger, CpuState};
pub use disassembler::{
    disassemble_count, disassemble_instruction, disassemble_range, DisassembledInstruction,
};
pub use logger::{LogLevel, Logger, TraceEntry};
pub use memory::{MemoryRegion, MemoryViewer};
pub use ppu::{PpuDebugger, PpuState, SpriteInfo};

/// Main debugger interface
///
/// Provides a unified interface for all debugging features.
/// The debugger can be enabled/disabled at runtime to avoid
/// performance overhead during normal emulation.
pub struct Debugger {
    /// CPU debugger
    pub cpu: CpuDebugger,

    /// Memory viewer
    pub memory: MemoryViewer,

    /// PPU debugger
    pub ppu: PpuDebugger,

    /// Logger
    pub logger: Logger,

    /// Whether debugging is enabled
    enabled: bool,

    /// Breakpoints (set of addresses)
    breakpoints: HashSet<u16>,

    /// Whether execution is paused
    paused: bool,

    /// Step mode (execute one instruction then pause)
    step_mode: bool,
}

impl Debugger {
    /// Create a new debugger instance
    ///
    /// # Returns
    ///
    /// A new debugger with all features enabled
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::debug::Debugger;
    ///
    /// let debugger = Debugger::new();
    /// ```
    pub fn new() -> Self {
        Debugger {
            cpu: CpuDebugger::new(),
            memory: MemoryViewer::new(),
            ppu: PpuDebugger::new(),
            logger: Logger::new(),
            enabled: false,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: false,
        }
    }

    /// Enable debugging
    ///
    /// When enabled, the debugger will track CPU and PPU state,
    /// check breakpoints, and log execution.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debugging
    ///
    /// When disabled, the debugger has minimal overhead.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if debugging is enabled
    ///
    /// # Returns
    ///
    /// `true` if debugging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a breakpoint at the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to break at
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::debug::Debugger;
    ///
    /// let mut debugger = Debugger::new();
    /// debugger.add_breakpoint(0x8000); // Break at start of ROM
    /// ```
    pub fn add_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    /// Remove a breakpoint at the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to remove breakpoint from
    ///
    /// # Returns
    ///
    /// `true` if a breakpoint was removed
    pub fn remove_breakpoint(&mut self, addr: u16) -> bool {
        self.breakpoints.remove(&addr)
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Get all breakpoints
    ///
    /// # Returns
    ///
    /// A vector containing all breakpoint addresses
    pub fn breakpoints(&self) -> Vec<u16> {
        self.breakpoints.iter().copied().collect()
    }

    /// Check if execution should break at the current PC
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    ///
    /// # Returns
    ///
    /// `true` if execution should pause
    pub fn should_break(&self, cpu: &Cpu) -> bool {
        if !self.enabled {
            return false;
        }

        self.breakpoints.contains(&cpu.pc)
    }

    /// Pause execution
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume execution
    pub fn resume(&mut self) {
        self.paused = false;
        self.step_mode = false;
    }

    /// Check if execution is paused
    ///
    /// # Returns
    ///
    /// `true` if execution is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Execute one instruction and pause (step mode)
    ///
    /// This allows exactly one instruction to execute even if the debugger
    /// is currently paused, then re-enters the paused state.
    pub fn step(&mut self) {
        self.step_mode = true;
    }

    /// Called before executing each CPU instruction
    ///
    /// This should be called by the emulator before executing each instruction.
    /// It checks breakpoints, logs execution, and handles stepping.
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// `true` if execution should continue, `false` if paused
    pub fn before_instruction(&mut self, cpu: &Cpu, bus: &mut Bus) -> bool {
        if !self.enabled {
            return true;
        }

        let stepping = self.step_mode;

        // If we're not stepping and already paused, don't execute further instructions
        if self.paused && !stepping {
            return false;
        }

        // In normal run mode (not single-step), honor breakpoints
        if !stepping && self.should_break(cpu) {
            self.pause();
            return false;
        }

        // Log CPU state if tracing is enabled
        if self.logger.is_cpu_trace_enabled() {
            let state = self.cpu.capture_state(cpu, bus);
            self.logger.log_cpu_state(&state);
        }

        // If we were in step mode, consume it and ensure we pause again
        // after this instruction has executed
        if stepping {
            self.step_mode = false;
            self.paused = true;
        }

        true
    }

    /// Called after each PPU step
    ///
    /// This should be called by the emulator after each PPU step.
    /// It logs PPU state if tracing is enabled.
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    pub fn after_ppu_step(&mut self, ppu: &Ppu) {
        if !self.enabled {
            return;
        }

        // Log PPU state if tracing is enabled
        if self.logger.is_ppu_trace_enabled() {
            let state = self.ppu.capture_state(ppu);
            self.logger.log_ppu_state(&state);
        }
    }

    /// Get the current CPU state
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// The current CPU state
    pub fn get_cpu_state(&self, cpu: &Cpu, bus: &mut Bus) -> CpuState {
        self.cpu.capture_state(cpu, bus)
    }

    /// Get the current PPU state
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// The current PPU state
    pub fn get_ppu_state(&self, ppu: &Ppu) -> PpuState {
        self.ppu.capture_state(ppu)
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_creation() {
        let debugger = Debugger::new();
        assert!(!debugger.is_enabled());
        assert!(!debugger.is_paused());
    }

    #[test]
    fn test_enable_disable() {
        let mut debugger = Debugger::new();

        debugger.enable();
        assert!(debugger.is_enabled());

        debugger.disable();
        assert!(!debugger.is_enabled());
    }

    #[test]
    fn test_breakpoints() {
        let mut debugger = Debugger::new();

        debugger.add_breakpoint(0x8000);
        debugger.add_breakpoint(0x8010);

        let breakpoints = debugger.breakpoints();
        assert_eq!(breakpoints.len(), 2);
        assert!(breakpoints.contains(&0x8000));
        assert!(breakpoints.contains(&0x8010));

        assert!(debugger.remove_breakpoint(0x8000));
        assert!(!debugger.remove_breakpoint(0x8000));

        let breakpoints = debugger.breakpoints();
        assert_eq!(breakpoints.len(), 1);

        debugger.clear_breakpoints();
        assert!(debugger.breakpoints().is_empty());
    }

    #[test]
    fn test_pause_resume() {
        let mut debugger = Debugger::new();

        debugger.pause();
        assert!(debugger.is_paused());

        debugger.resume();
        assert!(!debugger.is_paused());
    }

    #[test]
    fn test_step() {
        let mut debugger = Debugger::new();

        debugger.step();
        assert!(debugger.step_mode);
    }

    #[test]
    fn test_step_executes_one_instruction() {
        use crate::bus::Bus;
        use crate::cpu::Cpu;

        let mut debugger = Debugger::new();
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        debugger.enable();

        // Set up a simple program
        bus.write(0x8000, 0xEA); // NOP
        bus.write(0x8001, 0xEA); // NOP
        cpu.pc = 0x8000;

        // Start in paused state
        debugger.pause();
        assert!(debugger.is_paused());

        // Calling before_instruction while paused should return false
        assert!(!debugger.before_instruction(&cpu, &mut bus));
        assert_eq!(cpu.pc, 0x8000);

        // Call step() to request one instruction execution
        debugger.step();

        // before_instruction should now return true (allowing execution)
        assert!(debugger.before_instruction(&cpu, &mut bus));

        // Simulate instruction execution
        cpu.pc = 0x8001;

        // Should be paused again after the step
        assert!(debugger.is_paused());
        assert!(!debugger.step_mode);

        // Next call should return false (paused)
        assert!(!debugger.before_instruction(&cpu, &mut bus));
    }

    #[test]
    fn test_step_over_breakpoint() {
        use crate::bus::Bus;
        use crate::cpu::Cpu;

        let mut debugger = Debugger::new();
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        debugger.enable();

        // Set up a program with a breakpoint
        bus.write(0x8000, 0xEA); // NOP
        bus.write(0x8001, 0xEA); // NOP
        cpu.pc = 0x8000;

        // Add breakpoint at current PC
        debugger.add_breakpoint(0x8000);

        // In normal mode, breakpoint should stop execution
        assert!(!debugger.before_instruction(&cpu, &mut bus));
        assert!(debugger.is_paused());

        // Now step - this should execute the instruction at the breakpoint
        debugger.step();
        assert!(debugger.before_instruction(&cpu, &mut bus));

        // Simulate executing the instruction
        cpu.pc = 0x8001;

        // Should be paused after the step
        assert!(debugger.is_paused());
    }

    #[test]
    fn test_normal_execution_with_breakpoint() {
        use crate::bus::Bus;
        use crate::cpu::Cpu;

        let mut debugger = Debugger::new();
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        debugger.enable();

        bus.write(0x8000, 0xEA);
        cpu.pc = 0x8000;

        // Add breakpoint
        debugger.add_breakpoint(0x8000);

        // Not paused initially
        assert!(!debugger.is_paused());

        // Hitting breakpoint should pause
        assert!(!debugger.before_instruction(&cpu, &mut bus));
        assert!(debugger.is_paused());
    }

    #[test]
    fn test_resume_clears_step_mode() {
        let mut debugger = Debugger::new();

        debugger.step();
        assert!(debugger.step_mode);

        debugger.resume();
        assert!(!debugger.step_mode);
        assert!(!debugger.is_paused());
    }
}
