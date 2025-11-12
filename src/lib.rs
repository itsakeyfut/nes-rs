// NES Emulator Library
// Core library for the NES emulator implementation

// Public modules
pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod input;
pub mod ppu;

// Re-export main types for convenience
pub use apu::Apu;
pub use bus::Bus;
pub use cartridge::Cartridge;
pub use cpu::Cpu;
pub use input::Controller;
pub use ppu::Ppu;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_components() {
        // Test that all components can be instantiated
        let _cpu = Cpu::new();
        let _ppu = Ppu::new();
        let _apu = Apu::new();
        let _bus = Bus::new();
        let _cartridge = Cartridge::new();
        let _controller = Controller::new();
    }
}
