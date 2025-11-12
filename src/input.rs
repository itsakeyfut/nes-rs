// Input module - Controller input handling
// This module will contain controller input processing

/// Controller structure representing NES controller state
pub struct Controller {
    // Button states
    pub button_a: bool,
    pub button_b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Controller {
    /// Create a new controller instance with all buttons released
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
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_initialization() {
        let controller = Controller::new();
        assert!(!controller.button_a);
        assert!(!controller.button_b);
        assert!(!controller.select);
        assert!(!controller.start);
    }
}
