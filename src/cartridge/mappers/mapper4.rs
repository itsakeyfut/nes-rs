// Mapper 4 (MMC3) - One of the most important and complex NES mappers
//
// Memory Layout:
// - CPU $6000-$7FFF: 8KB PRG-RAM (optional, battery-backed in some games)
// - CPU $8000-$9FFF: 8KB PRG-ROM bank (switchable or fixed depending on mode)
// - CPU $A000-$BFFF: 8KB PRG-ROM bank (always switchable)
// - CPU $C000-$DFFF: 8KB PRG-ROM bank (switchable or fixed depending on mode)
// - CPU $E000-$FFFF: 8KB PRG-ROM bank (fixed to last bank)
// - PPU $0000-$07FF: 2KB CHR bank (switchable)
// - PPU $0800-$0FFF: 2KB CHR bank (switchable)
// - PPU $1000-$13FF: 1KB CHR bank (switchable)
// - PPU $1400-$17FF: 1KB CHR bank (switchable)
// - PPU $1800-$1BFF: 1KB CHR bank (switchable)
// - PPU $1C00-$1FFF: 1KB CHR bank (switchable)
//
// Features:
// - Configurable PRG-ROM banking modes (8KB switchable banks)
// - Configurable CHR-ROM banking modes (1KB/2KB switchable banks)
// - Scanline counter for IRQ generation
// - Dynamic mirroring control
// - PRG-ROM size: up to 512KB
// - CHR-ROM size: up to 256KB
//
// Register Interface:
// - $8000-$9FFE (even): Bank select register
//   Bit 7: PRG-ROM bank mode (0 = $8000 switchable, 1 = $C000 switchable)
//   Bit 6: CHR A12 inversion (0 = two 2KB banks at $0000, 1 = two 2KB banks at $1000)
//   Bits 0-2: Bank register to update (0-7)
//
// - $8001-$9FFF (odd): Bank data register
//   Updates the bank register selected by $8000
//
// - $A000-$BFFE (even): Mirroring register
//   Bit 0: Mirroring (0 = vertical, 1 = horizontal)
//
// - $A001-$BFFF (odd): PRG-RAM protect register
//   Bit 7: PRG-RAM chip enable (0 = disabled, 1 = enabled)
//   Bit 6: Write protect (0 = read-only, 1 = read/write)
//
// - $C000-$DFFE (even): IRQ latch register
//   Sets the IRQ counter reload value
//
// - $C001-$DFFF (odd): IRQ reload register
//   Reloads the IRQ counter with the latch value
//
// - $E000-$FFFE (even): IRQ disable register
//   Disables IRQ generation and acknowledges pending IRQs
//
// - $E001-$FFFF (odd): IRQ enable register
//   Enables IRQ generation

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// PRG-ROM bank size (8KB)
const PRG_BANK_SIZE: usize = 8 * 1024;

/// CHR-ROM 1KB bank size
const CHR_1KB_BANK_SIZE: usize = 1024;

/// PRG-RAM size (8KB)
const PRG_RAM_SIZE: usize = 8 * 1024;

/// Mapper 4 implementation (MMC3)
///
/// MMC3 is one of the most common NES mappers, used by games like:
/// - Super Mario Bros. 3
/// - Mega Man 3, 4, 5, 6
/// - Kirby's Adventure
/// - Super Mario Bros. 2 (USA)
/// - Crystalis
pub struct Mapper4 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-ROM or CHR-RAM data
    chr_mem: Vec<u8>,
    /// PRG-RAM (8KB, battery-backed in some games)
    prg_ram: Vec<u8>,
    /// Whether CHR memory is RAM (writable) or ROM (read-only)
    chr_is_ram: bool,

    // Internal registers
    /// Bank select register (which bank register to update)
    bank_select: u8,
    /// Bank data registers (8 registers for different banks)
    bank_registers: [u8; 8],
    /// Current mirroring mode
    mirroring: Mirroring,
    /// PRG-RAM protection (bit 7: enable, bit 6: write protect)
    prg_ram_protect: u8,

    // IRQ registers
    /// IRQ latch value (reload value for counter)
    irq_latch: u8,
    /// IRQ counter (decrements each scanline)
    irq_counter: u8,
    /// IRQ reload flag (reload counter on next scanline)
    irq_reload: bool,
    /// IRQ enabled flag
    irq_enabled: bool,
    /// IRQ pending flag (set when counter reaches 0)
    irq_pending: bool,

    // Derived state
    /// Number of 8KB PRG-ROM banks
    prg_banks: usize,
    /// Number of 1KB CHR banks
    chr_banks: usize,
}

