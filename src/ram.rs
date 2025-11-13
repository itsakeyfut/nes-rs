// RAM module - CPU internal RAM implementation
//
// This module implements the 2KB internal RAM for the NES CPU with proper mirroring.
// The RAM is located in the CPU address space at $0000-$07FF and is mirrored three times
// at $0800-$0FFF, $1000-$17FF, and $1800-$1FFF.
//
// # Memory Layout
//
// ```text
// $0000-$07FF: 2KB internal RAM (actual physical memory)
// $0800-$0FFF: Mirror of $0000-$07FF
// $1000-$17FF: Mirror of $0000-$07FF
// $1800-$1FFF: Mirror of $0000-$07FF
// ```
//
// # Zero Page and Stack
//
// - Zero Page: $0000-$00FF - Fast access memory area with special addressing modes
// - Stack: $0100-$01FF - Hardware stack used by the 6502 processor

use crate::bus::MemoryMappedDevice;

/// Size of the internal RAM in bytes (2KB)
const RAM_SIZE: usize = 2048; // 2KB = 2048 bytes = 0x0800 bytes

/// Mask for RAM address mirroring
///
/// The NES has 2KB of RAM but it's mapped to 8KB of address space.
/// Using AND with 0x07FF (2047) ensures addresses $0000-$1FFF all map to the same 2KB.
const RAM_MIRROR_MASK: u16 = 0x07FF;

/// CPU internal RAM (2KB with mirroring)
///
/// The RAM struct represents the NES's 2KB of internal RAM. It automatically handles
/// address mirroring so that any address in the range $0000-$1FFF maps to the actual
/// 2KB physical memory.
///
/// # Mirroring Behavior
///
/// The hardware mirrors the 2KB RAM across a 8KB address space ($0000-$1FFF).
/// This means:
/// - Writing to $0000 is the same as writing to $0800, $1000, or $1800
/// - Reading from $0100 returns the same value as $0900, $1100, or $1900
///
/// # Initialization
///
/// RAM can be initialized with a specific fill pattern. On real NES hardware, RAM
/// contains semi-random patterns at power-on. For deterministic emulation, we
/// default to 0xFF, but other patterns like 0x00 can be used.
///
/// # Examples
///
/// ```
/// use nes_rs::ram::Ram;
/// use nes_rs::bus::MemoryMappedDevice;
///
/// // Create RAM initialized to 0xFF
/// let mut ram = Ram::new();
///
/// // Write to base address
/// ram.write(0x0000, 0x42);
///
/// // Read from mirrored address - returns the same value
/// assert_eq!(ram.read(0x0800), 0x42);
/// assert_eq!(ram.read(0x1000), 0x42);
/// assert_eq!(ram.read(0x1800), 0x42);
/// ```
#[derive(Clone)]
pub struct Ram {
    /// Internal 2KB memory array
    ///
    /// This is the actual physical memory. All addresses in the range $0000-$1FFF
    /// map to this 2KB array through mirroring.
    memory: [u8; RAM_SIZE],
}

impl Ram {
    /// Create a new RAM instance initialized with 0xFF
    ///
    /// Initializes all 2KB of RAM to 0xFF, which is a common pattern for testing
    /// and provides deterministic behavior. On real hardware, RAM contains
    /// semi-random values at power-on.
    ///
    /// # Returns
    ///
    /// A new Ram instance with all bytes set to 0xFF
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let ram = Ram::new();
    /// assert_eq!(ram.read(0x0000), 0xFF); // Default initialization
    /// ```
    pub fn new() -> Self {
        Ram {
            memory: [0xFF; RAM_SIZE],
        }
    }

    /// Create a new RAM instance initialized with zeros
    ///
    /// Initializes all 2KB of RAM to 0x00. This can be useful for testing or
    /// debugging scenarios where you want a clean slate.
    ///
    /// # Returns
    ///
    /// A new Ram instance with all bytes set to 0x00
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let ram = Ram::with_zeros();
    /// assert_eq!(ram.read(0x0000), 0x00);
    /// ```
    pub fn with_zeros() -> Self {
        Ram {
            memory: [0x00; RAM_SIZE],
        }
    }

