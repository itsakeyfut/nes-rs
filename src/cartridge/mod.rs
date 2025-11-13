// Cartridge module - ROM loading and mapper implementation
// This module will contain cartridge and mapper implementations

use std::io::{self, Read};

/// iNES file format magic number: "NES" + MS-DOS EOF
const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/// Size of iNES header in bytes
const INES_HEADER_SIZE: usize = 16;

/// Size of trainer data in bytes (if present)
const TRAINER_SIZE: usize = 512;

/// Size of PRG-ROM bank in bytes (16KB)
const PRG_ROM_BANK_SIZE: usize = 16 * 1024;

/// Size of CHR-ROM bank in bytes (8KB)
const CHR_ROM_BANK_SIZE: usize = 8 * 1024;

/// Mirroring type for nametables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    /// Horizontal mirroring (vertical arrangement)
    Horizontal,
    /// Vertical mirroring (horizontal arrangement)
    Vertical,
    /// Four-screen VRAM
    FourScreen,
}

/// iNES ROM format errors
#[derive(Debug)]
pub enum INesError {
    /// Invalid magic number
    InvalidMagic,
    /// I/O error while reading
    IoError(io::Error),
    /// File too small to be a valid iNES file
    FileTooSmall,
    /// Invalid file size (doesn't match header specifications)
    InvalidFileSize,
    /// Unsupported format (iNES 2.0 is not currently supported)
    UnsupportedFormat,
}

impl From<io::Error> for INesError {
    fn from(err: io::Error) -> Self {
        INesError::IoError(err)
    }
}

impl std::fmt::Display for INesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            INesError::InvalidMagic => write!(f, "Invalid iNES magic number"),
            INesError::IoError(e) => write!(f, "I/O error: {}", e),
            INesError::FileTooSmall => write!(f, "File too small to be a valid iNES file"),
            INesError::InvalidFileSize => {
                write!(f, "File size doesn't match header specifications")
            }
            INesError::UnsupportedFormat => {
                write!(
                    f,
                    "Unsupported format: iNES 2.0 is not currently supported, only iNES 1.0"
                )
            }
        }
    }
}

impl std::error::Error for INesError {}

/// iNES header structure (16 bytes)
#[derive(Debug, Clone)]
pub struct INesHeader {
    /// PRG-ROM size in 16KB units
    pub prg_rom_banks: u8,
    /// CHR-ROM size in 8KB units (0 = CHR-RAM)
    pub chr_rom_banks: u8,
    /// Flags 6
    pub flags6: u8,
    /// Flags 7
    pub flags7: u8,
    /// PRG-RAM size in 8KB units (0 = 8KB for compatibility)
    pub prg_ram_size: u8,
    /// Flags 9 (TV system)
    pub flags9: u8,
    /// Flags 10 (unofficial)
    pub flags10: u8,
}

impl INesHeader {
    /// Parse iNES header from 16 bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, INesError> {
        if bytes.len() < INES_HEADER_SIZE {
            return Err(INesError::FileTooSmall);
        }

        // Validate magic number
        if bytes[0..4] != INES_MAGIC {
            return Err(INesError::InvalidMagic);
        }

        Ok(INesHeader {
            prg_rom_banks: bytes[4],
            chr_rom_banks: bytes[5],
            flags6: bytes[6],
            flags7: bytes[7],
            prg_ram_size: bytes[8],
            flags9: bytes[9],
            flags10: bytes[10],
        })
    }

    /// Get mapper number (combined from flags 6 and 7)
    pub fn mapper_number(&self) -> u8 {
        (self.flags7 & 0xF0) | (self.flags6 >> 4)
    }

    /// Get mirroring type
    pub fn mirroring(&self) -> Mirroring {
        if self.flags6 & 0x08 != 0 {
            // Four-screen VRAM
            Mirroring::FourScreen
        } else if self.flags6 & 0x01 != 0 {
            // Vertical mirroring
            Mirroring::Vertical
        } else {
            // Horizontal mirroring
            Mirroring::Horizontal
        }
    }

    /// Check if battery-backed RAM is present
    pub fn has_battery(&self) -> bool {
        self.flags6 & 0x02 != 0
    }

    /// Check if trainer is present (512 bytes before PRG-ROM)
    pub fn has_trainer(&self) -> bool {
        self.flags6 & 0x04 != 0
    }

    /// Check if this is iNES 2.0 format
    ///
    /// iNES 2.0 files have different header interpretations and are currently not supported.
    /// Files detected as iNES 2.0 will be rejected during loading.
    pub fn is_ines2(&self) -> bool {
        (self.flags7 & 0x0C) == 0x08
    }
}

