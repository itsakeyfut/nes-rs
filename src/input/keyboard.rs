// Keyboard input mapping module
//
// This module provides keyboard-to-controller mapping for NES emulation.
// It supports both Player 1 and Player 2 with customizable key bindings.

use super::Controller;
use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Represents which player's controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    /// Player 1
    One,
    /// Player 2
    Two,
}

/// NES controller button enum for mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// A button
    A,
    /// B button
    B,
    /// Select button
    Select,
    /// Start button
    Start,
    /// Up on D-pad
    Up,
    /// Down on D-pad
    Down,
    /// Left on D-pad
    Left,
    /// Right on D-pad
    Right,
}

/// Keyboard mapping configuration for a single player
#[derive(Debug, Clone)]
pub struct KeyboardMapping {
    /// Key for A button
    pub button_a: KeyCode,
    /// Key for B button
    pub button_b: KeyCode,
    /// Key for Select button
    pub select: KeyCode,
    /// Key for Start button
    pub start: KeyCode,
    /// Key for Up on D-pad
    pub up: KeyCode,
    /// Key for Down on D-pad
    pub down: KeyCode,
    /// Key for Left on D-pad
    pub left: KeyCode,
    /// Key for Right on D-pad
    pub right: KeyCode,
}

impl KeyboardMapping {
    /// Create default keyboard mapping for Player 1
    ///
    /// # Default Mappings
    /// - Arrow keys: D-pad
    /// - X: A button
    /// - Z: B button
    /// - Enter: Start
    /// - Right Shift: Select
    ///
    /// # Returns
    /// A new KeyboardMapping for Player 1
    pub fn player1_default() -> Self {
        Self {
            button_a: KeyCode::KeyX,
            button_b: KeyCode::KeyZ,
            select: KeyCode::ShiftRight,
            start: KeyCode::Enter,
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
        }
    }

    /// Create default keyboard mapping for Player 2
    ///
    /// # Default Mappings
    /// - WASD: D-pad
    /// - K: A button
    /// - J: B button
    /// - Y: Start
    /// - U: Select
    ///
    /// # Returns
    /// A new KeyboardMapping for Player 2
    pub fn player2_default() -> Self {
        Self {
            button_a: KeyCode::KeyK,
            button_b: KeyCode::KeyJ,
            select: KeyCode::KeyU,
            start: KeyCode::KeyY,
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
        }
    }

    /// Get the button for a given key code
    ///
    /// # Arguments
    /// * `key` - The key code to check
    ///
    /// # Returns
    /// Some(Button) if the key is mapped to a button, None otherwise
    fn get_button(&self, key: KeyCode) -> Option<Button> {
        if key == self.button_a {
            Some(Button::A)
        } else if key == self.button_b {
            Some(Button::B)
        } else if key == self.select {
            Some(Button::Select)
        } else if key == self.start {
            Some(Button::Start)
        } else if key == self.up {
            Some(Button::Up)
        } else if key == self.down {
            Some(Button::Down)
        } else if key == self.left {
            Some(Button::Left)
        } else if key == self.right {
            Some(Button::Right)
        } else {
            None
        }
    }
}

/// Keyboard input handler for NES controllers
///
/// Manages keyboard state and converts it to NES controller state.
/// Supports simultaneous key presses and both players.
pub struct KeyboardHandler {
    /// Keyboard mapping for Player 1
    player1_mapping: KeyboardMapping,
    /// Keyboard mapping for Player 2
    player2_mapping: KeyboardMapping,
    /// Set of currently pressed keys
    pressed_keys: HashSet<KeyCode>,
}