impl Mapper4 {
    /// Create a new Mapper4 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let chr_mem_size = cartridge.chr_rom.len();

        // Calculate number of banks
        let prg_banks = prg_rom_size / PRG_BANK_SIZE;
        let chr_banks = chr_mem_size / CHR_1KB_BANK_SIZE;

        // CHR-RAM is indicated by all zeros in chr_rom
        let chr_is_ram = chr_mem_size == 8 * 1024 && cartridge.chr_rom.iter().all(|&b| b == 0);

        Mapper4 {
            prg_rom: cartridge.prg_rom,
            chr_mem: cartridge.chr_rom,
            prg_ram: vec![0; PRG_RAM_SIZE],
            chr_is_ram,

            // Initialize registers to power-on state
            bank_select: 0,
            bank_registers: [0, 0, 0, 0, 0, 0, 0, 0],
            mirroring: cartridge.mirroring,
            prg_ram_protect: 0,

            // Initialize IRQ state
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,

            prg_banks,
            chr_banks,
        }
    }

    /// Get PRG-ROM bank mode from bank select register
    /// Returns true if $C000 is switchable, false if $8000 is switchable
    fn prg_bank_mode(&self) -> bool {
        self.bank_select & 0x40 != 0
    }

    /// Get CHR A12 inversion from bank select register
    /// Returns true if two 2KB banks are at $1000, false if at $0000
    fn chr_a12_inversion(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    /// Map CPU address to PRG-ROM offset
    fn map_prg_address(&self, address: u16) -> usize {
        let bank = match address {
            0x8000..=0x9FFF => {
                // First 8KB bank
                if self.prg_bank_mode() {
                    // Fixed to second-to-last bank
                    self.prg_banks - 2
                } else {
                    // Switchable via R6
                    self.bank_registers[6] as usize
                }
            }
            0xA000..=0xBFFF => {
                // Second 8KB bank (always switchable via R7)
                self.bank_registers[7] as usize
            }
            0xC000..=0xDFFF => {
                // Third 8KB bank
                if self.prg_bank_mode() {
                    // Switchable via R6
                    self.bank_registers[6] as usize
                } else {
                    // Fixed to second-to-last bank
                    self.prg_banks - 2
                }
            }
            0xE000..=0xFFFF => {
                // Last 8KB bank (always fixed to last bank)
                self.prg_banks - 1
            }
            _ => 0,
        };

        let offset = (address & 0x1FFF) as usize; // 8KB bank offset
        (bank % self.prg_banks) * PRG_BANK_SIZE + offset
    }

    /// Map PPU address to CHR offset
    fn map_chr_address(&self, address: u16) -> usize {
        let inversion = self.chr_a12_inversion();

        let bank = match address {
            0x0000..=0x03FF => {
                // First 1KB slot
                if inversion {
                    self.bank_registers[2] as usize
                } else {
                    (self.bank_registers[0] & 0xFE) as usize // 2KB bank, use even
                }
            }
            0x0400..=0x07FF => {
                // Second 1KB slot
                if inversion {
                    self.bank_registers[3] as usize
                } else {
                    ((self.bank_registers[0] & 0xFE) | 1) as usize // 2KB bank, use odd
                }
            }
            0x0800..=0x0BFF => {
                // Third 1KB slot
                if inversion {
                    self.bank_registers[4] as usize
                } else {
                    (self.bank_registers[1] & 0xFE) as usize // 2KB bank, use even
                }
            }
            0x0C00..=0x0FFF => {
                // Fourth 1KB slot
                if inversion {
                    self.bank_registers[5] as usize
                } else {
                    ((self.bank_registers[1] & 0xFE) | 1) as usize // 2KB bank, use odd
                }
            }
            0x1000..=0x13FF => {
                // Fifth 1KB slot
                if inversion {
                    (self.bank_registers[0] & 0xFE) as usize // 2KB bank, use even
                } else {
                    self.bank_registers[2] as usize
                }
            }
            0x1400..=0x17FF => {
                // Sixth 1KB slot
                if inversion {
                    ((self.bank_registers[0] & 0xFE) | 1) as usize // 2KB bank, use odd
                } else {
                    self.bank_registers[3] as usize
                }
            }
            0x1800..=0x1BFF => {
                // Seventh 1KB slot
                if inversion {
                    (self.bank_registers[1] & 0xFE) as usize // 2KB bank, use even
                } else {
                    self.bank_registers[4] as usize
                }
            }
            0x1C00..=0x1FFF => {
                // Eighth 1KB slot
                if inversion {
                    ((self.bank_registers[1] & 0xFE) | 1) as usize // 2KB bank, use odd
                } else {
                    self.bank_registers[5] as usize
                }
            }
            _ => 0,
        };

        let offset = (address & 0x03FF) as usize; // 1KB bank offset
        (bank % self.chr_banks) * CHR_1KB_BANK_SIZE + offset
    }

    /// Clock the IRQ counter (called by PPU on A12 rise, typically each scanline)
    ///
    /// Note: This is a simplified implementation. Full MMC3 IRQ emulation
    /// requires tracking A12 transitions from the PPU, which would need
    /// integration with the PPU scanline rendering. For now, this method
    /// can be called externally when needed.
    #[allow(dead_code)]
    pub fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }

    /// Check if an IRQ is pending
    #[allow(dead_code)]
    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    /// Clear the pending IRQ
    #[allow(dead_code)]
    pub fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

