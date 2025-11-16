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

        // Validate array sizes before copying to prevent panics
        if self.vram.len() != ppu.nametables.len()
            || self.palette_ram.len() != ppu.palette_ram.len()
            || self.oam.len() != ppu.oam.len()
        {
            let msg = format!(
                "Save state memory size mismatch: vram={} (expected {}), palette={} (expected {}), oam={} (expected {})",
                self.vram.len(),
                ppu.nametables.len(),
                self.palette_ram.len(),
                ppu.palette_ram.len(),
                self.oam.len(),
                ppu.oam.len()
            );
            return Err(SaveStateError::Serialization(
                serde_json::from_str::<()>(&msg).unwrap_err(),
            ));
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_save_state_error_display() {
        let err = SaveStateError::NoRomLoaded;
        assert_eq!(err.to_string(), "No ROM loaded");

        let err = SaveStateError::VersionMismatch {
            expected: 1,
            found: 2,
        };
        assert_eq!(err.to_string(), "Version mismatch: expected 1, found 2");
    }

    #[test]
    fn test_save_state_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err: SaveStateError = io_err.into();
        assert!(matches!(err, SaveStateError::Io(_)));
    }

    #[test]
    fn test_save_state_version_constant() {
        assert_eq!(SAVE_STATE_VERSION, 1);
    }

    #[test]
    fn test_cpu_state_serialization() {
        let cpu_state = CpuState {
            a: 0x12,
            x: 0x34,
            y: 0x56,
            sp: 0xFD,
            pc: 0x8000,
            status: 0x24,
            cycles: 1000,
        };

        // Test serialization roundtrip
        let json = serde_json::to_string(&cpu_state).unwrap();
        let restored: CpuState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.a, 0x12);
        assert_eq!(restored.x, 0x34);
        assert_eq!(restored.y, 0x56);
        assert_eq!(restored.sp, 0xFD);
        assert_eq!(restored.pc, 0x8000);
        assert_eq!(restored.status, 0x24);
        assert_eq!(restored.cycles, 1000);
    }

    #[test]
    fn test_ppu_state_serialization() {
        let ppu_state = PpuState {
            ppuctrl: 0x80,
            ppumask: 0x1E,
            ppustatus: 0x00,
            oam_addr: 0x00,
            v: 0x2000,
            t: 0x2400,
            fine_x: 3,
            write_latch: false,
            read_buffer: 0x00,
            scanline: 100,
            cycle: 200,
            frame: 1000,
        };

        // Test serialization roundtrip
        let json = serde_json::to_string(&ppu_state).unwrap();
        let restored: PpuState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.ppuctrl, 0x80);
        assert_eq!(restored.ppumask, 0x1E);
        assert_eq!(restored.v, 0x2000);
        assert_eq!(restored.t, 0x2400);
        assert_eq!(restored.fine_x, 3);
        assert_eq!(restored.scanline, 100);
        assert_eq!(restored.cycle, 200);
        assert_eq!(restored.frame, 1000);
    }

    #[test]
    fn test_apu_state_serialization() {
        let apu_state = ApuState { placeholder: 42 };

        let json = serde_json::to_string(&apu_state).unwrap();
        let restored: ApuState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.placeholder, 42);
    }

    #[test]
    fn test_get_save_directory_with_rom() {
        let rom_path = PathBuf::from("/path/to/game.nes");
        let save_dir = SaveState::get_save_directory(Some(&rom_path)).unwrap();

        assert_eq!(save_dir, PathBuf::from("saves/game"));
    }

    #[test]
    fn test_get_save_directory_without_rom() {
        let save_dir = SaveState::get_save_directory(None).unwrap();

        assert_eq!(save_dir, PathBuf::from("saves/default"));
    }

    #[test]
    fn test_get_save_directory_with_invalid_path() {
        let rom_path = PathBuf::from("/");
        let save_dir = SaveState::get_save_directory(Some(&rom_path)).unwrap();

        // Should fall back to default when file_stem() returns None
        assert_eq!(save_dir, PathBuf::from("saves/default"));
    }

    #[test]
    fn test_save_state_structure() {
        // Create a minimal save state
        let save_state = SaveState {
            version: SAVE_STATE_VERSION,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            rom_name: Some("test.nes".to_string()),
            cpu_state: CpuState {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xFD,
                pc: 0x8000,
                status: 0x24,
                cycles: 0,
            },
            ppu_state: PpuState {
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oam_addr: 0,
                v: 0,
                t: 0,
                fine_x: 0,
                write_latch: false,
                read_buffer: 0,
                scanline: 0,
                cycle: 0,
                frame: 0,
            },
            apu_state: ApuState { placeholder: 0 },
            ram: vec![0; 2048],
            vram: vec![0; 2048],
            palette_ram: vec![0; 32],
            oam: vec![0; 256],
            cartridge_ram: None,
        };

        // Test serialization
        let json = serde_json::to_string(&save_state).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"rom_name\":\"test.nes\""));

        // Test deserialization
        let restored: SaveState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, SAVE_STATE_VERSION);
        assert_eq!(restored.rom_name.as_deref(), Some("test.nes"));
        assert_eq!(restored.ram.len(), 2048);
        assert_eq!(restored.vram.len(), 2048);
        assert_eq!(restored.palette_ram.len(), 32);
        assert_eq!(restored.oam.len(), 256);
    }

    #[test]
    fn test_save_state_with_cartridge_ram() {
        let save_state = SaveState {
            version: SAVE_STATE_VERSION,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            rom_name: None,
            cpu_state: CpuState {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xFD,
                pc: 0x8000,
                status: 0x24,
                cycles: 0,
            },
            ppu_state: PpuState {
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oam_addr: 0,
                v: 0,
                t: 0,
                fine_x: 0,
                write_latch: false,
                read_buffer: 0,
                scanline: 0,
                cycle: 0,
                frame: 0,
            },
            apu_state: ApuState { placeholder: 0 },
            ram: vec![0; 2048],
            vram: vec![0; 2048],
            palette_ram: vec![0; 32],
            oam: vec![0; 256],
            cartridge_ram: Some(vec![0xAB; 8192]),
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&save_state).unwrap();
        let restored: SaveState = serde_json::from_str(&json).unwrap();

        assert!(restored.cartridge_ram.is_some());
        assert_eq!(restored.cartridge_ram.as_ref().unwrap().len(), 8192);
        assert_eq!(restored.cartridge_ram.as_ref().unwrap()[0], 0xAB);
    }

    #[test]
    fn test_save_state_preserves_cpu_state() {
        let cpu_state = CpuState {
            a: 0xFF,
            x: 0xAA,
            y: 0x55,
            sp: 0xF0,
            pc: 0xC123,
            status: 0b11010101,
            cycles: 987654321,
        };

        let save_state = SaveState {
            version: SAVE_STATE_VERSION,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            rom_name: None,
            cpu_state,
            ppu_state: PpuState {
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oam_addr: 0,
                v: 0,
                t: 0,
                fine_x: 0,
                write_latch: false,
                read_buffer: 0,
                scanline: 0,
                cycle: 0,
                frame: 0,
            },
            apu_state: ApuState { placeholder: 0 },
            ram: vec![0; 2048],
            vram: vec![0; 2048],
            palette_ram: vec![0; 32],
            oam: vec![0; 256],
            cartridge_ram: None,
        };

        let json = serde_json::to_string(&save_state).unwrap();
        let restored: SaveState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.cpu_state.a, 0xFF);
        assert_eq!(restored.cpu_state.x, 0xAA);
        assert_eq!(restored.cpu_state.y, 0x55);
        assert_eq!(restored.cpu_state.sp, 0xF0);
        assert_eq!(restored.cpu_state.pc, 0xC123);
        assert_eq!(restored.cpu_state.status, 0b11010101);
        assert_eq!(restored.cpu_state.cycles, 987654321);
    }

    #[test]
    fn test_save_state_preserves_ppu_state() {
        let ppu_state = PpuState {
            ppuctrl: 0x88,
            ppumask: 0x1E,
            ppustatus: 0xA0,
            oam_addr: 0x40,
            v: 0x2345,
            t: 0x2678,
            fine_x: 5,
            write_latch: true,
            read_buffer: 0xCD,
            scanline: 240,
            cycle: 340,
            frame: 12345,
        };

        let save_state = SaveState {
            version: SAVE_STATE_VERSION,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            rom_name: None,
            cpu_state: CpuState {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xFD,
                pc: 0x8000,
                status: 0x24,
                cycles: 0,
            },
            ppu_state,
            apu_state: ApuState { placeholder: 0 },
            ram: vec![0; 2048],
            vram: vec![0; 2048],
            palette_ram: vec![0; 32],
            oam: vec![0; 256],
            cartridge_ram: None,
        };

        let json = serde_json::to_string(&save_state).unwrap();
        let restored: SaveState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.ppu_state.ppuctrl, 0x88);
        assert_eq!(restored.ppu_state.ppumask, 0x1E);
        assert_eq!(restored.ppu_state.ppustatus, 0xA0);
        assert_eq!(restored.ppu_state.oam_addr, 0x40);
        assert_eq!(restored.ppu_state.v, 0x2345);
        assert_eq!(restored.ppu_state.t, 0x2678);
        assert_eq!(restored.ppu_state.fine_x, 5);
        assert!(restored.ppu_state.write_latch);
        assert_eq!(restored.ppu_state.read_buffer, 0xCD);
        assert_eq!(restored.ppu_state.scanline, 240);
        assert_eq!(restored.ppu_state.cycle, 340);
        assert_eq!(restored.ppu_state.frame, 12345);
    }
}
