// Gamepad input mapping module
//
// This module provides gamepad-to-controller mapping for NES emulation.
// It supports both Player 1 and Player 2 with customizable button bindings.

use super::{Button, Controller, Player};
use gilrs::{Button as GilrsButton, Event, EventType, Gilrs};
use std::collections::HashMap;

/// Gamepad mapping configuration for a single player
#[derive(Debug, Clone)]
pub struct GamepadMapping {
    /// Button for A
    pub button_a: GilrsButton,
    /// Button for B
    pub button_b: GilrsButton,
    /// Button for Select
    pub select: GilrsButton,
    /// Button for Start
    pub start: GilrsButton,
    /// Button for Up on D-pad
    pub up: GilrsButton,
    /// Button for Down on D-pad
    pub down: GilrsButton,
    /// Button for Left on D-pad
    pub left: GilrsButton,
    /// Button for Right on D-pad
    pub right: GilrsButton,
}

impl GamepadMapping {
    /// Create default gamepad mapping
    ///
    /// # Default Mappings (Standard Gamepad Layout)
    /// - D-pad: D-pad buttons
    /// - South button (A/Cross): B button
    /// - East button (B/Circle): A button
    /// - Start: Start
    /// - Select/Back: Select
    ///
    /// # Returns
    /// A new GamepadMapping with standard button layout
    pub fn default_mapping() -> Self {
        Self {
            button_a: GilrsButton::East,   // B/Circle on PlayStation, B on Xbox
            button_b: GilrsButton::South,  // A/Cross on PlayStation, A on Xbox
            select: GilrsButton::Select,   // Select/Back button
            start: GilrsButton::Start,     // Start button
            up: GilrsButton::DPadUp,       // D-pad up
            down: GilrsButton::DPadDown,   // D-pad down
            left: GilrsButton::DPadLeft,   // D-pad left
            right: GilrsButton::DPadRight, // D-pad right
        }
    }

    /// Get the NES button for a given gamepad button
    ///
    /// # Arguments
    /// * `button` - The gamepad button to check
    ///
    /// # Returns
    /// Some(Button) if the button is mapped to a NES button, None otherwise
    fn get_button(&self, button: GilrsButton) -> Option<Button> {
        if button == self.button_a {
            Some(Button::A)
        } else if button == self.button_b {
            Some(Button::B)
        } else if button == self.select {
            Some(Button::Select)
        } else if button == self.start {
            Some(Button::Start)
        } else if button == self.up {
            Some(Button::Up)
        } else if button == self.down {
            Some(Button::Down)
        } else if button == self.left {
            Some(Button::Left)
        } else if button == self.right {
            Some(Button::Right)
        } else {
            None
        }
    }
}

/// Gamepad input handler for NES controllers
///
/// Manages gamepad state and converts it to NES controller state.
/// Supports multiple gamepads for Player 1 and Player 2.
pub struct GamepadHandler {
    /// Gilrs instance for gamepad events
    gilrs: Gilrs,
    /// Gamepad mapping for Player 1
    player1_mapping: GamepadMapping,
    /// Gamepad mapping for Player 2
    player2_mapping: GamepadMapping,
    /// Map of gamepad ID to player assignment
    gamepad_assignments: HashMap<usize, Player>,
    /// Current button states for Player 1
    player1_state: Controller,
    /// Current button states for Player 2
    player2_state: Controller,
}

impl GamepadHandler {
    /// Create a new gamepad handler with default mappings
    ///
    /// # Returns
    /// A new GamepadHandler with default button bindings for both players
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::gamepad::GamepadHandler;
    ///
    /// let handler = GamepadHandler::new();
    /// ```
    pub fn new() -> Self {
        let gilrs = Gilrs::new().unwrap_or_else(|e| {
            eprintln!("Failed to initialize gamepad support: {}", e);
            panic!("Gamepad initialization failed");
        });

        let mut handler = Self {
            gilrs,
            player1_mapping: GamepadMapping::default_mapping(),
            player2_mapping: GamepadMapping::default_mapping(),
            gamepad_assignments: HashMap::new(),
            player1_state: Controller::new(),
            player2_state: Controller::new(),
        };

        // Auto-assign connected gamepads
        handler.auto_assign_gamepads();

        handler
    }