impl Mapper for Mapper4 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            // PRG-RAM
            0x6000..=0x7FFF => {
                // Check if PRG-RAM is enabled
                if self.prg_ram_protect & 0x80 != 0 {
                    let index = (address - 0x6000) as usize;
                    self.prg_ram[index % PRG_RAM_SIZE]
                } else {
                    0 // PRG-RAM disabled, return open bus (simplified as 0)
                }
            }
            // PRG-ROM
            0x8000..=0xFFFF => {
                let index = self.map_prg_address(address);
                self.prg_rom[index]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            // PRG-RAM
            0x6000..=0x7FFF => {
                // Check if PRG-RAM is enabled and writable
                if self.prg_ram_protect & 0x80 != 0 && self.prg_ram_protect & 0x40 != 0 {
                    let index = (address - 0x6000) as usize;
                    self.prg_ram[index % PRG_RAM_SIZE] = value;
                }
            }
            // Mapper registers
            0x8000..=0xFFFF => {
                match address & 0xE001 {
                    // Bank select ($8000-$9FFE, even)
                    0x8000 => {
                        self.bank_select = value;
                    }
                    // Bank data ($8001-$9FFF, odd)
                    0x8001 => {
                        let reg = (self.bank_select & 0x07) as usize;
                        self.bank_registers[reg] = value;
                    }
                    // Mirroring ($A000-$BFFE, even)
                    0xA000 => {
                        self.mirroring = if value & 0x01 != 0 {
                            Mirroring::Horizontal
                        } else {
                            Mirroring::Vertical
                        };
                    }
                    // PRG-RAM protect ($A001-$BFFF, odd)
                    0xA001 => {
                        self.prg_ram_protect = value;
                    }
                    // IRQ latch ($C000-$DFFE, even)
                    0xC000 => {
                        self.irq_latch = value;
                    }
                    // IRQ reload ($C001-$DFFF, odd)
                    0xC001 => {
                        self.irq_reload = true;
                    }
                    // IRQ disable ($E000-$FFFE, even)
                    0xE000 => {
                        self.irq_enabled = false;
                        self.irq_pending = false;
                    }
                    // IRQ enable ($E001-$FFFF, odd)
                    0xE001 => {
                        self.irq_enabled = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = self.map_chr_address(address);
                self.chr_mem[index % self.chr_mem.len()]
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        if self.chr_is_ram {
            if let 0x0000..=0x1FFF = address {
                let chr_len = self.chr_mem.len();
                let index = self.map_chr_address(address);
                self.chr_mem[index % chr_len] = value;
            }
        }
        // Writes to CHR-ROM are ignored
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn prg_ram(&self) -> Option<&[u8]> {
        Some(&self.prg_ram)
    }

    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.prg_ram)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test cartridge
    fn create_test_cartridge(prg_banks: usize, chr_banks: usize) -> Cartridge {
        let prg_rom = vec![0; prg_banks * PRG_BANK_SIZE];
        let chr_rom = vec![0; chr_banks * CHR_1KB_BANK_SIZE];

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 4,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper4_creation() {
        let cartridge = create_test_cartridge(16, 128);
        let mapper = Mapper4::new(cartridge);

        assert_eq!(mapper.prg_banks, 16);
        assert_eq!(mapper.chr_banks, 128);
        assert_eq!(mapper.bank_select, 0);
        assert_eq!(mapper.mirroring, Mirroring::Horizontal);
    }

    #[test]
    fn test_bank_select_register() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // Write to bank select
        mapper.cpu_write(0x8000, 0x06); // Select R6, PRG bank mode 0
        assert_eq!(mapper.bank_select, 0x06);

        // Write to bank data
        mapper.cpu_write(0x8001, 0x0A); // Set R6 to 10
        assert_eq!(mapper.bank_registers[6], 0x0A);
    }

    #[test]
    fn test_prg_bank_switching() {
        let mut cartridge = create_test_cartridge(16, 128);

        // Fill banks with identifiable patterns
        for bank in 0..16 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper4::new(cartridge);

        // Test default state (bank mode 0)
        // R6 controls $8000-$9FFF, R7 controls $A000-$BFFF
        // $C000-$DFFF is fixed to second-to-last, $E000-$FFFF is fixed to last

        // Set R6 to bank 5
        mapper.cpu_write(0x8000, 0x06); // Select R6
        mapper.cpu_write(0x8001, 0x05); // Bank 5

        // Set R7 to bank 7
        mapper.cpu_write(0x8000, 0x07); // Select R7
        mapper.cpu_write(0x8001, 0x07); // Bank 7

        assert_eq!(mapper.cpu_read(0x8000), 5); // Bank 5 (R6)
        assert_eq!(mapper.cpu_read(0xA000), 7); // Bank 7 (R7)
        assert_eq!(mapper.cpu_read(0xC000), 14); // Fixed second-to-last (bank 14)
        assert_eq!(mapper.cpu_read(0xE000), 15); // Fixed last (bank 15)
    }

    #[test]
    fn test_prg_bank_mode_switching() {
        let mut cartridge = create_test_cartridge(16, 128);

        // Fill banks with identifiable patterns
        for bank in 0..16 {
            let start = bank * PRG_BANK_SIZE;
            cartridge.prg_rom[start] = bank as u8;
        }

        let mut mapper = Mapper4::new(cartridge);

        // Set R6 to bank 5
        mapper.cpu_write(0x8000, 0x06);
        mapper.cpu_write(0x8001, 0x05);

        // Set R7 to bank 7
        mapper.cpu_write(0x8000, 0x07);
        mapper.cpu_write(0x8001, 0x07);

        // Switch to bank mode 1 (bit 6 set)
        mapper.cpu_write(0x8000, 0x46); // Set bank mode bit

        // In mode 1:
        // $8000-$9FFF is fixed to second-to-last
        // $A000-$BFFF is R7 (still switchable)
        // $C000-$DFFF is R6 (switchable)
        // $E000-$FFFF is fixed to last

        assert_eq!(mapper.cpu_read(0x8000), 14); // Fixed second-to-last
        assert_eq!(mapper.cpu_read(0xA000), 7); // R7
        assert_eq!(mapper.cpu_read(0xC000), 5); // R6
        assert_eq!(mapper.cpu_read(0xE000), 15); // Fixed last
    }

    #[test]
    fn test_chr_bank_switching() {
        let mut cartridge = create_test_cartridge(16, 128);

        // Fill CHR banks with identifiable patterns
        for bank in 0..128 {
            let start = bank * CHR_1KB_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper4::new(cartridge);

        // Set up CHR banks (no inversion)
        // R0 = 2KB bank at $0000-$07FF
        // R1 = 2KB bank at $0800-$0FFF
        // R2-R5 = 1KB banks at $1000-$1FFF

        mapper.cpu_write(0x8000, 0x00); // Select R0
        mapper.cpu_write(0x8001, 0x04); // Bank 4 (will use 4 and 5 for 2KB)

        mapper.cpu_write(0x8000, 0x01); // Select R1
        mapper.cpu_write(0x8001, 0x08); // Bank 8 (will use 8 and 9 for 2KB)

        mapper.cpu_write(0x8000, 0x02); // Select R2
        mapper.cpu_write(0x8001, 0x10); // Bank 16

        mapper.cpu_write(0x8000, 0x03); // Select R3
        mapper.cpu_write(0x8001, 0x11); // Bank 17

        mapper.cpu_write(0x8000, 0x04); // Select R4
        mapper.cpu_write(0x8001, 0x12); // Bank 18

        mapper.cpu_write(0x8000, 0x05); // Select R5
        mapper.cpu_write(0x8001, 0x13); // Bank 19

        // Test reads
        assert_eq!(mapper.ppu_read(0x0000), 4); // R0 even (bank & 0xFE = 4)
        assert_eq!(mapper.ppu_read(0x0400), 5); // R0 odd (bank | 1 = 5)
        assert_eq!(mapper.ppu_read(0x0800), 8); // R1 even
        assert_eq!(mapper.ppu_read(0x0C00), 9); // R1 odd
        assert_eq!(mapper.ppu_read(0x1000), 16); // R2
        assert_eq!(mapper.ppu_read(0x1400), 17); // R3
        assert_eq!(mapper.ppu_read(0x1800), 18); // R4
        assert_eq!(mapper.ppu_read(0x1C00), 19); // R5
    }

    #[test]
    fn test_chr_a12_inversion() {
        let mut cartridge = create_test_cartridge(16, 128);

        // Fill CHR banks with identifiable patterns
        for bank in 0..128 {
            let start = bank * CHR_1KB_BANK_SIZE;
            cartridge.chr_rom[start] = bank as u8;
        }

        let mut mapper = Mapper4::new(cartridge);

        // Set up CHR banks with A12 inversion
        mapper.cpu_write(0x8000, 0x80); // Enable CHR A12 inversion

        mapper.cpu_write(0x8000, 0x80); // Select R0
        mapper.cpu_write(0x8001, 0x04); // Bank 4

        mapper.cpu_write(0x8000, 0x81); // Select R1
        mapper.cpu_write(0x8001, 0x08); // Bank 8

        mapper.cpu_write(0x8000, 0x82); // Select R2
        mapper.cpu_write(0x8001, 0x10); // Bank 16

        mapper.cpu_write(0x8000, 0x83); // Select R3
        mapper.cpu_write(0x8001, 0x11); // Bank 17

        mapper.cpu_write(0x8000, 0x84); // Select R4
        mapper.cpu_write(0x8001, 0x12); // Bank 18

        mapper.cpu_write(0x8000, 0x85); // Select R5
        mapper.cpu_write(0x8001, 0x13); // Bank 19

        // With inversion, R0 and R1 are at $1000-$1FFF, R2-R5 are at $0000-$0FFF
        assert_eq!(mapper.ppu_read(0x0000), 16); // R2
        assert_eq!(mapper.ppu_read(0x0400), 17); // R3
        assert_eq!(mapper.ppu_read(0x0800), 18); // R4
        assert_eq!(mapper.ppu_read(0x0C00), 19); // R5
        assert_eq!(mapper.ppu_read(0x1000), 4); // R0 even
        assert_eq!(mapper.ppu_read(0x1400), 5); // R0 odd
        assert_eq!(mapper.ppu_read(0x1800), 8); // R1 even
        assert_eq!(mapper.ppu_read(0x1C00), 9); // R1 odd
    }

    #[test]
    fn test_mirroring_control() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // Initial mirroring
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        // Switch to vertical
        mapper.cpu_write(0xA000, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        // Switch back to horizontal
        mapper.cpu_write(0xA000, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn test_prg_ram_protection() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // PRG-RAM disabled by default
        mapper.cpu_write(0x6000, 0x42);
        assert_eq!(mapper.cpu_read(0x6000), 0); // Should read 0 (disabled)

        // Enable PRG-RAM (bit 7) and make writable (bit 6)
        mapper.cpu_write(0xA001, 0xC0);

        // Now writes should work
        mapper.cpu_write(0x6000, 0x42);
        assert_eq!(mapper.cpu_read(0x6000), 0x42);

        // Make read-only (clear bit 6)
        mapper.cpu_write(0xA001, 0x80);

        // Writes should be ignored
        mapper.cpu_write(0x6000, 0x99);
        assert_eq!(mapper.cpu_read(0x6000), 0x42); // Should still be 0x42
    }

    #[test]
    fn test_irq_latch_and_reload() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // Set IRQ latch
        mapper.cpu_write(0xC000, 0x05);
        assert_eq!(mapper.irq_latch, 0x05);

        // Trigger reload
        mapper.cpu_write(0xC001, 0x00);
        assert!(mapper.irq_reload);
    }

    #[test]
    fn test_irq_enable_disable() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // IRQ disabled by default
        assert!(!mapper.irq_enabled);

        // Enable IRQ
        mapper.cpu_write(0xE001, 0x00);
        assert!(mapper.irq_enabled);

        // Disable IRQ
        mapper.cpu_write(0xE000, 0x00);
        assert!(!mapper.irq_enabled);
        assert!(!mapper.irq_pending); // Should also clear pending
    }

    #[test]
    fn test_irq_counter() {
        let cartridge = create_test_cartridge(16, 128);
        let mut mapper = Mapper4::new(cartridge);

        // Set latch to 3
        mapper.cpu_write(0xC000, 0x03);

        // Trigger reload
        mapper.cpu_write(0xC001, 0x00);

        // Enable IRQ
        mapper.cpu_write(0xE001, 0x00);

        // Clock the counter (simulating scanlines)
        mapper.clock_irq_counter(); // Counter = 3 (reloaded)
        assert!(!mapper.irq_pending());

        mapper.clock_irq_counter(); // Counter = 2
        assert!(!mapper.irq_pending());

        mapper.clock_irq_counter(); // Counter = 1
        assert!(!mapper.irq_pending());

        mapper.clock_irq_counter(); // Counter = 0, IRQ triggered
        assert!(mapper.irq_pending());
    }

    #[test]
    fn test_chr_ram_writes() {
        // Create cartridge with CHR-RAM
        let prg_rom = vec![0; 16 * PRG_BANK_SIZE];
        let chr_rom = vec![0; 8 * 1024]; // 8KB all zeros = CHR-RAM

        let cartridge = Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 4,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        };

        let mut mapper = Mapper4::new(cartridge);
        assert!(mapper.chr_is_ram);

        // Test writes to CHR-RAM
        mapper.ppu_write(0x0000, 0x42);
        assert_eq!(mapper.ppu_read(0x0000), 0x42);

        mapper.ppu_write(0x1FFF, 0x99);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x99);
    }
}
