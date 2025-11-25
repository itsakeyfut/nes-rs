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
pub mod execution_log;
pub mod logger;
pub mod memory;
pub mod ppu;
pub mod ui;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub use cpu::{CpuDebugger, CpuState};
pub use disassembler::{
    disassemble_count, disassemble_instruction, disassemble_range, DisassembledInstruction,
};
pub use execution_log::{ExecutionLog, ExecutionLogEntry, LogFilter, PpuEventType};
pub use logger::{LogLevel, Logger, TraceEntry};
pub use memory::{CpuMemoryRegionType, MemoryRegion, MemoryViewer};
pub use ppu::{PpuDebugger, PpuState, SpriteInfo};
pub use ui::DebugUI;

/// Step mode for execution control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// No stepping, continuous execution
    None,
    /// Execute one CPU instruction
    Instruction,
    /// Execute until next PPU scanline
    Scanline,
    /// Execute until next frame (VBlank)
    Frame,
}

/// Performance metrics for execution monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Current frames per second
    pub fps: f32,
    /// CPU cycles executed in last frame
    pub cpu_cycles_per_frame: u64,
    /// PPU cycles executed in last frame
    pub ppu_cycles_per_frame: u64,
    /// Total frames executed
    pub total_frames: u64,
    /// Total instructions executed
    pub total_instructions: u64,
    /// Time spent in emulation
    pub execution_time: Duration,
    /// Frame times for graphing (last 60 frames)
    pub frame_times: Vec<Duration>,
    /// Start time for uptime tracking
    start_time: Instant,
    /// Last frame time for FPS calculation
    last_frame_time: Option<Instant>,
    /// CPU cycles at start of current frame
    frame_start_cpu_cycles: u64,
    /// PPU cycles at start of current frame
    frame_start_ppu_cycles: u64,
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            cpu_cycles_per_frame: 0,
            ppu_cycles_per_frame: 0,
            total_frames: 0,
            total_instructions: 0,
            execution_time: Duration::ZERO,
            frame_times: Vec::with_capacity(60),
            start_time: Instant::now(),
            last_frame_time: None,
            frame_start_cpu_cycles: 0,
            frame_start_ppu_cycles: 0,
        }
    }

    /// Update metrics at the start of a new frame
    pub fn start_frame(&mut self, cpu_cycles: u64, ppu_cycles: u64) {
        self.frame_start_cpu_cycles = cpu_cycles;
        self.frame_start_ppu_cycles = ppu_cycles;
    }

    /// Update metrics at the end of a frame
    pub fn end_frame(&mut self, cpu_cycles: u64, ppu_cycles: u64) {
        let now = Instant::now();

        // Calculate frame time
        if let Some(last_time) = self.last_frame_time {
            let frame_time = now.duration_since(last_time);

            // Update FPS
            if !frame_time.is_zero() {
                self.fps = 1.0 / frame_time.as_secs_f32();
            }

            // Store frame time for graphing (keep last 60 frames)
            self.frame_times.push(frame_time);
            if self.frame_times.len() > 60 {
                self.frame_times.remove(0);
            }
        }

        self.last_frame_time = Some(now);

        // Calculate cycles per frame
        self.cpu_cycles_per_frame = cpu_cycles.saturating_sub(self.frame_start_cpu_cycles);
        self.ppu_cycles_per_frame = ppu_cycles.saturating_sub(self.frame_start_ppu_cycles);

        // Update totals
        self.total_frames += 1;
        self.execution_time = self.start_time.elapsed();
    }

    /// Record an instruction execution
    pub fn record_instruction(&mut self) {
        self.total_instructions += 1;
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get uptime as a formatted string
    pub fn uptime_string(&self) -> String {
        let uptime = self.start_time.elapsed();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        let seconds = uptime.as_secs() % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Execution log
    pub execution_log: ExecutionLog,

    /// Whether debugging is enabled
    enabled: bool,

    /// Breakpoints (set of addresses)
    breakpoints: HashSet<u16>,

    /// Whether execution is paused
    paused: bool,

    /// Current step mode
    step_mode: StepMode,

    /// Target scanline for step scanline mode
    target_scanline: Option<u16>,

    /// Target frame for step frame mode
    target_frame: Option<u64>,

    /// Performance metrics
    pub metrics: PerformanceMetrics,
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
            execution_log: ExecutionLog::new(),
            enabled: false,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: StepMode::None,
            target_scanline: None,
            target_frame: None,
            metrics: PerformanceMetrics::new(),
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
        self.step_mode = StepMode::None;
        self.target_scanline = None;
        self.target_frame = None;
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
        self.step_instruction();
    }

    /// Execute one CPU instruction (alias for step)
    pub fn step_instruction(&mut self) {
        self.step_mode = StepMode::Instruction;
        self.target_scanline = None;
        self.target_frame = None;
    }

    /// Execute until the next PPU scanline
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU to get current scanline
    pub fn step_scanline(&mut self, ppu: &Ppu) {
        self.step_mode = StepMode::Scanline;
        // Target the next scanline (wrap around at 261)
        let current_scanline = ppu.scanline();
        self.target_scanline = Some((current_scanline + 1) % 262);
        self.target_frame = None;
    }

    /// Execute until the next frame (VBlank)
    pub fn step_frame(&mut self) {
        self.step_mode = StepMode::Frame;
        self.target_scanline = None;
        // Target the next frame
        self.target_frame = Some(self.metrics.total_frames + 1);
    }

    /// Get the current step mode
    pub fn step_mode(&self) -> StepMode {
        self.step_mode
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

        let stepping = self.step_mode != StepMode::None;

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

        // Log instruction execution if enabled
        if self.execution_log.is_instruction_logging_enabled() {
            let instruction = disassemble_instruction(cpu.pc, bus);
            self.execution_log.log_instruction(
                cpu.cycles,
                cpu.pc,
                instruction,
                cpu.a,
                cpu.x,
                cpu.y,
                cpu.status,
                cpu.sp,
            );
        }

        // If we were in step instruction mode, consume it and pause
        if self.step_mode == StepMode::Instruction {
            self.step_mode = StepMode::None;
            self.paused = true;
        }

        // Record instruction execution for performance metrics
        self.metrics.record_instruction();

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

        // Check if we've reached the target scanline
        if self.step_mode == StepMode::Scanline {
            if let Some(target) = self.target_scanline {
                if ppu.scanline() == target {
                    self.step_mode = StepMode::None;
                    self.target_scanline = None;
                    self.paused = true;
                }
            }
        }
    }

    /// Called at the start of a new frame
    ///
    /// This should be called by the emulator at the start of each frame
    /// to update performance metrics and check frame stepping.
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU for cycle count
    pub fn on_frame_start(&mut self, cpu: &Cpu) {
        if !self.enabled {
            return;
        }

        // PPU runs at 3x CPU speed
        let ppu_cycles = cpu.cycles * 3;
        self.metrics.start_frame(cpu.cycles, ppu_cycles);
    }

    /// Called at the end of a frame (VBlank)
    ///
    /// This should be called by the emulator at the end of each frame
    /// to update performance metrics and check frame stepping.
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU for cycle count
    pub fn on_frame_end(&mut self, cpu: &Cpu) {
        if !self.enabled {
            return;
        }

        // PPU runs at 3x CPU speed
        let ppu_cycles = cpu.cycles * 3;
        self.metrics.end_frame(cpu.cycles, ppu_cycles);

        // Check if we've reached the target frame
        if self.step_mode == StepMode::Frame {
            if let Some(target) = self.target_frame {
                if self.metrics.total_frames >= target {
                    self.step_mode = StepMode::None;
                    self.target_frame = None;
                    self.paused = true;
                }
            }
        }
    }

    /// Log a memory read access
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU for cycle count and PC
    /// * `address` - Memory address being read
    /// * `value` - Value read from memory
    pub fn log_memory_read(&mut self, cpu: &Cpu, address: u16, value: u8) {
        if !self.enabled {
            return;
        }

        self.execution_log
            .log_memory_read(cpu.cycles, address, value, cpu.pc);
    }

    /// Log a memory write access
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU for cycle count and PC
    /// * `address` - Memory address being written
    /// * `value` - Value written to memory
    pub fn log_memory_write(&mut self, cpu: &Cpu, address: u16, value: u8) {
        if !self.enabled {
            return;
        }

        self.execution_log
            .log_memory_write(cpu.cycles, address, value, cpu.pc);
    }

    /// Log a PPU event
    ///
    /// # Arguments
    ///
    /// * `cpu` - Reference to the CPU for cycle count
    /// * `event` - The PPU event to log
    pub fn log_ppu_event(&mut self, cpu: &Cpu, event: PpuEventType) {
        if !self.enabled {
            return;
        }

        self.execution_log.log_ppu_event(cpu.cycles, event);
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
        assert_eq!(debugger.step_mode(), StepMode::Instruction);
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
        assert_eq!(debugger.step_mode(), StepMode::None);

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
        assert_eq!(debugger.step_mode(), StepMode::Instruction);

        debugger.resume();
        assert_eq!(debugger.step_mode(), StepMode::None);
        assert!(!debugger.is_paused());
    }

    #[test]
    fn test_step_modes() {
        use crate::ppu::Ppu;

        let mut debugger = Debugger::new();
        let ppu = Ppu::new();

        // Test step instruction
        debugger.step_instruction();
        assert_eq!(debugger.step_mode(), StepMode::Instruction);

        // Test step scanline
        debugger.step_scanline(&ppu);
        assert_eq!(debugger.step_mode(), StepMode::Scanline);
        assert!(debugger.target_scanline.is_some());

        // Test step frame
        debugger.step_frame();
        assert_eq!(debugger.step_mode(), StepMode::Frame);
        assert!(debugger.target_frame.is_some());

        // Resume should clear step mode
        debugger.resume();
        assert_eq!(debugger.step_mode(), StepMode::None);
        assert!(debugger.target_scanline.is_none());
        assert!(debugger.target_frame.is_none());
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        assert_eq!(metrics.fps, 0.0);
        assert_eq!(metrics.total_frames, 0);
        assert_eq!(metrics.total_instructions, 0);

        // Record some instructions
        metrics.record_instruction();
        metrics.record_instruction();
        assert_eq!(metrics.total_instructions, 2);

        // Simulate frame start and end
        metrics.start_frame(0, 0);
        metrics.end_frame(100, 300);

        assert_eq!(metrics.total_frames, 1);
        assert_eq!(metrics.cpu_cycles_per_frame, 100);
        assert_eq!(metrics.ppu_cycles_per_frame, 300);

        // Test reset
        metrics.reset();
        assert_eq!(metrics.total_frames, 0);
        assert_eq!(metrics.total_instructions, 0);
    }

    #[test]
    fn test_performance_metrics_uptime() {
        let metrics = PerformanceMetrics::new();
        let uptime = metrics.uptime_string();

        // Should be in format HH:MM:SS
        assert!(uptime.contains(':'));
        assert_eq!(uptime.len(), 8);
    }

    #[test]
    fn test_step_scanline_wraps() {
        use crate::ppu::Ppu;

        let mut debugger = Debugger::new();
        let ppu = Ppu::new();

        // Set scanline to 261 (last scanline)
        // Note: We can't directly set the scanline, so we'll just test the logic
        debugger.step_scanline(&ppu);

        // Target should be set
        assert!(debugger.target_scanline.is_some());
    }

    #[test]
    fn test_step_frame_increments_target() {
        let mut debugger = Debugger::new();

        // Set some frames as already executed
        debugger.metrics.total_frames = 10;

        debugger.step_frame();

        // Target frame should be 11 (next frame)
        assert_eq!(debugger.target_frame, Some(11));
    }
}