    /// Create a gamepad handler with custom mappings
    ///
    /// # Arguments
    /// * `player1_mapping` - Gamepad mapping for Player 1
    /// * `player2_mapping` - Gamepad mapping for Player 2
    ///
    /// # Returns
    /// A new GamepadHandler with the specified mappings
    pub fn with_mappings(player1_mapping: GamepadMapping, player2_mapping: GamepadMapping) -> Self {
        let gilrs = Gilrs::new().unwrap_or_else(|e| {
            eprintln!("Failed to initialize gamepad support: {}", e);
            panic!("Gamepad initialization failed");
        });

        let mut handler = Self {
            gilrs,
            player1_mapping,
            player2_mapping,
            gamepad_assignments: HashMap::new(),
            player1_state: Controller::new(),
            player2_state: Controller::new(),
        };

        // Auto-assign connected gamepads
        handler.auto_assign_gamepads();

        handler
    }

    /// Auto-assign connected gamepads to players
    ///
    /// Assigns the first gamepad to Player 1, second to Player 2
    fn auto_assign_gamepads(&mut self) {
        let mut player_index = 0;

        for (id, gamepad) in self.gilrs.gamepads() {
            if gamepad.is_connected() {
                let player = if player_index == 0 {
                    Player::One
                } else {
                    Player::Two
                };

                self.gamepad_assignments.insert(id.into(), player);
                println!(
                    "Gamepad '{}' (ID: {}) assigned to {:?}",
                    gamepad.name(),
                    id,
                    player
                );

                player_index += 1;
                if player_index >= 2 {
                    break; // Only support 2 players
                }
            }
        }

        if self.gamepad_assignments.is_empty() {
            println!(
                "No gamepads detected. Gamepad support is available when you connect a controller."
            );
        }
    }

    /// Manually assign a gamepad to a player
    ///
    /// # Arguments
    /// * `gamepad_id` - The gamepad ID to assign
    /// * `player` - Which player to assign to
    pub fn assign_gamepad(&mut self, gamepad_id: usize, player: Player) {
        self.gamepad_assignments.insert(gamepad_id, player);
    }

