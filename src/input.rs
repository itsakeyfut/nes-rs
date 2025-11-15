// Input module - Controller input handling
//
// This module contains controller input processing for the NES standard controller.
//
// # Controller I/O Registers
//
// The NES has two controller ports mapped to CPU memory at $4016 and $4017.
//
// This is a complete implementation of the NES standard controller with support for
// strobe-based button state reading.
//
// ## Register Map
//
// | Address | Name          | Access | Description                     |
// |---------|---------------|--------|---------------------------------|
// | $4016   | Controller 1  | R/W    | Controller 1 data / Strobe      |
// | $4017   | Controller 2  | R      | Controller 2 data               |
//
// Note: $4017 is also used by APU for the Frame Counter (write-only).
// Reads from $4017 return controller 2 data, writes go to APU.
//
// ## Controller Reading Sequence
//
// 1. Write $01 to $4016 (start strobe)
// 2. Write $00 to $4016 (end strobe)
// 3. Read $4016 eight times for button states (Controller 1)
// 4. Read $4017 eight times for button states (Controller 2)
//
// Each read returns bit 0 = button state (1 = pressed, 0 = released)
// Reading order: A, B, Select, Start, Up, Down, Left, Right

pub mod keyboard;

use crate::bus::MemoryMappedDevice;

pub use keyboard::{Button, KeyboardHandler, KeyboardMapping, Player};

/// Controller button state structure
///
/// Represents the state of all 8 buttons on a standard NES controller.
#[derive(Debug, Clone, Copy)]
pub struct Controller {
    /// A button state
    pub button_a: bool,
    /// B button state
    pub button_b: bool,
    /// Select button state
    pub select: bool,
    /// Start button state
    pub start: bool,
    /// Up D-pad state
    pub up: bool,
    /// Down D-pad state
    pub down: bool,
    /// Left D-pad state
    pub left: bool,
    /// Right D-pad state
    pub right: bool,
}

