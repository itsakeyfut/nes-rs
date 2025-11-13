// Mapper 1 (MMC1) - One of the most common NES mappers
//
// Memory Layout:
// - CPU $6000-$7FFF: 8KB PRG-RAM (optional, battery-backed)
// - CPU $8000-$BFFF: 16KB PRG-ROM bank (switchable or fixed depending on mode)
// - CPU $C000-$FFFF: 16KB PRG-ROM bank (switchable or fixed depending on mode)
// - PPU $0000-$0FFF: 4KB CHR bank 0 (switchable)
// - PPU $1000-$1FFF: 4KB CHR bank 1 (switchable)
//
// Features:
// - Serial write interface (5 writes to load a register)
// - Configurable PRG-ROM banking modes
// - Configurable CHR-ROM banking modes
// - Dynamic mirroring control
// - PRG-ROM size: up to 256KB (512KB possible with later variants)
// - CHR-ROM size: up to 128KB
//
// Register Interface:
// All writes to $8000-$FFFF use a serial shift register:
// - Bit 7 set: Reset shift register and write counter
// - Bit 0: Data bit to shift in
// - After 5 writes, the accumulated value is written to the target register
//
// Control Register ($8000-$9FFF):
//   Bits 0-1: Mirroring (0=one-screen lower, 1=one-screen upper, 2=vertical, 3=horizontal)
//   Bits 2-3: PRG-ROM bank mode
//   Bit 4: CHR-ROM bank mode
//
// CHR Bank 0 ($A000-$BFFF):
//   Bits 0-4: Select CHR bank for PPU $0000-$0FFF (or $0000-$1FFF in 8KB mode)
//
// CHR Bank 1 ($C000-$DFFF):
//   Bits 0-4: Select CHR bank for PPU $1000-$1FFF (ignored in 8KB mode)
//
// PRG Bank ($E000-$FFFF):
//   Bits 0-3: Select PRG-ROM bank
//   Bit 4: PRG-RAM chip enable (0=enabled, but often ignored)

use crate::cartridge::{Cartridge, Mapper, Mirroring};

/// PRG-ROM bank size (16KB)
const PRG_BANK_SIZE: usize = 16 * 1024;

/// CHR-ROM bank size (4KB)
const CHR_BANK_SIZE: usize = 4 * 1024;

/// PRG-RAM size (8KB)
const PRG_RAM_SIZE: usize = 8 * 1024;

/// PRG-ROM banking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrgBankMode {
    /// Switch 32KB at $8000, ignoring low bit of bank number
    Switch32KB = 0,
    /// Switch 32KB at $8000, ignoring low bit of bank number (same as mode 0)
    Switch32KBAlt = 1,
    /// Fix first bank at $8000, switch 16KB bank at $C000
    FixFirst = 2,
    /// Fix last bank at $C000, switch 16KB bank at $8000
    FixLast = 3,
}

impl From<u8> for PrgBankMode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => PrgBankMode::Switch32KB,
            1 => PrgBankMode::Switch32KBAlt,
            2 => PrgBankMode::FixFirst,
            3 => PrgBankMode::FixLast,
            _ => unreachable!(),
        }
    }
}

/// CHR-ROM banking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChrBankMode {
    /// Switch 8KB at a time
    Switch8KB = 0,
    /// Switch two separate 4KB banks
    Switch4KB = 1,
}

impl From<u8> for ChrBankMode {
    fn from(value: u8) -> Self {
        if value & 1 == 0 {
            ChrBankMode::Switch8KB
        } else {
            ChrBankMode::Switch4KB
        }
    }
}

/// Mapper 1 implementation (MMC1)
///
/// MMC1 is one of the most common NES mappers, used by games like:
/// - The Legend of Zelda
/// - Metroid
/// - Mega Man 2
/// - Kid Icarus
/// - Castlevania II
pub struct Mapper1 {
    /// PRG-ROM data
    prg_rom: Vec<u8>,
    /// CHR-ROM or CHR-RAM data
    chr_mem: Vec<u8>,
    /// PRG-RAM (8KB, battery-backed in some games)
    prg_ram: Vec<u8>,
    /// Whether CHR memory is RAM (writable) or ROM (read-only)
    chr_is_ram: bool,