    /// Create a new RAM instance with a custom fill pattern
    ///
    /// Initializes all 2KB of RAM to a specified byte value.
    ///
    /// # Arguments
    ///
    /// * `fill_byte` - The byte value to fill the RAM with
    ///
    /// # Returns
    ///
    /// A new Ram instance with all bytes set to the specified value
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let ram = Ram::with_pattern(0xAA);
    /// assert_eq!(ram.read(0x0000), 0xAA);
    /// assert_eq!(ram.read(0x07FF), 0xAA);
    /// ```
    pub fn with_pattern(fill_byte: u8) -> Self {
        Ram {
            memory: [fill_byte; RAM_SIZE],
        }
    }

    /// Reset RAM to power-on state (all bytes set to 0xFF)
    ///
    /// Resets all 2KB of RAM to 0xFF, simulating a power cycle.
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let mut ram = Ram::with_zeros();
    /// ram.write(0x0000, 0x42);
    /// ram.reset();
    /// assert_eq!(ram.read(0x0000), 0xFF); // Back to power-on state
    /// ```
    pub fn reset(&mut self) {
        self.memory.fill(0xFF);
    }

    /// Get the size of RAM in bytes
    ///
    /// # Returns
    ///
    /// The size of RAM (always 2048 bytes for NES)
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    ///
    /// let ram = Ram::new();
    /// assert_eq!(ram.size(), 2048);
    /// ```
    pub const fn size(&self) -> usize {
        RAM_SIZE
    }

    /// Apply mirroring to an address
    ///
    /// Converts any address in the range $0000-$1FFF to the corresponding
    /// physical address in the 2KB RAM ($0000-$07FF).
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to mirror (typically $0000-$1FFF)
    ///
    /// # Returns
    ///
    /// The mirrored address within the 2KB range ($0000-$07FF)
    ///
    /// # Implementation Note
    ///
    /// Uses bitwise AND with 0x07FF to efficiently handle mirroring:
    /// - $0000 & 0x07FF = $0000
    /// - $0800 & 0x07FF = $0000
    /// - $1000 & 0x07FF = $0000
    /// - $1800 & 0x07FF = $0000
    /// - $0123 & 0x07FF = $0123
    /// - $0923 & 0x07FF = $0123
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    ///
    /// let ram = Ram::new();
    /// assert_eq!(ram.mirror_address(0x0000), 0x0000);
    /// assert_eq!(ram.mirror_address(0x0800), 0x0000);
    /// assert_eq!(ram.mirror_address(0x1000), 0x0000);
    /// assert_eq!(ram.mirror_address(0x1800), 0x0000);
    /// assert_eq!(ram.mirror_address(0x0123), 0x0123);
    /// assert_eq!(ram.mirror_address(0x1923), 0x0123);
    /// ```
    #[inline]
    pub const fn mirror_address(&self, addr: u16) -> u16 {
        addr & RAM_MIRROR_MASK
    }
}

impl MemoryMappedDevice for Ram {
    /// Read a byte from RAM
    ///
    /// Reads a byte from the RAM at the specified address. The address is automatically
    /// mirrored to the 2KB range, so any address in $0000-$1FFF is valid.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from (will be mirrored to $0000-$07FF)
    ///
    /// # Returns
    ///
    /// The byte value at the mirrored address
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let mut ram = Ram::new();
    /// ram.write(0x0000, 0x42);
    ///
    /// // All mirrored addresses return the same value
    /// assert_eq!(ram.read(0x0000), 0x42);
    /// assert_eq!(ram.read(0x0800), 0x42);
    /// assert_eq!(ram.read(0x1000), 0x42);
    /// assert_eq!(ram.read(0x1800), 0x42);
    /// ```
    fn read(&self, addr: u16) -> u8 {
        let mirrored_addr = self.mirror_address(addr) as usize;
        self.memory[mirrored_addr]
    }

    /// Write a byte to RAM
    ///
    /// Writes a byte to the RAM at the specified address. The address is automatically
    /// mirrored to the 2KB range, so writing to any address in $0000-$1FFF affects
    /// the same physical memory location.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to (will be mirrored to $0000-$07FF)
    /// * `data` - The byte value to write
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::ram::Ram;
    /// use nes_rs::bus::MemoryMappedDevice;
    ///
    /// let mut ram = Ram::new();
    ///
    /// // Write to a mirrored address
    /// ram.write(0x0800, 0x42);
    ///
    /// // Read from base address - gets the same value
    /// assert_eq!(ram.read(0x0000), 0x42);
    /// ```
    fn write(&mut self, addr: u16, data: u8) {
        let mirrored_addr = self.mirror_address(addr) as usize;
        self.memory[mirrored_addr] = data;
    }
}

