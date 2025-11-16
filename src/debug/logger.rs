// Logger - Trace logging for CPU and PPU execution
//
// Provides:
// - CPU trace logging
// - PPU trace logging
// - Configurable log levels
// - Log output to file or memory

use super::cpu::CpuState;
use super::ppu::PpuState;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// No logging
    None,
    /// Error messages only
    Error,
    /// Warnings and errors
    Warning,
    /// Info, warnings, and errors
    Info,
    /// Debug information (includes traces)
    Debug,
    /// Verbose trace logging
    Trace,
}

/// Trace entry
///
/// Represents a single trace log entry
#[derive(Debug, Clone)]
pub enum TraceEntry {
    /// CPU state trace
    Cpu(CpuState),
    /// PPU state trace
    Ppu(PpuState),
    /// Custom message
    Message(String),
}

impl std::fmt::Display for TraceEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEntry::Cpu(state) => write!(f, "CPU: {}", state),
            TraceEntry::Ppu(state) => write!(f, "PPU: {}", state),
            TraceEntry::Message(msg) => write!(f, "{}", msg),
        }
    }
}

/// Logger
///
/// Handles trace logging for CPU and PPU execution.
/// Can log to memory buffer or file.
pub struct Logger {
    /// Current log level
    log_level: LogLevel,

    /// Enable CPU trace logging
    cpu_trace: bool,

    /// Enable PPU trace logging
    ppu_trace: bool,

    /// In-memory trace buffer
    trace_buffer: Vec<TraceEntry>,

    /// Maximum number of entries in trace buffer (0 = unlimited)
    max_buffer_size: usize,

    /// Output file
    output_file: Option<File>,
}

impl Logger {
    /// Create a new logger
    ///
    /// # Returns
    ///
    /// A new logger instance with default settings
    pub fn new() -> Self {
        Logger {
            log_level: LogLevel::None,
            cpu_trace: false,
            ppu_trace: false,
            trace_buffer: Vec::new(),
            max_buffer_size: 10000,
            output_file: None,
        }
    }

    /// Set the log level
    ///
    /// # Arguments
    ///
    /// * `level` - The log level to set
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Get the current log level
    ///
    /// # Returns
    ///
    /// The current log level
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Enable CPU trace logging
    pub fn enable_cpu_trace(&mut self) {
        self.cpu_trace = true;
    }

    /// Disable CPU trace logging
    pub fn disable_cpu_trace(&mut self) {
        self.cpu_trace = false;
    }

    /// Check if CPU trace logging is enabled
    ///
    /// # Returns
    ///
    /// `true` if CPU trace is enabled
    pub fn is_cpu_trace_enabled(&self) -> bool {
        self.cpu_trace && self.log_level >= LogLevel::Trace
    }

    /// Enable PPU trace logging
    pub fn enable_ppu_trace(&mut self) {
        self.ppu_trace = true;
    }

    /// Disable PPU trace logging
    pub fn disable_ppu_trace(&mut self) {
        self.ppu_trace = false;
    }

    /// Check if PPU trace logging is enabled
    ///
    /// # Returns
    ///
    /// `true` if PPU trace is enabled
    pub fn is_ppu_trace_enabled(&self) -> bool {
        self.ppu_trace && self.log_level >= LogLevel::Trace
    }

    /// Set maximum trace buffer size
    ///
    /// When the buffer exceeds this size, old entries are removed.
    /// Set to 0 for unlimited size.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum number of entries (0 = unlimited)
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;

