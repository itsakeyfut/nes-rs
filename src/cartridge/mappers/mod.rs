// Mappers module - Implementations of various NES cartridge mappers
//
// This module contains the mapper factory and individual mapper implementations.
// Each mapper handles memory mapping and banking for different cartridge types.

mod mapper0;

use super::{Cartridge, Mapper};
use mapper0::Mapper0;

/// Error type for mapper creation
#[derive(Debug)]
pub enum MapperError {
    /// The requested mapper number is not supported
    UnsupportedMapper(u8),
    /// Invalid cartridge configuration for the mapper
    InvalidConfiguration(String),
}

impl std::fmt::Display for MapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapperError::UnsupportedMapper(num) => {
                write!(f, "Mapper {} is not supported", num)
            }
            MapperError::InvalidConfiguration(msg) => {
                write!(f, "Invalid mapper configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for MapperError {}

/// Create a mapper instance based on the mapper number in the cartridge
///
/// This factory function creates the appropriate mapper implementation for the
/// given cartridge. The mapper number is determined from the iNES header.
///
/// # Arguments
/// * `cartridge` - The cartridge to create a mapper for
///
/// # Returns
/// A boxed trait object implementing the Mapper trait
///
/// # Errors
/// Returns `MapperError::UnsupportedMapper` if the mapper number is not implemented
///
/// # Example
/// ```no_run
/// use nes_rs::Cartridge;
/// use nes_rs::cartridge::mappers::create_mapper;
///
/// let cartridge = Cartridge::from_ines_file("game.nes").unwrap();
/// let mapper = create_mapper(cartridge).unwrap();
/// ```
pub fn create_mapper(cartridge: Cartridge) -> Result<Box<dyn Mapper>, MapperError> {
    match cartridge.mapper {
        0 => Ok(Box::new(Mapper0::new(cartridge))),
        // Future mapper implementations will be added here:
        // 1 => Ok(Box::new(Mapper1::new(cartridge))),
        // 2 => Ok(Box::new(Mapper2::new(cartridge))),
        // 3 => Ok(Box::new(Mapper3::new(cartridge))),
        // 4 => Ok(Box::new(Mapper4::new(cartridge))),
        mapper_num => Err(MapperError::UnsupportedMapper(mapper_num)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::Mirroring;

    #[test]
    fn test_mapper0_creation() {
        // Create a cartridge with Mapper 0 configuration
        let cartridge = Cartridge {
            prg_rom: vec![0xAA; 16 * 1024], // 16KB PRG-ROM
            chr_rom: vec![0xBB; 8 * 1024],  // 8KB CHR-ROM
            trainer: None,
            mapper: 0,
            mirroring: Mirroring::Horizontal,
            has_battery: false,
        };

        let result = create_mapper(cartridge);
        assert!(result.is_ok());

        let mapper = result.unwrap();
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn test_unsupported_mapper() {
        let mut cartridge = Cartridge::new();
        cartridge.mapper = 99; // Non-existent mapper

        let result = create_mapper(cartridge);
        assert!(matches!(result, Err(MapperError::UnsupportedMapper(99))));
    }
}