impl Controller {
    /// Create a new controller instance with all buttons released
    ///
    /// # Returns
    ///
    /// A new Controller with all buttons in released state
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::input::Controller;
    ///
    /// let controller = Controller::new();
    /// assert!(!controller.button_a);
    /// ```
    pub fn new() -> Self {
        Controller {
            button_a: false,
            button_b: false,
            select: false,
            start: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    /// Get button state by index (0-7)
    ///
    /// # Arguments
    ///
    /// * `index` - Button index (0=A, 1=B, 2=Select, 3=Start, 4=Up, 5=Down, 6=Left, 7=Right)
    ///
    /// # Returns
    ///
    /// True if button is pressed, false otherwise
    fn get_button(&self, index: u8) -> bool {
        match index {
            0 => self.button_a,
            1 => self.button_b,
            2 => self.select,
            3 => self.start,
            4 => self.up,
            5 => self.down,
            6 => self.left,
            7 => self.right,
            _ => false,
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Controller I/O interface
///
/// This structure manages the state of both NES controllers and handles
/// the serial reading protocol.
///
/// Supports the standard NES controller strobe and serial read mechanism.
/// Button states can be updated via set_controller1 and set_controller2 methods.
pub struct ControllerIO {
    /// Controller 1 state
    controller1: Controller,

    /// Controller 2 state
    controller2: Controller,

    /// Strobe state
    ///
    /// When true, controller is continuously reloading button state.
    /// When false, controller shifts out one bit per read.
    strobe: bool,

    /// Current button index for Controller 1 (0-7)
    ///
    /// Tracks which button will be returned on next read.
    button_index1: u8,

    /// Current button index for Controller 2 (0-7)
    ///
    /// Tracks which button will be returned on next read.
    button_index2: u8,
}

impl ControllerIO {
    /// Create a new controller I/O interface
    ///
    /// # Returns
    ///
    /// A new ControllerIO with both controllers in default state
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::input::ControllerIO;
    ///
    /// let controller_io = ControllerIO::new();
    /// ```
    pub fn new() -> Self {
        ControllerIO {
            controller1: Controller::new(),
            controller2: Controller::new(),
            strobe: false,
            button_index1: 0,
            button_index2: 0,
        }
    }

    /// Reset the controller I/O to default state
    pub fn reset(&mut self) {
        self.strobe = false;
        self.button_index1 = 0;
        self.button_index2 = 0;
    }

    /// Read from controller 1 ($4016)
    ///
    /// Returns the current button state bit.
    /// When strobe is off, advances to next button.
    ///
    /// # Returns
    ///
    /// Bit 0: Current button state (1 = pressed, 0 = released)
    /// Bits 1-7: Open bus (stub: return 0)
    fn read_controller1(&mut self) -> u8 {
        if self.strobe {
            // While strobing, always return button A state
            if self.controller1.button_a {
                0x01
            } else {
                0x00
            }
        } else {
            // Return current button state and advance
            let button_state = if self.button_index1 < 8 {
                if self.controller1.get_button(self.button_index1) {
                    0x01
                } else {
                    0x00
                }
            } else {
                // After 8 reads, return 1 (signature bit)
                0x01
            };

            // Advance button index (clamp at 8 to prevent wraparound)
            if self.button_index1 < 8 {
                self.button_index1 += 1;
            }

            button_state
        }
    }

    /// Read from controller 2 ($4017)
    ///
    /// Returns the current button state bit.
    /// When strobe is off, advances to next button.
    ///
    /// # Returns
    ///
    /// Bit 0: Current button state (1 = pressed, 0 = released)
    /// Bits 1-7: Open bus (stub: return 0)
    fn read_controller2(&mut self) -> u8 {
        if self.strobe {
            // While strobing, always return button A state
            if self.controller2.button_a {
                0x01
            } else {
                0x00
            }
        } else {
            // Return current button state and advance
            let button_state = if self.button_index2 < 8 {
                if self.controller2.get_button(self.button_index2) {
                    0x01
                } else {
                    0x00
                }
            } else {
                // After 8 reads, return 1 (signature bit)
                0x01
            };

            // Advance button index (clamp at 8 to prevent wraparound)
            if self.button_index2 < 8 {
                self.button_index2 += 1;
            }

            button_state
        }
    }

    /// Write to controller strobe ($4016)
    ///
    /// # Arguments
    ///
    /// * `data` - Bit 0: Strobe state (1 = start strobe, 0 = end strobe)
    ///
    /// # Behavior
    ///
    /// - Writing 1 continuously reloads button states
    /// - Writing 0 ends strobe and resets button index to 0
    fn write_strobe(&mut self, data: u8) {
        let new_strobe = (data & 0x01) != 0;

        // Detect strobe going from high to low (end of strobe)
        if self.strobe && !new_strobe {
            // Reset button indices when strobe ends
            self.button_index1 = 0;
            self.button_index2 = 0;
        }

        self.strobe = new_strobe;
    }

    /// Update controller 1 state
    ///
    /// Used to update button states from input events (e.g., keyboard, gamepad).
    ///
    /// # Arguments
    ///
    /// * `controller` - The new controller state
    pub fn set_controller1(&mut self, controller: Controller) {
        self.controller1 = controller;
    }

    /// Update controller 2 state
    ///
    /// Used to update button states from input events (e.g., keyboard, gamepad).
    ///
    /// # Arguments
    ///
    /// * `controller` - The new controller state
    pub fn set_controller2(&mut self, controller: Controller) {
        self.controller2 = controller;
    }
}

impl MemoryMappedDevice for ControllerIO {
    /// Read from controller I/O
    ///
    /// # Arguments
    ///
    /// * `addr` - The address ($4016 or $4017)
    ///
    /// # Returns
    ///
    /// Controller button state (bit 0)
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4016 => self.read_controller1(),
            0x4017 => self.read_controller2(),
            _ => 0,
        }
    }

    /// Write to controller I/O
    ///
    /// # Arguments
    ///
    /// * `addr` - The address ($4016 for strobe)
    /// * `data` - The value to write
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4016 => self.write_strobe(data),
            // $4017 writes go to APU frame counter, not handled here
            0x4017 => {}
            _ => {}
        }
    }
}

impl Default for ControllerIO {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Controller Tests
    // ========================================

