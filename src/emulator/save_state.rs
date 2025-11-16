// Save state functionality
//
// Implements serialization and deserialization of the complete emulator state
// to enable save states and quick save/load functionality.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur during save state operations
#[derive(Debug)]
pub enum SaveStateError {
    /// I/O error
    Io(io::Error),

    /// Serialization/deserialization error
    Serialization(serde_json::Error),

    /// Save state version mismatch
    VersionMismatch { expected: u32, found: u32 },

    /// No ROM loaded
    NoRomLoaded,
}

impl std::fmt::Display for SaveStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveStateError::Io(e) => write!(f, "I/O error: {}", e),
            SaveStateError::Serialization(e) => write!(f, "Serialization error: {}", e),
            SaveStateError::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Version mismatch: expected {}, found {}",
                    expected, found
                )
            }
            SaveStateError::NoRomLoaded => write!(f, "No ROM loaded"),
        }
    }
}

impl std::error::Error for SaveStateError {}

impl From<io::Error> for SaveStateError {
    fn from(e: io::Error) -> Self {
        SaveStateError::Io(e)
    }
}

impl From<serde_json::Error> for SaveStateError {
    fn from(e: serde_json::Error) -> Self {
        SaveStateError::Serialization(e)
    }
}

/// Current save state format version
const SAVE_STATE_VERSION: u32 = 1;

/// Complete emulator save state
///
/// Contains all the state needed to restore the emulator to an exact point in time.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveState {
    /// Version number for compatibility checking
    version: u32,

    /// Timestamp when the save state was created
    timestamp: String,

    /// ROM identifier (file name for validation)
    rom_name: Option<String>,

    /// CPU state
    cpu_state: CpuState,

    /// PPU state (placeholder for now)
    ppu_state: PpuState,

    /// APU state (placeholder for now)
    apu_state: ApuState,

    /// RAM contents
    ram: Vec<u8>,

    /// VRAM contents (nametables)
    vram: Vec<u8>,

    /// Palette RAM
    palette_ram: Vec<u8>,

    /// OAM (sprite memory)
    oam: Vec<u8>,

    /// Cartridge RAM (if battery-backed)
    cartridge_ram: Option<Vec<u8>>,
}

/// CPU state for serialization
#[derive(Debug, Serialize, Deserialize)]
struct CpuState {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
}

/// PPU state for serialization (placeholder)
#[derive(Debug, Serialize, Deserialize)]
struct PpuState {
    // PPU registers
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oam_addr: u8,

    // Internal scroll registers
    v: u16,
    t: u16,
    fine_x: u8,
    write_latch: bool,
    read_buffer: u8,

    // Timing
    scanline: u16,
    cycle: u16,
    frame: u64,
}

/// APU state for serialization (placeholder)
#[derive(Debug, Serialize, Deserialize)]
struct ApuState {
    // Placeholder - will be expanded with actual APU registers
    placeholder: u8,
}

impl SaveState {
    /// Create a save state from the current emulator state
    ///
    /// # Arguments
    ///
    /// * `emulator` - Reference to the emulator
    ///
    /// # Returns
    ///
    /// Result containing the save state or an error
    pub fn from_emulator(emulator: &super::Emulator) -> Result<Self, SaveStateError> {
        let cpu = emulator.cpu();
        let bus = emulator.bus();

        // Get ROM name for validation
        let rom_name = emulator
            .rom_path()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        // Create timestamp
        let timestamp = chrono::Local::now().to_rfc3339();

        // Capture CPU state
        let cpu_state = CpuState {
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            sp: cpu.sp,
            pc: cpu.pc,
            status: cpu.status,
            cycles: cpu.cycles,
        };

        // Capture PPU state
        let ppu = bus.ppu();
        let ppu_state = PpuState {
            ppuctrl: ppu.ppuctrl,
            ppumask: ppu.ppumask,
            ppustatus: ppu.ppustatus,
            oam_addr: ppu.oam_addr,
            v: ppu.v,
            t: ppu.t,
            fine_x: ppu.fine_x,
            write_latch: ppu.write_latch,
            read_buffer: ppu.read_buffer,
            scanline: ppu.scanline,
            cycle: ppu.cycle,
            frame: ppu.frame,
        };

        // APU state (placeholder)
        let apu_state = ApuState { placeholder: 0 };

        // Capture memory
        let ram = bus.ram_contents().to_vec();
        let vram = ppu.nametables.to_vec();
        let palette_ram = ppu.palette_ram.to_vec();
        let oam = ppu.oam.to_vec();

        Ok(SaveState {
            version: SAVE_STATE_VERSION,
            timestamp,
            rom_name,
            cpu_state,
            ppu_state,
            apu_state,
            ram,
            vram,
            palette_ram,
            oam,
            cartridge_ram: None, // TODO: Capture cartridge RAM
        })
    }