impl KeyboardHandler {
    /// Create a new keyboard handler with default mappings
    ///
    /// # Returns
    /// A new KeyboardHandler with default key bindings for both players
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::keyboard::KeyboardHandler;
    ///
    /// let handler = KeyboardHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            player1_mapping: KeyboardMapping::player1_default(),
            player2_mapping: KeyboardMapping::player2_default(),
            pressed_keys: HashSet::new(),
        }
    }

    /// Create a keyboard handler with custom mappings
    ///
    /// # Arguments
    /// * `player1_mapping` - Keyboard mapping for Player 1
    /// * `player2_mapping` - Keyboard mapping for Player 2
    ///
    /// # Returns
    /// A new KeyboardHandler with the specified mappings
    pub fn with_mappings(
        player1_mapping: KeyboardMapping,
        player2_mapping: KeyboardMapping,
    ) -> Self {
        Self {
            player1_mapping,
            player2_mapping,
            pressed_keys: HashSet::new(),
        }
    }

    /// Handle a key press event
    ///
    /// # Arguments
    /// * `physical_key` - The physical key that was pressed
    pub fn handle_key_press(&mut self, physical_key: PhysicalKey) {
        if let PhysicalKey::Code(key_code) = physical_key {
            self.pressed_keys.insert(key_code);
        }
    }

    /// Handle a key release event
    ///
    /// # Arguments
    /// * `physical_key` - The physical key that was released
    pub fn handle_key_release(&mut self, physical_key: PhysicalKey) {
        if let PhysicalKey::Code(key_code) = physical_key {
            self.pressed_keys.remove(&key_code);
        }
    }

    /// Check if a button is pressed for a given player
    ///
    /// # Arguments
    /// * `player` - Which player to check
    /// * `button` - Which button to check
    ///
    /// # Returns
    /// true if the button is pressed, false otherwise
    fn is_button_pressed(&self, player: Player, button: Button) -> bool {
        let mapping = match player {
            Player::One => &self.player1_mapping,
            Player::Two => &self.player2_mapping,
        };

        self.pressed_keys.iter().any(|&key| {
            if let Some(mapped_button) = mapping.get_button(key) {
                mapped_button == button
            } else {
                false
            }
        })
    }

    /// Get the current controller state for a player
    ///
    /// # Arguments
    /// * `player` - Which player's controller to get
    ///
    /// # Returns
    /// A Controller with button states based on currently pressed keys
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::keyboard::{KeyboardHandler, Player};
    ///
    /// let handler = KeyboardHandler::new();
    /// let controller = handler.get_controller_state(Player::One);
    /// ```
    pub fn get_controller_state(&self, player: Player) -> Controller {
        Controller {
            button_a: self.is_button_pressed(player, Button::A),
            button_b: self.is_button_pressed(player, Button::B),
            select: self.is_button_pressed(player, Button::Select),
            start: self.is_button_pressed(player, Button::Start),
            up: self.is_button_pressed(player, Button::Up),
            down: self.is_button_pressed(player, Button::Down),
            left: self.is_button_pressed(player, Button::Left),
            right: self.is_button_pressed(player, Button::Right),
        }
    }

    /// Get keyboard mapping for Player 1
    pub fn player1_mapping(&self) -> &KeyboardMapping {
        &self.player1_mapping
    }

    /// Get keyboard mapping for Player 2
    pub fn player2_mapping(&self) -> &KeyboardMapping {
        &self.player2_mapping
    }

    /// Set keyboard mapping for Player 1
    pub fn set_player1_mapping(&mut self, mapping: KeyboardMapping) {
        self.player1_mapping = mapping;
    }

    /// Set keyboard mapping for Player 2
    pub fn set_player2_mapping(&mut self, mapping: KeyboardMapping) {
        self.player2_mapping = mapping;
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_mapping_player1_default() {
        let mapping = KeyboardMapping::player1_default();
        assert_eq!(mapping.button_a, KeyCode::KeyX);
        assert_eq!(mapping.button_b, KeyCode::KeyZ);
        assert_eq!(mapping.select, KeyCode::ShiftRight);
        assert_eq!(mapping.start, KeyCode::Enter);
        assert_eq!(mapping.up, KeyCode::ArrowUp);
        assert_eq!(mapping.down, KeyCode::ArrowDown);
        assert_eq!(mapping.left, KeyCode::ArrowLeft);
        assert_eq!(mapping.right, KeyCode::ArrowRight);
    }

    #[test]
    fn test_keyboard_mapping_player2_default() {
        let mapping = KeyboardMapping::player2_default();
        assert_eq!(mapping.button_a, KeyCode::KeyK);
        assert_eq!(mapping.button_b, KeyCode::KeyJ);
        assert_eq!(mapping.select, KeyCode::KeyU);
        assert_eq!(mapping.start, KeyCode::KeyY);
        assert_eq!(mapping.up, KeyCode::KeyW);
        assert_eq!(mapping.down, KeyCode::KeyS);
        assert_eq!(mapping.left, KeyCode::KeyA);
        assert_eq!(mapping.right, KeyCode::KeyD);
    }

    #[test]
    fn test_keyboard_mapping_get_button() {
        let mapping = KeyboardMapping::player1_default();
        assert_eq!(mapping.get_button(KeyCode::KeyX), Some(Button::A));
        assert_eq!(mapping.get_button(KeyCode::KeyZ), Some(Button::B));
        assert_eq!(mapping.get_button(KeyCode::ArrowUp), Some(Button::Up));
        assert_eq!(mapping.get_button(KeyCode::KeyQ), None);
    }

    #[test]
    fn test_keyboard_handler_initialization() {
        let handler = KeyboardHandler::new();
        assert_eq!(handler.pressed_keys.len(), 0);
    }

    #[test]
    fn test_keyboard_handler_default() {
        let handler = KeyboardHandler::default();
        assert_eq!(handler.pressed_keys.len(), 0);
    }

    #[test]
    fn test_handle_key_press() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX));
        assert!(handler.pressed_keys.contains(&KeyCode::KeyX));
    }

    #[test]
    fn test_handle_key_release() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX));
        handler.handle_key_release(PhysicalKey::Code(KeyCode::KeyX));
        assert!(!handler.pressed_keys.contains(&KeyCode::KeyX));
    }

    #[test]
    fn test_get_controller_state_no_keys_pressed() {
        let handler = KeyboardHandler::new();
        let controller = handler.get_controller_state(Player::One);

        assert!(!controller.button_a);
        assert!(!controller.button_b);
        assert!(!controller.select);
        assert!(!controller.start);
        assert!(!controller.up);
        assert!(!controller.down);
        assert!(!controller.left);
        assert!(!controller.right);
    }

    #[test]
    fn test_get_controller_state_player1_a_button() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX)); // Player 1 A button

        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
        assert!(!controller.button_b);
    }

    #[test]
    fn test_get_controller_state_player1_multiple_buttons() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX)); // A
        handler.handle_key_press(PhysicalKey::Code(KeyCode::ArrowUp)); // Up

        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
        assert!(controller.up);
        assert!(!controller.button_b);
    }

    #[test]
    fn test_get_controller_state_player2() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyK)); // Player 2 A button
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyW)); // Player 2 Up

        let controller = handler.get_controller_state(Player::Two);
        assert!(controller.button_a);
        assert!(controller.up);
        assert!(!controller.button_b);
    }

    #[test]
    fn test_both_players_independent() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX)); // Player 1 A
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyK)); // Player 2 A

        let controller1 = handler.get_controller_state(Player::One);
        let controller2 = handler.get_controller_state(Player::Two);

        assert!(controller1.button_a);
        assert!(controller2.button_a);
    }

    #[test]
    fn test_simultaneous_key_presses() {
        let mut handler = KeyboardHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX)); // A
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyZ)); // B
        handler.handle_key_press(PhysicalKey::Code(KeyCode::ArrowUp)); // Up
        handler.handle_key_press(PhysicalKey::Code(KeyCode::ArrowRight)); // Right

        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
        assert!(controller.button_b);
        assert!(controller.up);
        assert!(controller.right);
    }

    #[test]
    fn test_custom_mapping() {
        let custom_mapping = KeyboardMapping {
            button_a: KeyCode::Space,
            button_b: KeyCode::ControlLeft,
            select: KeyCode::Backspace,
            start: KeyCode::Escape,
            up: KeyCode::KeyI,
            down: KeyCode::KeyK,
            left: KeyCode::KeyJ,
            right: KeyCode::KeyL,
        };

        let mut handler =
            KeyboardHandler::with_mappings(custom_mapping, KeyboardMapping::player2_default());
        handler.handle_key_press(PhysicalKey::Code(KeyCode::Space));
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyI));

        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
        assert!(controller.up);
    }

    #[test]
    fn test_set_player_mappings() {
        let mut handler = KeyboardHandler::new();
        let custom_mapping = KeyboardMapping {
            button_a: KeyCode::Space,
            button_b: KeyCode::ControlLeft,
            select: KeyCode::Backspace,
            start: KeyCode::Escape,
            up: KeyCode::KeyI,
            down: KeyCode::KeyK,
            left: KeyCode::KeyJ,
            right: KeyCode::KeyL,
        };

        handler.set_player1_mapping(custom_mapping.clone());
        assert_eq!(handler.player1_mapping().button_a, KeyCode::Space);

        handler.set_player2_mapping(custom_mapping.clone());
        assert_eq!(handler.player2_mapping().button_a, KeyCode::Space);
    }
}