    /// Process pending gamepad events
    ///
    /// This should be called regularly (e.g., in the event loop) to update controller states
    pub fn update(&mut self) {
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            let gamepad_id: usize = id.into();

            // Check if this gamepad is assigned to a player
            if let Some(&player) = self.gamepad_assignments.get(&gamepad_id) {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        self.handle_button_press(player, button);
                    }
                    EventType::ButtonReleased(button, _) => {
                        self.handle_button_release(player, button);
                    }
                    EventType::Connected => {
                        println!("Gamepad {} connected", gamepad_id);
                    }
                    EventType::Disconnected => {
                        println!("Gamepad {} disconnected", gamepad_id);
                        // Clear button states for this player
                        match player {
                            Player::One => self.player1_state = Controller::new(),
                            Player::Two => self.player2_state = Controller::new(),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle a button press event
    ///
    /// # Arguments
    /// * `player` - Which player's gamepad triggered the event
    /// * `button` - The button that was pressed
    fn handle_button_press(&mut self, player: Player, button: GilrsButton) {
        let mapping = match player {
            Player::One => &self.player1_mapping,
            Player::Two => &self.player2_mapping,
        };

        if let Some(nes_button) = mapping.get_button(button) {
            let state = match player {
                Player::One => &mut self.player1_state,
                Player::Two => &mut self.player2_state,
            };

            match nes_button {
                Button::A => state.button_a = true,
                Button::B => state.button_b = true,
                Button::Select => state.select = true,
                Button::Start => state.start = true,
                Button::Up => state.up = true,
                Button::Down => state.down = true,
                Button::Left => state.left = true,
                Button::Right => state.right = true,
            }
        }
    }

    /// Handle a button release event
    ///
    /// # Arguments
    /// * `player` - Which player's gamepad triggered the event
    /// * `button` - The button that was released
    fn handle_button_release(&mut self, player: Player, button: GilrsButton) {
        let mapping = match player {
            Player::One => &self.player1_mapping,
            Player::Two => &self.player2_mapping,
        };

        if let Some(nes_button) = mapping.get_button(button) {
            let state = match player {
                Player::One => &mut self.player1_state,
                Player::Two => &mut self.player2_state,
            };

            match nes_button {
                Button::A => state.button_a = false,
                Button::B => state.button_b = false,
                Button::Select => state.select = false,
                Button::Start => state.start = false,
                Button::Up => state.up = false,
                Button::Down => state.down = false,
                Button::Left => state.left = false,
                Button::Right => state.right = false,
            }
        }
    }

    /// Get the current controller state for a player
    ///
    /// # Arguments
    /// * `player` - Which player's controller to get
    ///
    /// # Returns
    /// A Controller with button states based on gamepad input
    ///
    /// # Example
    /// ```
    /// use nes_rs::input::gamepad::GamepadHandler;
    /// use nes_rs::input::Player;
    ///
    /// let mut handler = GamepadHandler::new();
    /// handler.update(); // Process events
    /// let controller = handler.get_controller_state(Player::One);
    /// ```
    pub fn get_controller_state(&self, player: Player) -> Controller {
        match player {
            Player::One => self.player1_state,
            Player::Two => self.player2_state,
        }
    }

    /// Get gamepad mapping for Player 1
    pub fn player1_mapping(&self) -> &GamepadMapping {
        &self.player1_mapping
    }

    /// Get gamepad mapping for Player 2
    pub fn player2_mapping(&self) -> &GamepadMapping {
        &self.player2_mapping
    }

    /// Set gamepad mapping for Player 1
    pub fn set_player1_mapping(&mut self, mapping: GamepadMapping) {
        self.player1_mapping = mapping;
    }

    /// Set gamepad mapping for Player 2
    pub fn set_player2_mapping(&mut self, mapping: GamepadMapping) {
        self.player2_mapping = mapping;
    }

    /// Get list of connected gamepads with their assignments
    pub fn list_gamepads(&self) -> Vec<(usize, String, Option<Player>)> {
        self.gilrs
            .gamepads()
            .filter(|(_, gamepad)| gamepad.is_connected())
            .map(|(id, gamepad)| {
                let gamepad_id: usize = id.into();
                let name = gamepad.name().to_string();
                let player = self.gamepad_assignments.get(&gamepad_id).copied();
                (gamepad_id, name, player)
            })
            .collect()
    }
}

impl Default for GamepadHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_mapping_default() {
        let mapping = GamepadMapping::default_mapping();
        assert_eq!(mapping.button_a, GilrsButton::East);
        assert_eq!(mapping.button_b, GilrsButton::South);
        assert_eq!(mapping.select, GilrsButton::Select);
        assert_eq!(mapping.start, GilrsButton::Start);
        assert_eq!(mapping.up, GilrsButton::DPadUp);
        assert_eq!(mapping.down, GilrsButton::DPadDown);
        assert_eq!(mapping.left, GilrsButton::DPadLeft);
        assert_eq!(mapping.right, GilrsButton::DPadRight);
    }

    #[test]
    fn test_gamepad_mapping_get_button() {
        let mapping = GamepadMapping::default_mapping();
        assert_eq!(mapping.get_button(GilrsButton::East), Some(Button::A));
        assert_eq!(mapping.get_button(GilrsButton::South), Some(Button::B));
        assert_eq!(mapping.get_button(GilrsButton::DPadUp), Some(Button::Up));
        assert_eq!(mapping.get_button(GilrsButton::North), None);
    }

    #[test]
    fn test_gamepad_handler_initialization() {
        let handler = GamepadHandler::new();
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
    fn test_gamepad_handler_default() {
        let handler = GamepadHandler::default();
        let controller = handler.get_controller_state(Player::One);
        assert!(!controller.button_a);
    }

    #[test]
    fn test_set_player_mappings() {
        let mut handler = GamepadHandler::new();
        let custom_mapping = GamepadMapping::default_mapping();

        handler.set_player1_mapping(custom_mapping.clone());
        assert_eq!(handler.player1_mapping().button_a, GilrsButton::East);

        handler.set_player2_mapping(custom_mapping.clone());
        assert_eq!(handler.player2_mapping().button_a, GilrsButton::East);
    }

    #[test]
    fn test_manual_gamepad_assignment() {
        let mut handler = GamepadHandler::new();
        handler.assign_gamepad(0, Player::One);
        assert_eq!(handler.gamepad_assignments.get(&0), Some(&Player::One));
    }
}