impl Default for Ram {
    /// Create a default RAM instance
    ///
    /// Equivalent to `Ram::new()` - creates RAM initialized with 0xFF.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Initialization Tests
    // ========================================

    #[test]
    fn test_ram_new_initialization() {
        let ram = Ram::new();
        // Default initialization should be 0xFF
        assert_eq!(ram.read(0x0000), 0xFF);
        assert_eq!(ram.read(0x0400), 0xFF);
        assert_eq!(ram.read(0x07FF), 0xFF);
    }

    #[test]
    fn test_ram_with_zeros() {
        let ram = Ram::with_zeros();
        // Should be initialized to 0x00
        assert_eq!(ram.read(0x0000), 0x00);
        assert_eq!(ram.read(0x0400), 0x00);
        assert_eq!(ram.read(0x07FF), 0x00);
    }

    #[test]
    fn test_ram_with_pattern() {
        let ram = Ram::with_pattern(0xAA);
        // Should be initialized to the pattern
        assert_eq!(ram.read(0x0000), 0xAA);
        assert_eq!(ram.read(0x0400), 0xAA);
        assert_eq!(ram.read(0x07FF), 0xAA);
    }

    #[test]
    fn test_ram_default() {
        let ram = Ram::default();
        // Default should be same as new()
        assert_eq!(ram.read(0x0000), 0xFF);
    }

    #[test]
    fn test_ram_size() {
        let ram = Ram::new();
        assert_eq!(ram.size(), 2048);
    }

    // ========================================
    // Basic Read/Write Tests
    // ========================================

