// Emulator module - Main emulator coordinator
//
// This module provides the main emulator interface that coordinates all NES components
// (CPU, PPU, APU, Bus) and implements quality-of-life features like save states,
// screenshots, speed control, and configuration management.

mod config;
mod recent_roms;
mod save_state;
mod screenshot;

pub use config::{EmulatorConfig, SpeedMode};
pub use recent_roms::RecentRomsList;
pub use save_state::{SaveState, SaveStateError};
pub use screenshot::{save_screenshot, ScreenshotError};

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Main emulator structure
///
/// Coordinates all NES components and provides high-level functionality
/// for running games, saving/loading states, and managing configuration.
pub struct Emulator {
    /// CPU (6502 processor)
    cpu: Cpu,

    /// Bus (connects all components)
    bus: Bus,

    /// Currently loaded cartridge
    ///
    /// Note: The cartridge is stored here but not yet fully integrated with the Bus.
    /// When the mapper system is implemented, this will be properly wired into
    /// the Bus's memory mapping system.
    cartridge: Option<Cartridge>,

    /// Configuration
    config: EmulatorConfig,

    /// Currently loaded ROM path
    rom_path: Option<PathBuf>,

    /// Paused state
    paused: bool,

    /// Speed mode
    speed_mode: SpeedMode,

    /// Frame timing for speed control
    #[allow(dead_code)]
    last_frame_time: Option<Instant>,
}

impl Emulator {
    /// Create a new emulator instance
    ///
    /// Initializes all components to their power-on state.
    ///
    /// # Returns
    ///
    /// A new emulator instance
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// ```
    pub fn new() -> Self {
        Emulator {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cartridge: None,
            config: EmulatorConfig::load_or_default(),
            rom_path: None,
            paused: false,
            speed_mode: SpeedMode::Normal,
            last_frame_time: None,
        }
    }

    /// Load a ROM file
    ///
    /// Loads a ROM from the specified path and initializes the emulator state.
    /// Adds the ROM to the recent ROMs list.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ROM file (.nes)
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.load_rom("game.nes").expect("Failed to load ROM");
    /// ```
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let cartridge = Cartridge::from_ines_file(path)?;

        // Load PRG-ROM data into bus
        // Note: This is a temporary solution until the mapper system is implemented.
        // The Bus currently uses a fixed ROM array instead of a proper cartridge interface.
        if !cartridge.prg_rom.is_empty() {
            // Load PRG-ROM starting at offset 0x3FE0 in the Bus ROM array
            // (which maps to $8000 in CPU address space)
            self.bus.load_rom(&cartridge.prg_rom, 0x3FE0);
        }

        // Store the cartridge and path
        self.cartridge = Some(cartridge);
        self.rom_path = Some(path.to_path_buf());

        // Add to recent ROMs list
        let mut recent_roms = RecentRomsList::load_or_default();
        recent_roms.add(path);
        recent_roms.save()?;

        // Reset the emulator
        self.reset();

        Ok(())
    }

    /// Reset the emulator
    ///
    /// Resets all components to their power-on state, as if pressing the reset button.
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.reset();
    /// ```
    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
        // PPU and APU will be reset through the bus
        self.paused = false;
    }

    /// Save state to a file
    ///
    /// Saves the complete emulator state to a file slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - Save slot number (0-9)
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.save_state(0).expect("Failed to save state");
    /// ```
    pub fn save_state(&self, slot: u8) -> Result<(), SaveStateError> {
        let save_state = SaveState::from_emulator(self)?;
        save_state.save_to_file(slot, self.rom_path.as_deref())
    }

    /// Quick save to slot 0
    ///
    /// Convenience method for quick save (F5 hotkey).
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn quick_save(&self) -> Result<(), SaveStateError> {
        self.save_state(0)
    }

    /// Load state from a file
    ///
    /// Loads the complete emulator state from a file slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - Save slot number (0-9)
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.load_state(0).expect("Failed to load state");
    /// ```
    pub fn load_state(&mut self, slot: u8) -> Result<(), SaveStateError> {
        let save_state = SaveState::load_from_file(slot, self.rom_path.as_deref())?;
        save_state.restore_to_emulator(self)
    }

    /// Quick load from slot 0
    ///
    /// Convenience method for quick load (F7 hotkey).
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn quick_load(&mut self) -> Result<(), SaveStateError> {
        self.load_state(0)
    }

    /// Take a screenshot
    ///
    /// Captures the current frame buffer and saves it as a PNG file.
    ///
    /// # Returns
    ///
    /// Result containing the path to the saved screenshot or an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// let screenshot_path = emulator.screenshot().expect("Failed to save screenshot");
    /// println!("Screenshot saved to: {}", screenshot_path.display());
    /// ```
    pub fn screenshot(&self) -> Result<PathBuf, ScreenshotError> {
        screenshot::save_screenshot(self.bus.ppu().frame(), self.rom_path.as_deref())
    }

    /// Set speed mode
    ///
    /// Controls emulation speed (normal, fast forward, slow motion).
    ///
    /// # Arguments
    ///
    /// * `mode` - The speed mode to set
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::{Emulator, SpeedMode};
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.set_speed_mode(SpeedMode::FastForward2x);
    /// ```
    pub fn set_speed_mode(&mut self, mode: SpeedMode) {
        self.speed_mode = mode;
    }

    /// Get current speed mode
    ///
    /// # Returns
    ///
    /// The current speed mode
    pub fn speed_mode(&self) -> SpeedMode {
        self.speed_mode
    }

    /// Pause the emulator
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.pause();
    /// ```
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the emulator
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.resume();
    /// ```
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause state
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::Emulator;
    ///
    /// let mut emulator = Emulator::new();
    /// emulator.toggle_pause();
    /// ```
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Check if emulator is paused
    ///
    /// # Returns
    ///
    /// true if paused, false otherwise
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get reference to CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get mutable reference to CPU
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Get reference to Bus
    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    /// Get mutable reference to Bus
    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.bus
    }

    /// Get reference to configuration
    pub fn config(&self) -> &EmulatorConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut EmulatorConfig {
        &mut self.config
    }

    /// Get the currently loaded ROM path
    pub fn rom_path(&self) -> Option<&Path> {
        self.rom_path.as_deref()
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