    // Shift register state
    /// Shift register for serial writes (5 bits)
    shift_register: u8,
    /// Number of writes to the shift register (0-4, resets to 0 after 5th write)
    write_count: u8,

    // Internal registers
    /// Control register (mirroring and banking modes)
    control: u8,
    /// CHR bank 0 register
    chr_bank_0: u8,
    /// CHR bank 1 register
    chr_bank_1: u8,
    /// PRG bank register
    prg_bank: u8,

    // Derived state
    /// Number of 16KB PRG-ROM banks
    prg_banks: usize,
    /// Number of 4KB CHR banks
    chr_banks: usize,
}

impl Mapper1 {
    /// Create a new Mapper1 instance from a cartridge
    ///
    /// # Arguments
    /// * `cartridge` - The cartridge containing ROM data
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom.len();
        let chr_mem_size = cartridge.chr_rom.len();

        // Calculate number of banks
        let prg_banks = prg_rom_size / PRG_BANK_SIZE;
        let chr_banks = chr_mem_size / CHR_BANK_SIZE;

        // CHR-RAM is indicated by all zeros in chr_rom
        let chr_is_ram = chr_mem_size == 8 * 1024 && cartridge.chr_rom.iter().all(|&b| b == 0);

        Mapper1 {
            prg_rom: cartridge.prg_rom,
            chr_mem: cartridge.chr_rom,
            prg_ram: vec![0; PRG_RAM_SIZE],
            chr_is_ram,

            // Initialize shift register
            shift_register: 0,
            write_count: 0,

            // Initialize registers to power-on state
            // Control register defaults: last bank mode, 4KB CHR mode, horizontal mirroring
            control: 0x1F, // bits 0-1 = 11 (horizontal), bits 2-3 = 11 (fix last), bit 4 = 1 (4KB CHR)
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,

            prg_banks,
            chr_banks,
        }
    }

    /// Reset the shift register (called when bit 7 of write value is set)
    fn reset_shift_register(&mut self) {
        self.shift_register = 0;
        self.write_count = 0;
        // Also set control register to fix last bank mode
        self.control |= 0x0C;
    }

    /// Write a bit to the shift register
    ///
    /// Returns true if the register is full (5 bits written)
    fn write_shift_register(&mut self, value: u8) -> bool {
        // Shift in bit 0 of the value
        self.shift_register >>= 1;
        self.shift_register |= (value & 1) << 4;
        self.write_count += 1;

        if self.write_count == 5 {
            // Register is full, ready to write
            true
        } else {
            false
        }
    }

    /// Write to an internal register after shift register is full
    fn write_internal_register(&mut self, address: u16, value: u8) {
        match address {
            // Control register ($8000-$9FFF)
            0x8000..=0x9FFF => {
                self.control = value & 0x1F;
            }
            // CHR bank 0 ($A000-$BFFF)
            0xA000..=0xBFFF => {
                self.chr_bank_0 = value & 0x1F;
            }
            // CHR bank 1 ($C000-$DFFF)
            0xC000..=0xDFFF => {
                self.chr_bank_1 = value & 0x1F;
            }
            // PRG bank ($E000-$FFFF)
            0xE000..=0xFFFF => {
                self.prg_bank = value & 0x0F;
            }
            _ => {}
        }
    }

    /// Get the current mirroring mode from the control register
    fn get_mirroring(&self) -> Mirroring {
        match self.control & 0b11 {
            0 => Mirroring::SingleScreen, // One-screen, lower bank
            1 => Mirroring::SingleScreen, // One-screen, upper bank (we'll treat both as SingleScreen)
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }

    /// Get the current PRG banking mode
    fn get_prg_bank_mode(&self) -> PrgBankMode {
        PrgBankMode::from((self.control >> 2) & 0b11)
    }

    /// Get the current CHR banking mode
    fn get_chr_bank_mode(&self) -> ChrBankMode {
        ChrBankMode::from((self.control >> 4) & 1)
    }

    /// Map CPU address to PRG-ROM offset
    fn map_prg_address(&self, address: u16) -> usize {
        let mode = self.get_prg_bank_mode();
        let bank = self.prg_bank as usize;
        let last_bank = self.prg_banks - 1;

        match address {
            0x8000..=0xBFFF => {
                // First 16KB bank
                let bank_num = match mode {
                    PrgBankMode::Switch32KB | PrgBankMode::Switch32KBAlt => {
                        // 32KB mode: use bank number with bit 0 cleared
                        bank & !1
                    }
                    PrgBankMode::FixFirst => {
                        // First bank fixed to bank 0
                        0
                    }
                    PrgBankMode::FixLast => {
                        // First bank is switchable
                        bank
                    }
                };
                let offset = (address - 0x8000) as usize;
                (bank_num % self.prg_banks) * PRG_BANK_SIZE + offset
            }
            0xC000..=0xFFFF => {
                // Second 16KB bank
                let bank_num = match mode {
                    PrgBankMode::Switch32KB | PrgBankMode::Switch32KBAlt => {
                        // 32KB mode: use bank number with bit 0 set
                        (bank & !1) | 1
                    }
                    PrgBankMode::FixFirst => {
                        // Second bank is switchable
                        bank
                    }
                    PrgBankMode::FixLast => {
                        // Last bank fixed to last bank
                        last_bank
                    }
                };
                let offset = (address - 0xC000) as usize;
                (bank_num % self.prg_banks) * PRG_BANK_SIZE + offset
            }
            _ => 0, // Should not happen
        }
    }

    /// Map PPU address to CHR offset
    fn map_chr_address(&self, address: u16) -> usize {
        let mode = self.get_chr_bank_mode();

        match mode {
            ChrBankMode::Switch8KB => {
                // 8KB mode: use chr_bank_0 with bit 0 ignored
                let bank = (self.chr_bank_0 >> 1) as usize;
                let offset = address as usize;
                (bank % (self.chr_banks / 2)) * CHR_BANK_SIZE * 2 + offset
            }
            ChrBankMode::Switch4KB => {
                // 4KB mode: use separate banks
                match address {
                    0x0000..=0x0FFF => {
                        let bank = self.chr_bank_0 as usize;
                        let offset = address as usize;
                        (bank % self.chr_banks) * CHR_BANK_SIZE + offset
                    }
                    0x1000..=0x1FFF => {
                        let bank = self.chr_bank_1 as usize;
                        let offset = (address - 0x1000) as usize;
                        (bank % self.chr_banks) * CHR_BANK_SIZE + offset
                    }
                    _ => 0, // Should not happen
                }
            }
        }
    }
}

