// NES Emulator Library
// Core library for the NES emulator implementation

// Public modules
pub mod apu;
#[cfg(feature = "audio")]
pub mod audio;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod display;
pub mod input;
pub mod ppu;
pub mod ram;

// Re-export main types for convenience
pub use apu::Apu;
#[cfg(feature = "audio")]
pub use audio::{AudioConfig, AudioOutput, AudioSystem, Mixer};
pub use bus::{Bus, MemoryMappedDevice};
pub use cartridge::{Cartridge, INesError, INesHeader, Mapper, Mirroring};
pub use cpu::Cpu;
pub use display::{FrameBuffer, WindowConfig};
pub use input::{Controller, ControllerIO};
pub use ppu::Ppu;
pub use ram::Ram;

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
        let _controller_io = ControllerIO::new();
        let _ram = Ram::new();
    }
}
