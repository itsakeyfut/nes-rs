// Bus module - Memory bus implementation
// This module will contain the memory bus that connects CPU to memory and peripherals

/// Memory bus structure for connecting components
pub struct Bus {
    // RAM: 2KB internal RAM
    ram: [u8; 2048],
    // ROM: Temporary ROM storage for testing (will be replaced with proper cartridge)
    // Covers $4020-$FFFF (48KB)
    rom: [u8; 0xC000],
}

impl Bus {
    /// Create a new bus instance
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            rom: [0; 0xC000],
        }
    }

    /// Read a byte from memory
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // RAM and mirrors: $0000-$1FFF
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            // PPU registers: $2000-$3FFF
            0x2000..=0x3FFF => {
                // TODO: Implement PPU register reads
                0
            }
            // APU and I/O registers: $4000-$4017
            0x4000..=0x4017 => {
                // TODO: Implement APU/IO register reads
                0
            }
            // Cartridge space: $4020-$FFFF
            0x4020..=0xFFFF => {
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                self.rom[rom_addr]
            }
            _ => 0,
        }
    }

    /// Write a byte to memory
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // RAM and mirrors: $0000-$1FFF
            0x0000..=0x1FFF => {
                self.ram[(addr & 0x07FF) as usize] = data;
            }
            // PPU registers: $2000-$3FFF
            0x2000..=0x3FFF => {
                // TODO: Implement PPU register writes
            }
            // APU and I/O registers: $4000-$4017
            0x4000..=0x4017 => {
                // TODO: Implement APU/IO register writes
            }
            // Cartridge space: $4020-$FFFF
            0x4020..=0xFFFF => {
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                self.rom[rom_addr] = data;
            }
            _ => {}
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_initialization() {
        let bus = Bus::new();
        // Verify RAM is zero-initialized
        assert_eq!(bus.read(0x0000), 0);
        assert_eq!(bus.read(0x07FF), 0);
    }

    #[test]
    fn test_ram_read_write() {
        let mut bus = Bus::new();
        bus.write(0x0000, 0x42);
        assert_eq!(bus.read(0x0000), 0x42);
    }

    #[test]
    fn test_ram_mirroring() {
        let mut bus = Bus::new();
        // Write to base RAM
        bus.write(0x0000, 0x42);
        // Read from mirrored addresses
        assert_eq!(bus.read(0x0800), 0x42); // Mirror 1
        assert_eq!(bus.read(0x1000), 0x42); // Mirror 2
        assert_eq!(bus.read(0x1800), 0x42); // Mirror 3
    }
}