impl Mapper for Mapper1 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            // PRG-RAM
            0x6000..=0x7FFF => {
                let index = (address - 0x6000) as usize;
                self.prg_ram[index % PRG_RAM_SIZE]
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
                let index = (address - 0x6000) as usize;
                self.prg_ram[index % PRG_RAM_SIZE] = value;
            }
            // Mapper registers (via serial write)
            0x8000..=0xFFFF => {
                // Check for reset bit
                if value & 0x80 != 0 {
                    self.reset_shift_register();
                    return;
                }

                // Write to shift register
                if self.write_shift_register(value) {
                    // Shift register is full, write to internal register
                    let register_value = self.shift_register;
                    self.write_internal_register(address, register_value);

                    // Reset shift register for next write
                    self.shift_register = 0;
                    self.write_count = 0;
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
        self.get_mirroring()
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
        let chr_rom = vec![0; chr_banks * CHR_BANK_SIZE];

        Cartridge {
            prg_rom,
            chr_rom,
            trainer: None,
            mapper: 1,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        }
    }

    #[test]
    fn test_mapper1_creation() {
        let cartridge = create_test_cartridge(16, 32);
        let mapper = Mapper1::new(cartridge);

        assert_eq!(mapper.prg_banks, 16);
        assert_eq!(mapper.chr_banks, 32);
        assert_eq!(mapper.control, 0x1F); // Default: fix last bank, 4KB CHR mode, horizontal mirroring
    }

    #[test]
    fn test_shift_register_reset() {
        let cartridge = create_test_cartridge(16, 32);
        let mut mapper = Mapper1::new(cartridge);

        // Write some bits
        mapper.cpu_write(0x8000, 0x01);
        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.write_count, 2);

        // Reset with bit 7 set
        mapper.cpu_write(0x8000, 0x80);
        assert_eq!(mapper.write_count, 0);
        assert_eq!(mapper.shift_register, 0);
    }

    #[test]
    fn test_serial_write_control_register() {
        let cartridge = create_test_cartridge(16, 32);
        let mut mapper = Mapper1::new(cartridge);

        // Write value 0b10101 (21) to control register via serial writes
        // Bit order: LSB first, so we write 1, 0, 1, 0, 1
        mapper.cpu_write(0x8000, 0x01); // bit 0 = 1
        mapper.cpu_write(0x8000, 0x00); // bit 1 = 0
        mapper.cpu_write(0x8000, 0x01); // bit 2 = 1
        mapper.cpu_write(0x8000, 0x00); // bit 3 = 0
        mapper.cpu_write(0x8000, 0x01); // bit 4 = 1

        assert_eq!(mapper.control, 0b10101);
        assert_eq!(mapper.write_count, 0); // Should reset after 5 writes
    }

    #[test]
    fn test_mirroring_control() {
        let cartridge = create_test_cartridge(16, 32);
        let mut mapper = Mapper1::new(cartridge);

        // Write 0b00010 (2) to control register - vertical mirroring
        mapper.cpu_write(0x8000, 0x00); // bit 0 = 0
        mapper.cpu_write(0x8000, 0x01); // bit 1 = 1
        mapper.cpu_write(0x8000, 0x00); // bit 2 = 0
        mapper.cpu_write(0x8000, 0x00); // bit 3 = 0
        mapper.cpu_write(0x8000, 0x00); // bit 4 = 0

        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        // Write 0b00011 (3) to control register - horizontal mirroring
        mapper.cpu_write(0x8000, 0x80); // Reset
        mapper.cpu_write(0x8000, 0x01); // bit 0 = 1
        mapper.cpu_write(0x8000, 0x01); // bit 1 = 1
        mapper.cpu_write(0x8000, 0x00); // bit 2 = 0
        mapper.cpu_write(0x8000, 0x00); // bit 3 = 0
        mapper.cpu_write(0x8000, 0x00); // bit 4 = 0

        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn test_prg_bank_mode_switch_32kb() {
        let mut cartridge = create_test_cartridge(4, 32); // 64KB PRG-ROM

        // Fill banks with identifiable patterns
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..PRG_BANK_SIZE {
                cartridge.prg_rom[start + i] = (bank as u8)
                    .wrapping_mul(0x10)
                    .wrapping_add((i & 0xFF) as u8);
            }
        }

        let mut mapper = Mapper1::new(cartridge);

        // Set to 32KB mode (control bits 2-3 = 00)
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);

        // Select bank 2 (which will select banks 2 and 3)
        mapper.cpu_write(0xE000, 0x00);
        mapper.cpu_write(0xE000, 0x01); // bit 0 = 1
        mapper.cpu_write(0xE000, 0x00); // bit 1 = 0
        mapper.cpu_write(0xE000, 0x00); // bit 2 = 0
        mapper.cpu_write(0xE000, 0x00); // bit 3 = 0

        // Read from first half (should be bank 2 due to bit 0 cleared)
        assert_eq!(mapper.cpu_read(0x8000), 0x20);
        // Read from second half (should be bank 3 due to bit 0 set)
        assert_eq!(mapper.cpu_read(0xC000), 0x30);
    }

    #[test]
    fn test_prg_bank_mode_fix_last() {
        let mut cartridge = create_test_cartridge(4, 32); // 64KB PRG-ROM

        // Fill banks with identifiable patterns
        for bank in 0..4 {
            let start = bank * PRG_BANK_SIZE;
            for i in 0..PRG_BANK_SIZE {
                cartridge.prg_rom[start + i] = (bank as u8).wrapping_mul(0x10);
            }
        }

        let mut mapper = Mapper1::new(cartridge);

        // Default mode is fix last (control = 0x1F, bits 2-3 = 11)
        assert_eq!(mapper.get_prg_bank_mode(), PrgBankMode::FixLast);

        // Select bank 1 for first 16KB
        mapper.cpu_write(0xE000, 0x01);
        mapper.cpu_write(0xE000, 0x00);
        mapper.cpu_write(0xE000, 0x00);
        mapper.cpu_write(0xE000, 0x00);
        mapper.cpu_write(0xE000, 0x00);

        // First half should be bank 1
        assert_eq!(mapper.cpu_read(0x8000), 0x10);
        // Second half should be last bank (bank 3)
        assert_eq!(mapper.cpu_read(0xC000), 0x30);
    }

    #[test]
    fn test_chr_bank_mode_8kb() {
        let mut cartridge = create_test_cartridge(16, 4); // 16KB CHR-ROM

        // Fill CHR banks with identifiable patterns
        for bank in 0..4 {
            let start = bank * CHR_BANK_SIZE;
            for i in 0..CHR_BANK_SIZE {
                cartridge.chr_rom[start + i] = (bank as u8).wrapping_mul(0x10);
            }
        }

        let mut mapper = Mapper1::new(cartridge);

        // Set to 8KB CHR mode (control bit 4 = 0)
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);

        // Select CHR bank 2 (8KB, so banks 2-3)
        // In 8KB mode, bit 0 is ignored, so we write bank >> 1 = 1
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x01); // bit 0 = 1
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);

        // Both halves should read from banks 2-3
        assert_eq!(mapper.ppu_read(0x0000), 0x20);
        assert_eq!(mapper.ppu_read(0x1000), 0x30);
    }

    #[test]
    fn test_chr_bank_mode_4kb() {
        let mut cartridge = create_test_cartridge(16, 4); // 16KB CHR-ROM

        // Fill CHR banks with identifiable patterns
        for bank in 0..4 {
            let start = bank * CHR_BANK_SIZE;
            for i in 0..CHR_BANK_SIZE {
                cartridge.chr_rom[start + i] = (bank as u8).wrapping_mul(0x10);
            }
        }

        let mut mapper = Mapper1::new(cartridge);

        // Set to 4KB CHR mode (control bit 4 = 1)
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x00);
        mapper.cpu_write(0x8000, 0x01); // bit 4 = 1

        // Select CHR bank 0 for first 4KB
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);
        mapper.cpu_write(0xA000, 0x00);

        // Select CHR bank 1 for second 4KB
        mapper.cpu_write(0xC000, 0x01);
        mapper.cpu_write(0xC000, 0x00);
        mapper.cpu_write(0xC000, 0x00);
        mapper.cpu_write(0xC000, 0x00);
        mapper.cpu_write(0xC000, 0x00);

        // First half should be bank 0
        assert_eq!(mapper.ppu_read(0x0000), 0x00);
        // Second half should be bank 1
        assert_eq!(mapper.ppu_read(0x1000), 0x10);
    }

    #[test]
    fn test_prg_ram_read_write() {
        let cartridge = create_test_cartridge(16, 32);
        let mut mapper = Mapper1::new(cartridge);

        // Write to PRG-RAM
        mapper.cpu_write(0x6000, 0x42);
        mapper.cpu_write(0x7FFF, 0x99);

        // Read back
        assert_eq!(mapper.cpu_read(0x6000), 0x42);
        assert_eq!(mapper.cpu_read(0x7FFF), 0x99);
    }
}
