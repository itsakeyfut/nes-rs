// Input configuration module
//
// This module provides functionality to save and load input configurations
// (keyboard and gamepad mappings) to/from TOML files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use winit::keyboard::KeyCode;

/// Serializable keyboard button mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardMappingConfig {
    /// Key for A button (as string, e.g., "KeyX")
    pub button_a: String,
    /// Key for B button
    pub button_b: String,
    /// Key for Select button
    pub select: String,
    /// Key for Start button
    pub start: String,
    /// Key for Up on D-pad
    pub up: String,
    /// Key for Down on D-pad
    pub down: String,
    /// Key for Left on D-pad
    pub left: String,
    /// Key for Right on D-pad
    pub right: String,
}

impl KeyboardMappingConfig {
    /// Create default keyboard mapping for Player 1
    pub fn player1_default() -> Self {
        Self {
            button_a: "KeyX".to_string(),
            button_b: "KeyZ".to_string(),
            select: "ShiftRight".to_string(),
            start: "Enter".to_string(),
            up: "ArrowUp".to_string(),
            down: "ArrowDown".to_string(),
            left: "ArrowLeft".to_string(),
            right: "ArrowRight".to_string(),
        }
    }

    /// Create default keyboard mapping for Player 2
    pub fn player2_default() -> Self {
        Self {
            button_a: "KeyK".to_string(),
            button_b: "KeyJ".to_string(),
            select: "KeyU".to_string(),
            start: "KeyY".to_string(),
            up: "KeyW".to_string(),
            down: "KeyS".to_string(),
            left: "KeyA".to_string(),
            right: "KeyD".to_string(),
        }
    }

    /// Convert to runtime KeyboardMapping
    ///
    /// # Returns
    /// Result containing KeyboardMapping or error message
    pub fn to_keyboard_mapping(&self) -> Result<super::KeyboardMapping, String> {
        Ok(super::KeyboardMapping {
            button_a: string_to_keycode(&self.button_a)?,
            button_b: string_to_keycode(&self.button_b)?,
            select: string_to_keycode(&self.select)?,
            start: string_to_keycode(&self.start)?,
            up: string_to_keycode(&self.up)?,
            down: string_to_keycode(&self.down)?,
            left: string_to_keycode(&self.left)?,
            right: string_to_keycode(&self.right)?,
        })
    }

    /// Create from runtime KeyboardMapping
    pub fn from_keyboard_mapping(mapping: &super::KeyboardMapping) -> Self {
        Self {
            button_a: keycode_to_string(mapping.button_a),
            button_b: keycode_to_string(mapping.button_b),
            select: keycode_to_string(mapping.select),
            start: keycode_to_string(mapping.start),
            up: keycode_to_string(mapping.up),
            down: keycode_to_string(mapping.down),
            left: keycode_to_string(mapping.left),
            right: keycode_to_string(mapping.right),
        }
    }
}

/// Serializable gamepad button mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamepadMappingConfig {
    /// Button for A (as string, e.g., "East")
    pub button_a: String,
    /// Button for B
    pub button_b: String,
    /// Button for Select
    pub select: String,
    /// Button for Start
    pub start: String,
    /// Button for Up on D-pad
    pub up: String,
    /// Button for Down on D-pad
    pub down: String,
    /// Button for Left on D-pad
    pub left: String,
    /// Button for Right on D-pad
    pub right: String,
}

impl GamepadMappingConfig {
    /// Create default gamepad mapping
    pub fn default_mapping() -> Self {
        Self {
            button_a: "East".to_string(),
            button_b: "South".to_string(),
            select: "Select".to_string(),
            start: "Start".to_string(),
            up: "DPadUp".to_string(),
            down: "DPadDown".to_string(),
            left: "DPadLeft".to_string(),
            right: "DPadRight".to_string(),
        }
    }

    /// Convert to runtime GamepadMapping
    ///
    /// # Returns
    /// Result containing GamepadMapping or error message
    pub fn to_gamepad_mapping(&self) -> Result<super::GamepadMapping, String> {
        Ok(super::GamepadMapping {
            button_a: string_to_gilrs_button(&self.button_a)?,
            button_b: string_to_gilrs_button(&self.button_b)?,
            select: string_to_gilrs_button(&self.select)?,
            start: string_to_gilrs_button(&self.start)?,
            up: string_to_gilrs_button(&self.up)?,
            down: string_to_gilrs_button(&self.down)?,
            left: string_to_gilrs_button(&self.left)?,
            right: string_to_gilrs_button(&self.right)?,
        })
    }

    /// Create from runtime GamepadMapping
    pub fn from_gamepad_mapping(mapping: &super::GamepadMapping) -> Self {
        Self {
            button_a: gilrs_button_to_string(mapping.button_a),
            button_b: gilrs_button_to_string(mapping.button_b),
            select: gilrs_button_to_string(mapping.select),
            start: gilrs_button_to_string(mapping.start),
            up: gilrs_button_to_string(mapping.up),
            down: gilrs_button_to_string(mapping.down),
            left: gilrs_button_to_string(mapping.left),
            right: gilrs_button_to_string(mapping.right),
        }
    }
}