/// Cartridge structure representing a loaded ROM
pub struct Cartridge {
    /// PRG-ROM data (program memory)
    pub prg_rom: Vec<u8>,
    /// CHR-ROM data (character/pattern memory)
    pub chr_rom: Vec<u8>,
    /// Trainer data (if present)
    pub trainer: Option<Vec<u8>>,
    /// Mapper number
    pub mapper: u8,
    /// Mirroring type
    pub mirroring: Mirroring,
    /// Battery-backed RAM present
    pub has_battery: bool,
}

impl Cartridge {
    /// Create a new empty cartridge
    pub fn new() -> Self {
        Cartridge {
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            trainer: None,
            mapper: 0,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        }
    }

    /// Load a ROM from iNES format bytes
    ///
    /// Note: Only iNES 1.0 format is currently supported. iNES 2.0 files will be rejected.
    pub fn from_ines_bytes(data: &[u8]) -> Result<Self, INesError> {
        if data.len() < INES_HEADER_SIZE {
            return Err(INesError::FileTooSmall);
        }

        // Parse header
        let header = INesHeader::from_bytes(&data[0..INES_HEADER_SIZE])?;

        // Check for unsupported iNES 2.0 format
        if header.is_ines2() {
            return Err(INesError::UnsupportedFormat);
        }

        // Calculate expected sizes
        let prg_rom_size = header.prg_rom_banks as usize * PRG_ROM_BANK_SIZE;
        let chr_rom_size = header.chr_rom_banks as usize * CHR_ROM_BANK_SIZE;
        let trainer_size = if header.has_trainer() {
            TRAINER_SIZE
        } else {
            0
        };

        let expected_size = INES_HEADER_SIZE + trainer_size + prg_rom_size + chr_rom_size;

        // Validate file size
        if data.len() < expected_size {
            return Err(INesError::InvalidFileSize);
        }

        let mut offset = INES_HEADER_SIZE;

        // Load trainer data if present
        let trainer = if header.has_trainer() {
            let trainer_data = data[offset..offset + TRAINER_SIZE].to_vec();
            offset += TRAINER_SIZE;
            Some(trainer_data)
        } else {
            None
        };

        // Load PRG-ROM data
        let prg_rom = data[offset..offset + prg_rom_size].to_vec();
        offset += prg_rom_size;

        // Load CHR-ROM data (or allocate CHR-RAM if size is 0)
        let chr_rom = if chr_rom_size > 0 {
            data[offset..offset + chr_rom_size].to_vec()
        } else {
            // CHR-RAM: allocate 8KB
            vec![0; CHR_ROM_BANK_SIZE]
        };

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            trainer,
            mapper: header.mapper_number(),
            mirroring: header.mirroring(),
            has_battery: header.has_battery(),
        })
    }

    /// Load a ROM from a reader implementing Read
    pub fn from_ines_reader<R: Read>(mut reader: R) -> Result<Self, INesError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::from_ines_bytes(&data)
    }

    /// Load a ROM from a file path
    ///
    /// # Example
    /// ```no_run
    /// use nes_rs::Cartridge;
    ///
    /// let cartridge = Cartridge::from_ines_file("path/to/rom.nes").unwrap();
    /// println!("Mapper: {}", cartridge.mapper);
    /// println!("PRG-ROM size: {} bytes", cartridge.prg_rom_size());
    /// println!("CHR-ROM size: {} bytes", cartridge.chr_rom_size());
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_ines_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, INesError> {
        use std::fs::File;
        let file = File::open(path)?;
        Self::from_ines_reader(file)
    }

    /// Get PRG-ROM size in bytes
    pub fn prg_rom_size(&self) -> usize {
        self.prg_rom.len()
    }

    /// Get CHR-ROM size in bytes
    pub fn chr_rom_size(&self) -> usize {
        self.chr_rom.len()
    }

    /// Check if trainer is present
    pub fn has_trainer(&self) -> bool {
        self.trainer.is_some()
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a minimal valid iNES header
    fn create_test_header(
        prg_banks: u8,
        chr_banks: u8,
        mapper: u8,
        mirroring: Mirroring,
        has_trainer: bool,
        has_battery: bool,
    ) -> Vec<u8> {
        let mut header = vec![0u8; INES_HEADER_SIZE];

        // Magic number
        header[0..4].copy_from_slice(&INES_MAGIC);

        // PRG and CHR ROM sizes
        header[4] = prg_banks;
        header[5] = chr_banks;

        // Flags 6
        let mut flags6 = mapper << 4;
        if mirroring == Mirroring::Vertical {
            flags6 |= 0x01;
        }
        if has_battery {
            flags6 |= 0x02;
        }
        if has_trainer {
            flags6 |= 0x04;
        }
        if mirroring == Mirroring::FourScreen {
            flags6 |= 0x08;
        }
        header[6] = flags6;

        // Flags 7
        header[7] = mapper & 0xF0;

        header
    }

    #[test]
    fn test_ines_header_validation() {
        // Test valid header
        let valid_header = create_test_header(2, 1, 0, Mirroring::Horizontal, false, false);
        assert!(INesHeader::from_bytes(&valid_header).is_ok());

        // Test invalid magic number
        let mut invalid_header = valid_header.clone();
        invalid_header[0] = 0xFF;
        assert!(matches!(
            INesHeader::from_bytes(&invalid_header),
            Err(INesError::InvalidMagic)
        ));

        // Test too small
        assert!(matches!(
            INesHeader::from_bytes(&[0u8; 10]),
            Err(INesError::FileTooSmall)
        ));
    }

    #[test]
    fn test_mapper_number_extraction() {
        // Test Mapper 0 (NROM)
        let header = create_test_header(2, 1, 0, Mirroring::Horizontal, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mapper_number(), 0);

        // Test Mapper 1 (MMC1)
        let header = create_test_header(2, 1, 1, Mirroring::Horizontal, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mapper_number(), 1);

        // Test Mapper 4 (MMC3)
        let header = create_test_header(2, 1, 4, Mirroring::Horizontal, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mapper_number(), 4);
    }

    #[test]
    fn test_mirroring_detection() {
        // Test horizontal mirroring
        let header = create_test_header(2, 1, 0, Mirroring::Horizontal, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mirroring(), Mirroring::Horizontal);

        // Test vertical mirroring
        let header = create_test_header(2, 1, 0, Mirroring::Vertical, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mirroring(), Mirroring::Vertical);

        // Test four-screen VRAM
        let header = create_test_header(2, 1, 0, Mirroring::FourScreen, false, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert_eq!(parsed.mirroring(), Mirroring::FourScreen);
    }

    #[test]
    fn test_flags_detection() {
        // Test battery flag
        let header = create_test_header(2, 1, 0, Mirroring::Horizontal, false, true);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert!(parsed.has_battery());

        // Test trainer flag
        let header = create_test_header(2, 1, 0, Mirroring::Horizontal, true, false);
        let parsed = INesHeader::from_bytes(&header).unwrap();
        assert!(parsed.has_trainer());
    }

    #[test]
    fn test_cartridge_loading_basic() {
        // Create a minimal ROM: header + 32KB PRG-ROM + 8KB CHR-ROM
        let mut rom_data = create_test_header(2, 1, 0, Mirroring::Horizontal, false, false);

        // Add PRG-ROM data (32KB = 2 banks)
        rom_data.extend(vec![0xAA; 32 * 1024]);

        // Add CHR-ROM data (8KB = 1 bank)
        rom_data.extend(vec![0xBB; 8 * 1024]);

        let cartridge = Cartridge::from_ines_bytes(&rom_data).unwrap();

        assert_eq!(cartridge.mapper, 0);
        assert_eq!(cartridge.mirroring, Mirroring::Horizontal);
        assert_eq!(cartridge.prg_rom_size(), 32 * 1024);
        assert_eq!(cartridge.chr_rom_size(), 8 * 1024);
        assert!(!cartridge.has_trainer());
        assert!(!cartridge.has_battery);
    }

    #[test]
    fn test_cartridge_loading_with_trainer() {
        // Create a ROM with trainer data
        let mut rom_data = create_test_header(1, 1, 0, Mirroring::Horizontal, true, false);

        // Add trainer data (512 bytes)
        rom_data.extend(vec![0x11; TRAINER_SIZE]);

        // Add PRG-ROM data (16KB = 1 bank)
        rom_data.extend(vec![0xAA; 16 * 1024]);

        // Add CHR-ROM data (8KB = 1 bank)
        rom_data.extend(vec![0xBB; 8 * 1024]);

        let cartridge = Cartridge::from_ines_bytes(&rom_data).unwrap();

        assert!(cartridge.has_trainer());
        assert_eq!(cartridge.trainer.as_ref().unwrap().len(), TRAINER_SIZE);
        assert_eq!(cartridge.prg_rom_size(), 16 * 1024);
        assert_eq!(cartridge.chr_rom_size(), 8 * 1024);
    }

    #[test]
    fn test_cartridge_chr_ram() {
        // Create a ROM with CHR-RAM (chr_banks = 0)
        let mut rom_data = create_test_header(1, 0, 0, Mirroring::Horizontal, false, false);

        // Add PRG-ROM data (16KB = 1 bank)
        rom_data.extend(vec![0xAA; 16 * 1024]);

        let cartridge = Cartridge::from_ines_bytes(&rom_data).unwrap();

        // CHR-RAM should be allocated (8KB)
        assert_eq!(cartridge.chr_rom_size(), 8 * 1024);
    }

    #[test]
    fn test_cartridge_invalid_size() {
        // Create a header that claims more data than is present
        let rom_data = create_test_header(2, 1, 0, Mirroring::Horizontal, false, false);
        // Only header, no PRG or CHR data

        let result = Cartridge::from_ines_bytes(&rom_data);
        assert!(matches!(result, Err(INesError::InvalidFileSize)));
    }

    #[test]
    fn test_cartridge_initialization() {
        let cartridge = Cartridge::new();
        assert_eq!(cartridge.prg_rom_size(), 0);
        assert_eq!(cartridge.chr_rom_size(), 0);
        assert!(!cartridge.has_trainer());
    }

    #[test]
    fn test_ines2_format_rejected() {
        // Create a valid iNES 2.0 header
        let mut header = vec![0u8; INES_HEADER_SIZE];
        header[0..4].copy_from_slice(&INES_MAGIC);
        header[4] = 2; // PRG-ROM banks
        header[5] = 1; // CHR-ROM banks
        header[6] = 0; // Flags 6
        header[7] = 0x08; // Flags 7: bits 2-3 = 10 indicates iNES 2.0

        // Create complete ROM data
        let mut rom_data = header;
        rom_data.extend(vec![0xAA; 32 * 1024]); // PRG-ROM
        rom_data.extend(vec![0xBB; 8 * 1024]); // CHR-ROM

        // Verify the header is detected as iNES 2.0
        let parsed_header = INesHeader::from_bytes(&rom_data[0..INES_HEADER_SIZE]).unwrap();
        assert!(parsed_header.is_ines2());

        // Attempt to load should fail with UnsupportedFormat
        let result = Cartridge::from_ines_bytes(&rom_data);
        assert!(
            matches!(result, Err(INesError::UnsupportedFormat)),
            "Expected UnsupportedFormat error for iNES 2.0 file"
        );
    }
}
