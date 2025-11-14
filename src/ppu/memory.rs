// PPU memory access methods

use super::constants::NAMETABLE_SIZE;
use super::Ppu;
use crate::cartridge::Mirroring;

impl Ppu {
    /// Mirror nametable address based on mirroring mode
    ///
    /// The PPU has 2KB of internal VRAM for nametables, but the address space
    /// allows for 4 nametables ($2000-$2FFF). This function maps a nametable
    /// address to the appropriate physical memory location based on the mirroring mode.
    ///
    /// # Arguments
    ///
    /// * `addr` - Nametable address ($2000-$2FFF)
    ///
    /// # Returns
    ///
    /// Physical VRAM address (0-2047)
    pub(super) fn mirror_nametable_addr(&self, addr: u16) -> usize {
        // Normalize address to 0-0xFFF range (remove $2000 base)
        let addr = (addr & 0x0FFF) as usize;

        // Determine which nametable (0-3)
        let table = addr / NAMETABLE_SIZE;
        let offset = addr % NAMETABLE_SIZE;

        let mirrored_table = match self.mirroring {
            Mirroring::Horizontal => {
                // Horizontal: 0->0, 1->0, 2->1, 3->1
                // $2000=$2400, $2800=$2C00
                match table {
                    0 | 1 => 0,
                    2 | 3 => 1,
                    _ => unreachable!(),
                }
            }
            Mirroring::Vertical => {
                // Vertical: 0->0, 1->1, 2->0, 3->1
                // $2000=$2800, $2400=$2C00
                match table {
                    0 | 2 => 0,
                    1 | 3 => 1,
                    _ => unreachable!(),
                }
            }
            Mirroring::SingleScreen => {
                // All nametables point to the same physical table
                0
            }
            Mirroring::FourScreen => {
                // Four-screen would require 4KB of VRAM
                // For now, treat as horizontal mirroring
                // TODO: Implement four-screen VRAM when cartridge support is added
                match table {
                    0 | 1 => 0,
                    2 | 3 => 1,
                    _ => unreachable!(),
                }
            }
        };

        mirrored_table * NAMETABLE_SIZE + offset
    }

    /// Mirror palette address
    ///
    /// Palette RAM has special mirroring:
    /// - $3F10, $3F14, $3F18, $3F1C mirror $3F00, $3F04, $3F08, $3F0C
    /// - This is because sprite palette entry 0 is actually the background color
    ///
    /// # Arguments
    ///
    /// * `addr` - Palette address ($3F00-$3FFF)
    ///
    /// # Returns
    ///
    /// Physical palette RAM address (0-31)
    pub(super) fn mirror_palette_addr(&self, addr: u16) -> usize {
        // Palette RAM is at $3F00-$3F1F, mirrored every 32 bytes
        let addr = (addr & 0x001F) as usize;

        // Special mirroring: $3F10, $3F14, $3F18, $3F1C -> $3F00, $3F04, $3F08, $3F0C
        if addr >= 16 && addr.is_multiple_of(4) {
            addr - 16
        } else {
            addr
        }
    }

    /// Read from PPU memory (VRAM)
    ///
    /// Handles reading from pattern tables (via cartridge), nametables, and palette RAM.
    /// This is the internal memory read function used by PPUDATA.
    ///
    /// # Arguments
    ///
    /// * `addr` - PPU memory address ($0000-$3FFF)
    ///
    /// # Returns
    ///
    /// The byte value at the specified address
    pub(super) fn read_ppu_memory(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF; // Mirror to 14-bit address space

        match addr {
            // Pattern tables: $0000-$1FFF
            // Read from cartridge CHR-ROM/RAM via mapper
            0x0000..=0x1FFF => {
                if let Some(ref mapper) = self.mapper {
                    mapper.borrow().ppu_read(addr)
                } else {
                    // No cartridge loaded, return 0
                    0
                }
            }

            // Nametables: $2000-$2FFF
            0x2000..=0x2FFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr);
                self.nametables[mirrored_addr]
            }

            // Nametable mirrors: $3000-$3EFF -> $2000-$2EFF
            0x3000..=0x3EFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr - 0x1000);
                self.nametables[mirrored_addr]
            }

            // Palette RAM: $3F00-$3FFF
            0x3F00..=0x3FFF => {
                let mirrored_addr = self.mirror_palette_addr(addr);
                self.palette_ram[mirrored_addr]
            }

            _ => unreachable!(),
        }
    }

    /// Write to PPU memory (VRAM)
    ///
    /// Handles writing to pattern tables (via cartridge), nametables, and palette RAM.
    /// This is the internal memory write function used by PPUDATA.
    ///
    /// # Arguments
    ///
    /// * `addr` - PPU memory address ($0000-$3FFF)
    /// * `data` - Byte value to write
    pub(super) fn write_ppu_memory(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF; // Mirror to 14-bit address space

        match addr {
            // Pattern tables: $0000-$1FFF
            // Write to cartridge CHR-RAM (if present) via mapper
            0x0000..=0x1FFF => {
                if let Some(ref mapper) = self.mapper {
                    mapper.borrow_mut().ppu_write(addr, data);
                }
                // If no cartridge loaded, ignore writes
            }

            // Nametables: $2000-$2FFF
            0x2000..=0x2FFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr);
                self.nametables[mirrored_addr] = data;
            }

            // Nametable mirrors: $3000-$3EFF -> $2000-$2EFF
            0x3000..=0x3EFF => {
                let mirrored_addr = self.mirror_nametable_addr(addr - 0x1000);
                self.nametables[mirrored_addr] = data;
            }

            // Palette RAM: $3F00-$3FFF
            0x3F00..=0x3FFF => {
                let mirrored_addr = self.mirror_palette_addr(addr);
                self.palette_ram[mirrored_addr] = data;
            }

            _ => unreachable!(),
        }
    }
}