    #[test]
    fn test_controller_initialization() {
        let controller = Controller::new();
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
    fn test_controller_default() {
        let controller = Controller::default();
        assert!(!controller.button_a);
    }

    #[test]
    fn test_controller_get_button() {
        let mut controller = Controller::new();
        controller.button_a = true;
        controller.start = true;

        assert!(controller.get_button(0)); // A
        assert!(!controller.get_button(1)); // B
        assert!(controller.get_button(3)); // Start
    }

    // ========================================
    // Controller I/O Tests
    // ========================================

    #[test]
    fn test_controller_io_initialization() {
        let controller_io = ControllerIO::new();
        assert!(!controller_io.strobe);
        assert_eq!(controller_io.button_index1, 0);
        assert_eq!(controller_io.button_index2, 0);
    }

    #[test]
    fn test_controller_io_default() {
        let controller_io = ControllerIO::default();
        assert!(!controller_io.strobe);
    }

    #[test]
    fn test_controller_io_reset() {
        let mut controller_io = ControllerIO::new();
        controller_io.strobe = true;
        controller_io.button_index1 = 5;

        controller_io.reset();

        assert!(!controller_io.strobe);
        assert_eq!(controller_io.button_index1, 0);
    }

    // ========================================
    // Strobe Tests
    // ========================================

    #[test]
    fn test_write_strobe_on() {
        let mut controller_io = ControllerIO::new();
        controller_io.write(0x4016, 0x01);

        assert!(controller_io.strobe);
    }

    #[test]
    fn test_write_strobe_off() {
        let mut controller_io = ControllerIO::new();
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        assert!(!controller_io.strobe);
    }

    #[test]
    fn test_strobe_resets_button_index() {
        let mut controller_io = ControllerIO::new();

        // Read a few buttons
        controller_io.write(0x4016, 0x00);
        controller_io.read(0x4016);
        controller_io.read(0x4016);
        controller_io.read(0x4016);

        assert_eq!(controller_io.button_index1, 3);

        // Strobe should reset index
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        assert_eq!(controller_io.button_index1, 0);
    }

    // ========================================
    // Controller Read Tests
    // ========================================

    #[test]
    fn test_read_controller1_all_released() {
        let mut controller_io = ControllerIO::new();

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read all 8 buttons (should all be 0 - not pressed)
        for _ in 0..8 {
            assert_eq!(controller_io.read(0x4016), 0x00);
        }

        // 9th read returns signature bit (1)
        assert_eq!(controller_io.read(0x4016), 0x01);
    }

    #[test]
    fn test_read_controller1_with_buttons_pressed() {
        let mut controller_io = ControllerIO::new();

        // Press some buttons
        let mut controller = Controller::new();
        controller.button_a = true; // Button 0
        controller.select = true; // Button 2
        controller.up = true; // Button 4
        controller_io.set_controller1(controller);

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read all 8 buttons
        assert_eq!(controller_io.read(0x4016), 0x01); // A pressed
        assert_eq!(controller_io.read(0x4016), 0x00); // B released
        assert_eq!(controller_io.read(0x4016), 0x01); // Select pressed
        assert_eq!(controller_io.read(0x4016), 0x00); // Start released
        assert_eq!(controller_io.read(0x4016), 0x01); // Up pressed
        assert_eq!(controller_io.read(0x4016), 0x00); // Down released
        assert_eq!(controller_io.read(0x4016), 0x00); // Left released
        assert_eq!(controller_io.read(0x4016), 0x00); // Right released
    }

    #[test]
    fn test_read_controller2_all_released() {
        let mut controller_io = ControllerIO::new();

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read all 8 buttons (should all be 0 - not pressed)
        for _ in 0..8 {
            assert_eq!(controller_io.read(0x4017), 0x00);
        }

        // 9th read returns signature bit (1)
        assert_eq!(controller_io.read(0x4017), 0x01);
    }

    #[test]
    fn test_read_controller2_with_buttons_pressed() {
        let mut controller_io = ControllerIO::new();

        // Press some buttons
        let mut controller = Controller::new();
        controller.button_b = true; // Button 1
        controller.start = true; // Button 3
        controller_io.set_controller2(controller);

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read all 8 buttons
        assert_eq!(controller_io.read(0x4017), 0x00); // A released
        assert_eq!(controller_io.read(0x4017), 0x01); // B pressed
        assert_eq!(controller_io.read(0x4017), 0x00); // Select released
        assert_eq!(controller_io.read(0x4017), 0x01); // Start pressed
        assert_eq!(controller_io.read(0x4017), 0x00); // Up released
        assert_eq!(controller_io.read(0x4017), 0x00); // Down released
        assert_eq!(controller_io.read(0x4017), 0x00); // Left released
        assert_eq!(controller_io.read(0x4017), 0x00); // Right released
    }

    #[test]
    fn test_strobe_returns_button_a() {
        let mut controller_io = ControllerIO::new();

        // Press button A
        let mut controller = Controller::new();
        controller.button_a = true;
        controller_io.set_controller1(controller);

        // While strobe is high, reading returns button A state
        controller_io.write(0x4016, 0x01);

        assert_eq!(controller_io.read(0x4016), 0x01);
        assert_eq!(controller_io.read(0x4016), 0x01);
        assert_eq!(controller_io.read(0x4016), 0x01);

        // Button index should not advance during strobe
        assert_eq!(controller_io.button_index1, 0);
    }

    #[test]
    fn test_multiple_read_sequences() {
        let mut controller_io = ControllerIO::new();

        // First sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);
        for _ in 0..8 {
            controller_io.read(0x4016);
        }

        // Second sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Should start from button 0 again
        assert_eq!(controller_io.button_index1, 0);
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_typical_controller_read_sequence() {
        let mut controller_io = ControllerIO::new();

        // Typical game controller reading
        controller_io.write(0x4016, 0x01); // Start strobe
        controller_io.write(0x4016, 0x00); // End strobe

        // Read controller 1
        for _ in 0..8 {
            let _ = controller_io.read(0x4016);
        }

        // Read controller 2
        for _ in 0..8 {
            let _ = controller_io.read(0x4017);
        }

        // Should complete without issues
    }

    #[test]
    fn test_simultaneous_controllers() {
        let mut controller_io = ControllerIO::new();

        // Press different buttons on each controller
        let mut controller1 = Controller::new();
        controller1.button_a = true;
        controller_io.set_controller1(controller1);

        let mut controller2 = Controller::new();
        controller2.button_b = true;
        controller_io.set_controller2(controller2);

        // Read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Controller 1 should have A pressed
        assert_eq!(controller_io.read(0x4016), 0x01); // A

        // Controller 2 should have B pressed
        assert_eq!(controller_io.read(0x4017), 0x00); // A
        assert_eq!(controller_io.read(0x4017), 0x01); // B
    }

    #[test]
    fn test_controller_independence() {
        let mut controller_io = ControllerIO::new();

        // Set different states
        let mut controller1 = Controller::new();
        controller1.button_a = true;
        controller_io.set_controller1(controller1);

        let mut controller2 = Controller::new();
        controller2.button_a = false;
        controller_io.set_controller2(controller2);

        // Read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Verify independence
        assert_eq!(controller_io.read(0x4016), 0x01); // Controller 1 A pressed
        assert_eq!(controller_io.read(0x4017), 0x00); // Controller 2 A released
    }

    // ========================================
    // Wraparound Prevention Tests
    // ========================================

    #[test]
    fn test_no_wraparound_after_256_reads() {
        let mut controller_io = ControllerIO::new();

        // Press button A on controller 1
        let mut controller = Controller::new();
        controller.button_a = true;
        controller_io.set_controller1(controller);

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read first 8 buttons (index 0-7)
        assert_eq!(controller_io.read(0x4016), 0x01); // A pressed
        for _ in 1..8 {
            assert_eq!(controller_io.read(0x4016), 0x00);
        }

        // Reads 9-300 should all return signature bit (1)
        // This tests that button_index stays clamped at 8 and doesn't wrap
        for i in 9..=300 {
            assert_eq!(
                controller_io.read(0x4016),
                0x01,
                "Read {} should return signature bit (1), not wrap to button data",
                i
            );
        }

        // Verify index is still clamped at 8
        assert_eq!(controller_io.button_index1, 8);
    }

    #[test]
    fn test_no_wraparound_controller2() {
        let mut controller_io = ControllerIO::new();

        // Standard read sequence
        controller_io.write(0x4016, 0x01);
        controller_io.write(0x4016, 0x00);

        // Read first 8 buttons
        for _ in 0..8 {
            controller_io.read(0x4017);
        }

        // Reads 9-100 should all return signature bit (1)
        for i in 9..=100 {
            assert_eq!(
                controller_io.read(0x4017),
                0x01,
                "Controller 2 read {} should return signature bit (1)",
                i
            );
        }

        // Verify index is still clamped at 8
        assert_eq!(controller_io.button_index2, 8);
    }
}