/// Complete input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Keyboard mapping for Player 1
    pub keyboard_player1: KeyboardMappingConfig,
    /// Keyboard mapping for Player 2
    pub keyboard_player2: KeyboardMappingConfig,
    /// Gamepad mapping for Player 1
    pub gamepad_player1: GamepadMappingConfig,
    /// Gamepad mapping for Player 2
    pub gamepad_player2: GamepadMappingConfig,
}

impl InputConfig {
    /// Create a new input configuration with default mappings
    pub fn new() -> Self {
        Self {
            keyboard_player1: KeyboardMappingConfig::player1_default(),
            keyboard_player2: KeyboardMappingConfig::player2_default(),
            gamepad_player1: GamepadMappingConfig::default_mapping(),
            gamepad_player2: GamepadMappingConfig::default_mapping(),
        }
    }

    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// Result containing InputConfig or error message
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: InputConfig = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    /// Save configuration to a TOML file
    ///
    /// # Arguments
    /// * `path` - Path where the TOML configuration file will be saved
    ///
    /// # Returns
    /// Result indicating success or error message
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Try to load configuration from file, or create default if it doesn't exist
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// InputConfig (either loaded or default)
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load_from_file(&path).unwrap_or_else(|e| {
            eprintln!("Could not load config ({}), using defaults", e);
            let config = Self::new();
            // Try to save default config
            if let Err(e) = config.save_to_file(&path) {
                eprintln!("Warning: Could not save default config: {}", e);
            } else {
                println!("Created default configuration file");
            }
            config
        })
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert KeyCode to string representation
fn keycode_to_string(key: KeyCode) -> String {
    format!("{:?}", key)
}

/// Convert string to KeyCode
fn string_to_keycode(s: &str) -> Result<KeyCode, String> {
    // This is a simple implementation that handles common keys
    // For a complete implementation, you'd need to match all possible KeyCode variants
    match s {
        "KeyA" => Ok(KeyCode::KeyA),
        "KeyB" => Ok(KeyCode::KeyB),
        "KeyC" => Ok(KeyCode::KeyC),
        "KeyD" => Ok(KeyCode::KeyD),
        "KeyE" => Ok(KeyCode::KeyE),
        "KeyF" => Ok(KeyCode::KeyF),
        "KeyG" => Ok(KeyCode::KeyG),
        "KeyH" => Ok(KeyCode::KeyH),
        "KeyI" => Ok(KeyCode::KeyI),
        "KeyJ" => Ok(KeyCode::KeyJ),
        "KeyK" => Ok(KeyCode::KeyK),
        "KeyL" => Ok(KeyCode::KeyL),
        "KeyM" => Ok(KeyCode::KeyM),
        "KeyN" => Ok(KeyCode::KeyN),
        "KeyO" => Ok(KeyCode::KeyO),
        "KeyP" => Ok(KeyCode::KeyP),
        "KeyQ" => Ok(KeyCode::KeyQ),
        "KeyR" => Ok(KeyCode::KeyR),
        "KeyS" => Ok(KeyCode::KeyS),
        "KeyT" => Ok(KeyCode::KeyT),
        "KeyU" => Ok(KeyCode::KeyU),
        "KeyV" => Ok(KeyCode::KeyV),
        "KeyW" => Ok(KeyCode::KeyW),
        "KeyX" => Ok(KeyCode::KeyX),
        "KeyY" => Ok(KeyCode::KeyY),
        "KeyZ" => Ok(KeyCode::KeyZ),
        "ArrowUp" => Ok(KeyCode::ArrowUp),
        "ArrowDown" => Ok(KeyCode::ArrowDown),
        "ArrowLeft" => Ok(KeyCode::ArrowLeft),
        "ArrowRight" => Ok(KeyCode::ArrowRight),
        "Enter" => Ok(KeyCode::Enter),
        "Space" => Ok(KeyCode::Space),
        "Escape" => Ok(KeyCode::Escape),
        "Backspace" => Ok(KeyCode::Backspace),
        "ShiftLeft" => Ok(KeyCode::ShiftLeft),
        "ShiftRight" => Ok(KeyCode::ShiftRight),
        "ControlLeft" => Ok(KeyCode::ControlLeft),
        "ControlRight" => Ok(KeyCode::ControlRight),
        "AltLeft" => Ok(KeyCode::AltLeft),
        "AltRight" => Ok(KeyCode::AltRight),
        _ => Err(format!("Unknown key code: {}", s)),
    }
}

/// Convert gilrs::Button to string representation
fn gilrs_button_to_string(button: gilrs::Button) -> String {
    format!("{:?}", button)
}

/// Convert string to gilrs::Button
fn string_to_gilrs_button(s: &str) -> Result<gilrs::Button, String> {
    use gilrs::Button;

    match s {
        "South" => Ok(Button::South),
        "East" => Ok(Button::East),
        "North" => Ok(Button::North),
        "West" => Ok(Button::West),
        "C" => Ok(Button::C),
        "Z" => Ok(Button::Z),
        "LeftTrigger" => Ok(Button::LeftTrigger),
        "LeftTrigger2" => Ok(Button::LeftTrigger2),
        "RightTrigger" => Ok(Button::RightTrigger),
        "RightTrigger2" => Ok(Button::RightTrigger2),
        "Select" => Ok(Button::Select),
        "Start" => Ok(Button::Start),
        "Mode" => Ok(Button::Mode),
        "LeftThumb" => Ok(Button::LeftThumb),
        "RightThumb" => Ok(Button::RightThumb),
        "DPadUp" => Ok(Button::DPadUp),
        "DPadDown" => Ok(Button::DPadDown),
        "DPadLeft" => Ok(Button::DPadLeft),
        "DPadRight" => Ok(Button::DPadRight),
        _ => Err(format!("Unknown gamepad button: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_config_defaults() {
        let config = KeyboardMappingConfig::player1_default();
        assert_eq!(config.button_a, "KeyX");
        assert_eq!(config.button_b, "KeyZ");
        assert_eq!(config.up, "ArrowUp");
    }

    #[test]
    fn test_gamepad_config_defaults() {
        let config = GamepadMappingConfig::default_mapping();
        assert_eq!(config.button_a, "East");
        assert_eq!(config.button_b, "South");
        assert_eq!(config.up, "DPadUp");
    }

    #[test]
    fn test_input_config_creation() {
        let config = InputConfig::new();
        assert_eq!(config.keyboard_player1.button_a, "KeyX");
        assert_eq!(config.gamepad_player1.button_a, "East");
    }

    #[test]
    fn test_keycode_conversion() {
        assert_eq!(keycode_to_string(KeyCode::KeyX), "KeyX");
        assert!(string_to_keycode("KeyX").is_ok());
        assert_eq!(string_to_keycode("KeyX").unwrap(), KeyCode::KeyX);
        assert!(string_to_keycode("InvalidKey").is_err());
    }

    #[test]
    fn test_gilrs_button_conversion() {
        use gilrs::Button;
        assert_eq!(gilrs_button_to_string(Button::East), "East");
        assert!(string_to_gilrs_button("East").is_ok());
        assert_eq!(string_to_gilrs_button("East").unwrap(), Button::East);
        assert!(string_to_gilrs_button("InvalidButton").is_err());
    }

    #[test]
    fn test_keyboard_mapping_conversion() {
        let config = KeyboardMappingConfig::player1_default();
        let mapping = config.to_keyboard_mapping().unwrap();
        assert_eq!(mapping.button_a, KeyCode::KeyX);

        let config2 = KeyboardMappingConfig::from_keyboard_mapping(&mapping);
        assert_eq!(config2.button_a, "KeyX");
    }

    #[test]
    fn test_gamepad_mapping_conversion() {
        let config = GamepadMappingConfig::default_mapping();
        let mapping = config.to_gamepad_mapping().unwrap();
        assert_eq!(mapping.button_a, gilrs::Button::East);

        let config2 = GamepadMappingConfig::from_gamepad_mapping(&mapping);
        assert_eq!(config2.button_a, "East");
    }

    #[test]
    fn test_config_serialization() {
        let config = InputConfig::new();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("keyboard_player1"));
        assert!(toml_str.contains("gamepad_player1"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [keyboard_player1]
            button_a = "KeyX"
            button_b = "KeyZ"
            select = "ShiftRight"
            start = "Enter"
            up = "ArrowUp"
            down = "ArrowDown"
            left = "ArrowLeft"
            right = "ArrowRight"

            [keyboard_player2]
            button_a = "KeyK"
            button_b = "KeyJ"
            select = "KeyU"
            start = "KeyY"
            up = "KeyW"
            down = "KeyS"
            left = "KeyA"
            right = "KeyD"

            [gamepad_player1]
            button_a = "East"
            button_b = "South"
            select = "Select"
            start = "Start"
            up = "DPadUp"
            down = "DPadDown"
            left = "DPadLeft"
            right = "DPadRight"

            [gamepad_player2]
            button_a = "East"
            button_b = "South"
            select = "Select"
            start = "Start"
            up = "DPadUp"
            down = "DPadDown"
            left = "DPadLeft"
            right = "DPadRight"
        "#;

        let config: InputConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keyboard_player1.button_a, "KeyX");
        assert_eq!(config.gamepad_player1.button_a, "East");
    }
}