        // Trim buffer if needed
        if size > 0 && self.trace_buffer.len() > size {
            self.trace_buffer.drain(0..self.trace_buffer.len() - size);
        }
    }

    /// Open a log file for output
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the log file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err` otherwise
    pub fn open_log_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        self.output_file = Some(file);
        Ok(())
    }

    /// Close the log file
    pub fn close_log_file(&mut self) {
        self.output_file = None;
    }

    /// Log a CPU state
    ///
    /// # Arguments
    ///
    /// * `state` - The CPU state to log
    pub fn log_cpu_state(&mut self, state: &CpuState) {
        if !self.is_cpu_trace_enabled() {
            return;
        }

        let entry = TraceEntry::Cpu(state.clone());
        self.add_entry(entry);
    }

    /// Log a PPU state
    ///
    /// # Arguments
    ///
    /// * `state` - The PPU state to log
    pub fn log_ppu_state(&mut self, state: &PpuState) {
        if !self.is_ppu_trace_enabled() {
            return;
        }

        let entry = TraceEntry::Ppu(state.clone());
        self.add_entry(entry);
    }

    /// Log a message
    ///
    /// # Arguments
    ///
    /// * `level` - The log level for this message
    /// * `message` - The message to log
    pub fn log_message(&mut self, level: LogLevel, message: String) {
        if level > self.log_level {
            return;
        }

        let entry = TraceEntry::Message(message);
        self.add_entry(entry);
    }

    /// Add an entry to the trace buffer and optionally write to file
    ///
    /// # Arguments
    ///
    /// * `entry` - The trace entry to add
    fn add_entry(&mut self, entry: TraceEntry) {
        // Write to file if enabled
        if let Some(ref mut file) = self.output_file {
            let _ = writeln!(file, "{}", entry);
        }

        // Add to buffer
        self.trace_buffer.push(entry);

        // Trim buffer if needed
        if self.max_buffer_size > 0 && self.trace_buffer.len() > self.max_buffer_size {
            self.trace_buffer.remove(0);
        }
    }

    /// Get the trace buffer
    ///
    /// # Returns
    ///
    /// A slice of all trace entries in the buffer
    pub fn trace_buffer(&self) -> &[TraceEntry] {
        &self.trace_buffer
    }

    /// Clear the trace buffer
    pub fn clear_buffer(&mut self) {
        self.trace_buffer.clear();
    }

    /// Get the last N trace entries
    ///
    /// # Arguments
    ///
    /// * `count` - Number of entries to retrieve
    ///
    /// # Returns
    ///
    /// A slice of the last N entries
    pub fn last_entries(&self, count: usize) -> &[TraceEntry] {
        let start = self.trace_buffer.len().saturating_sub(count);
        &self.trace_buffer[start..]
    }

    /// Format the entire trace buffer as a string
    ///
    /// # Returns
    ///
    /// A formatted string containing all trace entries
    pub fn format_trace_buffer(&self) -> String {
        let mut output = String::new();

        for entry in &self.trace_buffer {
            output.push_str(&format!("{}\n", entry));
        }

        output
    }

    /// Format the last N entries as a string
    ///
    /// # Arguments
    ///
    /// * `count` - Number of entries to format
    ///
    /// # Returns
    ///
    /// A formatted string containing the last N entries
    pub fn format_last_entries(&self, count: usize) -> String {
        let mut output = String::new();

        for entry in self.last_entries(count) {
            output.push_str(&format!("{}\n", entry));
        }

        output
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new();
        assert_eq!(logger.log_level(), LogLevel::None);
        assert!(!logger.is_cpu_trace_enabled());
        assert!(!logger.is_ppu_trace_enabled());
    }

    #[test]
    fn test_set_log_level() {
        let mut logger = Logger::new();

        logger.set_log_level(LogLevel::Debug);
        assert_eq!(logger.log_level(), LogLevel::Debug);

        logger.set_log_level(LogLevel::Trace);
        assert_eq!(logger.log_level(), LogLevel::Trace);
    }

    #[test]
    fn test_cpu_trace_toggle() {
        let mut logger = Logger::new();

        logger.enable_cpu_trace();
        logger.set_log_level(LogLevel::Trace);
        assert!(logger.is_cpu_trace_enabled());

        logger.disable_cpu_trace();
        assert!(!logger.is_cpu_trace_enabled());
    }

    #[test]
    fn test_ppu_trace_toggle() {
        let mut logger = Logger::new();

        logger.enable_ppu_trace();
        logger.set_log_level(LogLevel::Trace);
        assert!(logger.is_ppu_trace_enabled());

        logger.disable_ppu_trace();
        assert!(!logger.is_ppu_trace_enabled());
    }

    #[test]
    fn test_trace_requires_trace_level() {
        let mut logger = Logger::new();

        logger.enable_cpu_trace();
        logger.set_log_level(LogLevel::Debug);
        assert!(!logger.is_cpu_trace_enabled());

        logger.set_log_level(LogLevel::Trace);
        assert!(logger.is_cpu_trace_enabled());
    }

    #[test]
    fn test_log_message() {
        let mut logger = Logger::new();
        logger.set_log_level(LogLevel::Info);

        logger.log_message(LogLevel::Info, "Test message".to_string());

        assert_eq!(logger.trace_buffer().len(), 1);
        match &logger.trace_buffer()[0] {
            TraceEntry::Message(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Expected Message entry"),
        }
    }

    #[test]
    fn test_clear_buffer() {
        let mut logger = Logger::new();
        logger.set_log_level(LogLevel::Info);

        logger.log_message(LogLevel::Info, "Test 1".to_string());
        logger.log_message(LogLevel::Info, "Test 2".to_string());

        assert_eq!(logger.trace_buffer().len(), 2);

        logger.clear_buffer();
        assert_eq!(logger.trace_buffer().len(), 0);
    }

    #[test]
    fn test_max_buffer_size() {
        let mut logger = Logger::new();
        logger.set_log_level(LogLevel::Info);
        logger.set_max_buffer_size(3);

        logger.log_message(LogLevel::Info, "1".to_string());
        logger.log_message(LogLevel::Info, "2".to_string());
        logger.log_message(LogLevel::Info, "3".to_string());
        logger.log_message(LogLevel::Info, "4".to_string());

        assert_eq!(logger.trace_buffer().len(), 3);

        // Should have removed the first entry
        match &logger.trace_buffer()[0] {
            TraceEntry::Message(msg) => assert_eq!(msg, "2"),
            _ => panic!("Expected Message entry"),
        }
    }

    #[test]
    fn test_last_entries() {
        let mut logger = Logger::new();
        logger.set_log_level(LogLevel::Info);

        logger.log_message(LogLevel::Info, "1".to_string());
        logger.log_message(LogLevel::Info, "2".to_string());
        logger.log_message(LogLevel::Info, "3".to_string());

        let last = logger.last_entries(2);
        assert_eq!(last.len(), 2);

        match &last[0] {
            TraceEntry::Message(msg) => assert_eq!(msg, "2"),
            _ => panic!("Expected Message entry"),
        }

        match &last[1] {
            TraceEntry::Message(msg) => assert_eq!(msg, "3"),
            _ => panic!("Expected Message entry"),
        }
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::None < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }
}
