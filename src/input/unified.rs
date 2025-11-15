// Unified input handler module
//
// This module provides a unified input handler that combines keyboard and gamepad
// inputs, allowing both to control the NES controllers simultaneously.

use super::{Controller, GamepadHandler, KeyboardHandler, Player};
use winit::keyboard::PhysicalKey;

/// Unified input handler that combines keyboard and gamepad inputs
///
/// This handler manages both keyboard and gamepad inputs, merging their states
/// to produce a single controller state for each player. This allows players to
/// use either keyboard or gamepad, or even both simultaneously.
pub struct UnifiedInputHandler {
    /// Keyboard input handler
    keyboard_handler: KeyboardHandler,
    /// Gamepad input handler
    gamepad_handler: GamepadHandler,
}

impl UnifiedInputHandler {
    /// Create a new unified input handler with default mappings
    ///
    /// # Returns
    /// A new UnifiedInputHandler with default keyboard and gamepad mappings
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::unified::UnifiedInputHandler;
    ///
    /// let handler = UnifiedInputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            keyboard_handler: KeyboardHandler::new(),
            gamepad_handler: GamepadHandler::new(),
        }
    }

    /// Create a unified input handler with custom handlers
    ///
    /// # Arguments
    /// * `keyboard_handler` - The keyboard input handler to use
    /// * `gamepad_handler` - The gamepad input handler to use
    ///
    /// # Returns
    /// A new UnifiedInputHandler with the specified handlers
    pub fn with_handlers(
        keyboard_handler: KeyboardHandler,
        gamepad_handler: GamepadHandler,
    ) -> Self {
        Self {
            keyboard_handler,
            gamepad_handler,
        }
    }

    /// Handle a keyboard key press event
    ///
    /// # Arguments
    /// * `physical_key` - The physical key that was pressed
    pub fn handle_key_press(&mut self, physical_key: PhysicalKey) {
        self.keyboard_handler.handle_key_press(physical_key);
    }

    /// Handle a keyboard key release event
    ///
    /// # Arguments
    /// * `physical_key` - The physical key that was released
    pub fn handle_key_release(&mut self, physical_key: PhysicalKey) {
        self.keyboard_handler.handle_key_release(physical_key);
    }

    /// Update gamepad states by processing pending events
    ///
    /// This should be called regularly (e.g., in the event loop)
    pub fn update_gamepads(&mut self) {
        self.gamepad_handler.update();
    }

    /// Get the combined controller state for a player
    ///
    /// This merges the keyboard and gamepad states using OR logic:
    /// if either input source indicates a button is pressed, it will
    /// be considered pressed in the final state.
    ///
    /// # Arguments
    /// * `player` - Which player's controller to get
    ///
    /// # Returns
    /// A Controller with button states merged from keyboard and gamepad
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::unified::UnifiedInputHandler;
    /// use nes_rs::input::Player;
    ///
    /// let mut handler = UnifiedInputHandler::new();
    /// handler.update_gamepads();
    /// let controller = handler.get_controller_state(Player::One);
    /// ```
    pub fn get_controller_state(&self, player: Player) -> Controller {
        let keyboard_state = self.keyboard_handler.get_controller_state(player);
        let gamepad_state = self.gamepad_handler.get_controller_state(player);

        // Merge states using OR logic (if either is pressed, button is pressed)
        Controller {
            button_a: keyboard_state.button_a || gamepad_state.button_a,
            button_b: keyboard_state.button_b || gamepad_state.button_b,
            select: keyboard_state.select || gamepad_state.select,
            start: keyboard_state.start || gamepad_state.start,
            up: keyboard_state.up || gamepad_state.up,
            down: keyboard_state.down || gamepad_state.down,
            left: keyboard_state.left || gamepad_state.left,
            right: keyboard_state.right || gamepad_state.right,
        }
    }

    /// Get a reference to the keyboard handler
    pub fn keyboard_handler(&self) -> &KeyboardHandler {
        &self.keyboard_handler
    }

    /// Get a mutable reference to the keyboard handler
    pub fn keyboard_handler_mut(&mut self) -> &mut KeyboardHandler {
        &mut self.keyboard_handler
    }

    /// Get a reference to the gamepad handler
    pub fn gamepad_handler(&self) -> &GamepadHandler {
        &self.gamepad_handler
    }

    /// Get a mutable reference to the gamepad handler
    pub fn gamepad_handler_mut(&mut self) -> &mut GamepadHandler {
        &mut self.gamepad_handler
    }

    /// List all connected gamepads with their assignments
    pub fn list_gamepads(&self) -> Vec<(usize, String, Option<Player>)> {
        self.gamepad_handler.list_gamepads()
    }

    /// Manually assign a gamepad to a player
    ///
    /// # Arguments
    /// * `gamepad_id` - The gamepad ID to assign
    /// * `player` - Which player to assign to
    pub fn assign_gamepad(&mut self, gamepad_id: usize, player: Player) {
        self.gamepad_handler.assign_gamepad(gamepad_id, player);
    }
}

impl Default for UnifiedInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;

    #[test]
    fn test_unified_handler_initialization() {
        let handler = UnifiedInputHandler::new();
        let controller = handler.get_controller_state(Player::One);

        // All buttons should be released initially
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
    fn test_unified_handler_default() {
        let handler = UnifiedInputHandler::default();
        let controller = handler.get_controller_state(Player::One);
        assert!(!controller.button_a);
    }

    #[test]
    fn test_keyboard_input() {
        let mut handler = UnifiedInputHandler::new();
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX)); // Player 1 A button

        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
        assert!(!controller.button_b);
    }

    #[test]
    fn test_merged_state() {
        let mut handler = UnifiedInputHandler::new();

        // Press A button on keyboard
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX));

        // The merged state should show A pressed
        let controller = handler.get_controller_state(Player::One);
        assert!(controller.button_a);
    }

    #[test]
    fn test_both_players_independent() {
        let mut handler = UnifiedInputHandler::new();

        // Player 1 presses A
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyX));

        // Player 2 presses B
        handler.handle_key_press(PhysicalKey::Code(KeyCode::KeyJ));

        let controller1 = handler.get_controller_state(Player::One);
        let controller2 = handler.get_controller_state(Player::Two);

        assert!(controller1.button_a);
        assert!(!controller1.button_b);

        assert!(!controller2.button_a);
        assert!(controller2.button_b);
    }

    #[test]
    fn test_handler_accessors() {
        let mut handler = UnifiedInputHandler::new();

        // Test keyboard handler access
        let _kb = handler.keyboard_handler();
        let _kb_mut = handler.keyboard_handler_mut();

        // Test gamepad handler access
        let _gp = handler.gamepad_handler();
        let _gp_mut = handler.gamepad_handler_mut();
    }
}
