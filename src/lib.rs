// NES Emulator Library
// Core library for the NES emulator implementation

// Public modules
pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod input;

// Re-export main types for convenience
pub use cpu::Cpu;
pub use ppu::Ppu;
pub use apu::Apu;
pub use bus::Bus;
pub use cartridge::Cartridge;
pub use input::Controller;

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