    /// Restore emulator state from this save state
    ///
    /// # Arguments
    ///
    /// * `emulator` - Mutable reference to the emulator
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn restore_to_emulator(
        &self,
        emulator: &mut super::Emulator,
    ) -> Result<(), SaveStateError> {
        // Version check
        if self.version != SAVE_STATE_VERSION {
            return Err(SaveStateError::VersionMismatch {
                expected: SAVE_STATE_VERSION,
                found: self.version,
            });
        }

        // Restore CPU state
        let cpu = emulator.cpu_mut();
        cpu.a = self.cpu_state.a;
        cpu.x = self.cpu_state.x;
        cpu.y = self.cpu_state.y;
        cpu.sp = self.cpu_state.sp;
        cpu.pc = self.cpu_state.pc;
        cpu.status = self.cpu_state.status;
        cpu.cycles = self.cpu_state.cycles;

        // Restore bus/memory state
        let bus = emulator.bus_mut();
        bus.restore_ram_contents(&self.ram);

        // Restore PPU state
        let ppu = bus.ppu_mut();
        ppu.ppuctrl = self.ppu_state.ppuctrl;
        ppu.ppumask = self.ppu_state.ppumask;
        ppu.ppustatus = self.ppu_state.ppustatus;
        ppu.oam_addr = self.ppu_state.oam_addr;
        ppu.v = self.ppu_state.v;
        ppu.t = self.ppu_state.t;
        ppu.fine_x = self.ppu_state.fine_x;
        ppu.write_latch = self.ppu_state.write_latch;
        ppu.read_buffer = self.ppu_state.read_buffer;
        ppu.scanline = self.ppu_state.scanline;
        ppu.cycle = self.ppu_state.cycle;
        ppu.frame = self.ppu_state.frame;

        ppu.nametables.copy_from_slice(&self.vram);
        ppu.palette_ram.copy_from_slice(&self.palette_ram);
        ppu.oam.copy_from_slice(&self.oam);

        // TODO: Restore APU state
        // TODO: Restore cartridge RAM

        Ok(())
    }

    /// Save this save state to a file
    ///
    /// # Arguments
    ///
    /// * `slot` - Save slot number (0-9)
    /// * `rom_path` - Optional path to the currently loaded ROM (for naming)
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn save_to_file(&self, slot: u8, rom_path: Option<&Path>) -> Result<(), SaveStateError> {
        let save_dir = Self::get_save_directory(rom_path)?;
        fs::create_dir_all(&save_dir)?;

        let file_path = save_dir.join(format!("slot_{}.state", slot));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;

        Ok(())
    }

    /// Load a save state from a file
    ///
    /// # Arguments
    ///
    /// * `slot` - Save slot number (0-9)
    /// * `rom_path` - Optional path to the currently loaded ROM (for naming)
    ///
    /// # Returns
    ///
    /// Result containing the save state or an error
    pub fn load_from_file(slot: u8, rom_path: Option<&Path>) -> Result<Self, SaveStateError> {
        let save_dir = Self::get_save_directory(rom_path)?;
        let file_path = save_dir.join(format!("slot_{}.state", slot));

        let json = fs::read_to_string(file_path)?;
        let save_state: SaveState = serde_json::from_str(&json)?;

        Ok(save_state)
    }

    /// Get the save directory for the current ROM
    ///
    /// Creates a directory structure like: saves/<rom_name>/
    fn get_save_directory(rom_path: Option<&Path>) -> Result<PathBuf, SaveStateError> {
        let base_dir = PathBuf::from("saves");

        if let Some(rom_path) = rom_path {
            if let Some(rom_name) = rom_path.file_stem() {
                Ok(base_dir.join(rom_name))
            } else {
                Ok(base_dir.join("default"))
            }
        } else {
            // No ROM loaded, use default directory
            Ok(base_dir.join("default"))
        }
    }
}
