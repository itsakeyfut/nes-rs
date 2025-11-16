// Configuration management
//
// Handles emulator configuration, settings persistence, and speed control.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Default configuration file path
const CONFIG_FILE: &str = "emulator_config.toml";

/// Emulator configuration
///
/// Stores all user-configurable settings for the emulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    /// Video settings
    pub video: VideoConfig,

    /// Audio settings
    pub audio: AudioConfig,

    /// Save state settings
    pub save_state: SaveStateConfig,

    /// Screenshot settings
    pub screenshot: ScreenshotConfig,

    /// Hotkeys
    pub hotkeys: HotkeyConfig,
}

/// Video configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Window scale (1-4)
    pub scale: u32,

    /// Enable VSync
    pub vsync: bool,

    /// Target FPS (usually 60 for NTSC)
    pub fps: u32,

    /// Enable fullscreen
    pub fullscreen: bool,
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Enable audio
    pub enabled: bool,

    /// Volume (0.0-1.0)
    pub volume: f32,
}

/// Save state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveStateConfig {
    /// Number of save slots (1-10)
    pub slots: u8,

    /// Auto-save on exit
    pub auto_save_on_exit: bool,

    /// Save directory
    pub save_directory: PathBuf,
}

/// Screenshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    /// Screenshot directory
    pub screenshot_directory: PathBuf,

    /// Include timestamp in filename
    pub include_timestamp: bool,
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Quick save (default: F5)
    pub quick_save: String,

    /// Quick load (default: F7)
    pub quick_load: String,

    /// Reset (default: F8)
    pub reset: String,

    /// Screenshot (default: F9)
    pub screenshot: String,

    /// Fast forward (default: Tab)
    pub fast_forward: String,

    /// Pause (default: P)
    pub pause: String,
}

/// Speed mode for emulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedMode {
    /// Normal speed (1x)
    Normal,

    /// Fast forward 2x
    FastForward2x,

    /// Fast forward 4x
    FastForward4x,

    /// Slow motion (0.5x)
    SlowMotion,

    /// Paused (0x)
    Paused,
}

impl SpeedMode {
    /// Get the speed multiplier
    ///
    /// # Returns
    ///
    /// The speed multiplier (1.0 = normal speed)
    pub fn multiplier(self) -> f32 {
        match self {
            SpeedMode::Normal => 1.0,
            SpeedMode::FastForward2x => 2.0,
            SpeedMode::FastForward4x => 4.0,
            SpeedMode::SlowMotion => 0.5,
            SpeedMode::Paused => 0.0,
        }
    }
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        EmulatorConfig {
            video: VideoConfig {
                scale: 3,
                vsync: true,
                fps: 60,
                fullscreen: false,
            },
            audio: AudioConfig {
                enabled: true,
                volume: 0.5,
            },
            save_state: SaveStateConfig {
                slots: 10,
                auto_save_on_exit: false,
                save_directory: PathBuf::from("saves"),
            },
            screenshot: ScreenshotConfig {
                screenshot_directory: PathBuf::from("screenshots"),
                include_timestamp: true,
            },
            hotkeys: HotkeyConfig {
                quick_save: "F5".to_string(),
                quick_load: "F7".to_string(),
                reset: "F8".to_string(),
                screenshot: "F9".to_string(),
                fast_forward: "Tab".to_string(),
                pause: "P".to_string(),
            },
        }
    }
}

impl EmulatorConfig {
    /// Load configuration from file or create default
    ///
    /// If the configuration file doesn't exist, creates a default configuration
    /// and saves it to the file.
    ///
    /// # Returns
    ///
    /// The loaded or default configuration
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::emulator::EmulatorConfig;
    ///
    /// let config = EmulatorConfig::load_or_default();
    /// ```
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|_| {
            let config = Self::default();
            // Try to save the default config, but don't fail if we can't
            let _ = config.save();
            config
        })
    }

    /// Load configuration from file
    ///
    /// # Returns
    ///
    /// Result containing the configuration or an error
    pub fn load() -> Result<Self, io::Error> {
        let contents = fs::read_to_string(CONFIG_FILE)?;
        toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save configuration to file
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nes_rs::emulator::EmulatorConfig;
    ///
    /// let config = EmulatorConfig::default();
    /// config.save().expect("Failed to save configuration");
    /// ```
    pub fn save(&self) -> Result<(), io::Error> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(CONFIG_FILE, contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmulatorConfig::default();
        assert_eq!(config.video.scale, 3);
        assert_eq!(config.video.fps, 60);
        assert_eq!(config.audio.volume, 0.5);
        assert_eq!(config.save_state.slots, 10);
    }

    #[test]
    fn test_speed_mode_multiplier() {
        assert_eq!(SpeedMode::Normal.multiplier(), 1.0);
        assert_eq!(SpeedMode::FastForward2x.multiplier(), 2.0);
        assert_eq!(SpeedMode::FastForward4x.multiplier(), 4.0);
        assert_eq!(SpeedMode::SlowMotion.multiplier(), 0.5);
        assert_eq!(SpeedMode::Paused.multiplier(), 0.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = EmulatorConfig::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");
        let deserialized: EmulatorConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize");

        assert_eq!(config.video.scale, deserialized.video.scale);
        assert_eq!(config.audio.volume, deserialized.audio.volume);
    }
}