    #[test]
    fn test_basic_read_write() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0000, 0x42);
        assert_eq!(ram.read(0x0000), 0x42);
    }

    #[test]
    fn test_multiple_writes() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0000, 0x11);
        ram.write(0x0100, 0x22);
        ram.write(0x0200, 0x33);
        ram.write(0x07FF, 0x44);

        assert_eq!(ram.read(0x0000), 0x11);
        assert_eq!(ram.read(0x0100), 0x22);
        assert_eq!(ram.read(0x0200), 0x33);
        assert_eq!(ram.read(0x07FF), 0x44);
    }

    #[test]
    fn test_overwrite() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0100, 0xAA);
        assert_eq!(ram.read(0x0100), 0xAA);

        ram.write(0x0100, 0xBB);
        assert_eq!(ram.read(0x0100), 0xBB);
    }

    // ========================================
    // Mirroring Tests
    // ========================================

    #[test]
    fn test_mirror_address_function() {
        let ram = Ram::new();

        // Base addresses should remain unchanged
        assert_eq!(ram.mirror_address(0x0000), 0x0000);
        assert_eq!(ram.mirror_address(0x0123), 0x0123);
        assert_eq!(ram.mirror_address(0x07FF), 0x07FF);

        // First mirror ($0800-$0FFF)
        assert_eq!(ram.mirror_address(0x0800), 0x0000);
        assert_eq!(ram.mirror_address(0x0923), 0x0123);
        assert_eq!(ram.mirror_address(0x0FFF), 0x07FF);

        // Second mirror ($1000-$17FF)
        assert_eq!(ram.mirror_address(0x1000), 0x0000);
        assert_eq!(ram.mirror_address(0x1123), 0x0123);
        assert_eq!(ram.mirror_address(0x17FF), 0x07FF);

        // Third mirror ($1800-$1FFF)
        assert_eq!(ram.mirror_address(0x1800), 0x0000);
        assert_eq!(ram.mirror_address(0x1923), 0x0123);
        assert_eq!(ram.mirror_address(0x1FFF), 0x07FF);
    }

    #[test]
    fn test_mirroring_first_mirror() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0000, 0x42);
        assert_eq!(ram.read(0x0800), 0x42, "First mirror ($0800)");
    }

    #[test]
    fn test_mirroring_second_mirror() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0100, 0x55);
        assert_eq!(ram.read(0x1100), 0x55, "Second mirror ($1100)");
    }

    #[test]
    fn test_mirroring_third_mirror() {
        let mut ram = Ram::with_zeros();
        ram.write(0x0200, 0x88);
        assert_eq!(ram.read(0x1A00), 0x88, "Third mirror ($1A00)");
    }

    #[test]
    fn test_mirroring_bidirectional_write_to_mirror() {
        let mut ram = Ram::with_zeros();
        // Write to mirror
        ram.write(0x0800, 0x99);
        // Read from base
        assert_eq!(ram.read(0x0000), 0x99, "Mirror write affects base");
    }

    #[test]
    fn test_mirroring_bidirectional_all_mirrors() {
        let mut ram = Ram::with_zeros();
        let test_addr = 0x0123;

        // Write to base
        ram.write(test_addr, 0xAB);

        // Read from all mirrors
        assert_eq!(ram.read(test_addr), 0xAB, "Base address");
        assert_eq!(ram.read(test_addr + 0x0800), 0xAB, "First mirror");
        assert_eq!(ram.read(test_addr + 0x1000), 0xAB, "Second mirror");
        assert_eq!(ram.read(test_addr + 0x1800), 0xAB, "Third mirror");
    }

    #[test]
    fn test_mirroring_write_to_different_mirrors() {
        let mut ram = Ram::with_zeros();

        // Write to different mirrors of the same address
        ram.write(0x0100, 0x11);
        ram.write(0x0900, 0x22); // Overwrites via first mirror
        ram.write(0x1100, 0x33); // Overwrites via second mirror
        ram.write(0x1900, 0x44); // Overwrites via third mirror

        // All mirrors should reflect the last write
        assert_eq!(ram.read(0x0100), 0x44);
        assert_eq!(ram.read(0x0900), 0x44);
        assert_eq!(ram.read(0x1100), 0x44);
        assert_eq!(ram.read(0x1900), 0x44);
    }

    #[test]
    fn test_mirroring_boundary_addresses() {
        let mut ram = Ram::with_zeros();

        // Test start boundaries
        ram.write(0x0000, 0x11);
        assert_eq!(ram.read(0x0800), 0x11);
        assert_eq!(ram.read(0x1000), 0x11);
        assert_eq!(ram.read(0x1800), 0x11);

        // Test end boundaries
        ram.write(0x07FF, 0x22);
        assert_eq!(ram.read(0x0FFF), 0x22);
        assert_eq!(ram.read(0x17FF), 0x22);
        assert_eq!(ram.read(0x1FFF), 0x22);
    }

    // ========================================
    // Zero Page Tests
    // ========================================

    #[test]
    fn test_zero_page_access() {
        let mut ram = Ram::with_zeros();

        // Zero page is $0000-$00FF
        ram.write(0x0000, 0xAA);
        ram.write(0x0050, 0xBB);
        ram.write(0x00FF, 0xCC);

        assert_eq!(ram.read(0x0000), 0xAA);
        assert_eq!(ram.read(0x0050), 0xBB);
        assert_eq!(ram.read(0x00FF), 0xCC);
    }

    #[test]
    fn test_zero_page_mirroring() {
        let mut ram = Ram::with_zeros();

        // Write to zero page
        ram.write(0x0042, 0x99);

        // Read from mirrored zero page addresses
        assert_eq!(ram.read(0x0842), 0x99);
        assert_eq!(ram.read(0x1042), 0x99);
        assert_eq!(ram.read(0x1842), 0x99);
    }

    // ========================================
    // Stack Tests
    // ========================================

    #[test]
    fn test_stack_access() {
        let mut ram = Ram::with_zeros();

        // Stack is $0100-$01FF
        ram.write(0x0100, 0x11);
        ram.write(0x01FF, 0x22);
        ram.write(0x01FD, 0x33);

        assert_eq!(ram.read(0x0100), 0x11);
        assert_eq!(ram.read(0x01FF), 0x22);
        assert_eq!(ram.read(0x01FD), 0x33);
    }

    #[test]
    fn test_stack_mirroring() {
        let mut ram = Ram::with_zeros();

        // Write to stack
        ram.write(0x01FD, 0x77);

        // Read from mirrored stack addresses
        assert_eq!(ram.read(0x09FD), 0x77);
        assert_eq!(ram.read(0x11FD), 0x77);
        assert_eq!(ram.read(0x19FD), 0x77);
    }

    #[test]
    fn test_stack_operations_simulation() {
        let mut ram = Ram::with_zeros();

        // Simulate stack push operations
        let mut sp: u8 = 0xFF; // Stack pointer starts at $01FF

        // Push values
        ram.write(0x0100 + sp as u16, 0x11);
        sp = sp.wrapping_sub(1);
        ram.write(0x0100 + sp as u16, 0x22);
        sp = sp.wrapping_sub(1);
        ram.write(0x0100 + sp as u16, 0x33);

        // Verify stack contents
        assert_eq!(ram.read(0x01FF), 0x11);
        assert_eq!(ram.read(0x01FE), 0x22);
        assert_eq!(ram.read(0x01FD), 0x33);
    }

    // ========================================
    // Reset Tests
    // ========================================

    #[test]
    fn test_reset() {
        let mut ram = Ram::with_zeros();

        // Write some values
        ram.write(0x0000, 0x42);
        ram.write(0x0100, 0x43);
        ram.write(0x0200, 0x44);

        // Reset
        ram.reset();

        // All values should be 0xFF now
        assert_eq!(ram.read(0x0000), 0xFF);
        assert_eq!(ram.read(0x0100), 0xFF);
        assert_eq!(ram.read(0x0200), 0xFF);
        assert_eq!(ram.read(0x07FF), 0xFF);
    }

    // ========================================
    // Comprehensive Tests
    // ========================================

    #[test]
    fn test_full_address_range() {
        let mut ram = Ram::with_zeros();

        // Test writing to various addresses across the full mirrored range
        let test_cases = [
            (0x0000, 0x01),
            (0x0100, 0x02),
            (0x0200, 0x03),
            (0x0400, 0x04),
            (0x0800, 0x05), // First mirror
            (0x1000, 0x06), // Second mirror
            (0x1800, 0x07), // Third mirror
            (0x1FFF, 0x08), // Last address
        ];

        for (addr, value) in test_cases {
            ram.write(addr, value);
            assert_eq!(ram.read(addr), value, "Failed at address ${:04X}", addr);
        }
    }

    #[test]
    fn test_mirror_independence() {
        let mut ram = Ram::with_zeros();

        // Write to different non-mirrored addresses
        ram.write(0x0000, 0x11);
        ram.write(0x0001, 0x22);
        ram.write(0x0002, 0x33);

        // Verify they don't interfere with each other
        assert_eq!(ram.read(0x0000), 0x11);
        assert_eq!(ram.read(0x0001), 0x22);
        assert_eq!(ram.read(0x0002), 0x33);

        // Verify mirrors work correctly
        assert_eq!(ram.read(0x0800), 0x11);
        assert_eq!(ram.read(0x0801), 0x22);
        assert_eq!(ram.read(0x0802), 0x33);
    }

    #[test]
    fn test_all_bytes_accessible() {
        let mut ram = Ram::with_zeros();

        // Write unique value to each byte in the 2KB range
        for addr in 0..RAM_SIZE {
            let value = (addr & 0xFF) as u8;
            ram.write(addr as u16, value);
        }

        // Verify all bytes can be read back correctly
        for addr in 0..RAM_SIZE {
            let expected = (addr & 0xFF) as u8;
            assert_eq!(
                ram.read(addr as u16),
                expected,
                "Failed at address ${:04X}",
                addr
            );
        }
    }

    #[test]
    fn test_clone() {
        let mut ram1 = Ram::with_zeros();
        ram1.write(0x0000, 0x42);
        ram1.write(0x0100, 0x43);

        let ram2 = ram1.clone();

        // Verify clone has same data
        assert_eq!(ram2.read(0x0000), 0x42);
        assert_eq!(ram2.read(0x0100), 0x43);
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_all_possible_byte_values() {
        let mut ram = Ram::with_zeros();

        // Test writing and reading all possible byte values
        for value in 0..=255u8 {
            ram.write(0x0000, value);
            assert_eq!(ram.read(0x0000), value, "Failed for value {}", value);
        }
    }

    #[test]
    fn test_sequential_access_pattern() {
        let mut ram = Ram::with_zeros();

        // Sequential write
        for i in 0..256 {
            ram.write(i, i as u8);
        }

        // Sequential read
        for i in 0..256 {
            assert_eq!(ram.read(i), i as u8, "Failed at offset {}", i);
        }
    }

    #[test]
    fn test_mirroring_with_high_addresses() {
        let mut ram = Ram::with_zeros();

        // Test mirroring works correctly with addresses beyond 2KB
        ram.write(0x1FFF, 0x99); // Last byte of mirrored range
        assert_eq!(ram.read(0x07FF), 0x99); // Should map to last byte of physical RAM
        assert_eq!(ram.read(0x0FFF), 0x99);
        assert_eq!(ram.read(0x17FF), 0x99);
    }
}
