// Memory Viewer - Inspect CPU and PPU memory
//
// Provides:
// - Hex dump of memory regions
// - Memory search
// - Memory comparison

use crate::bus::Bus;
use crate::ppu::Ppu;
use std::collections::HashMap;

/// Memory region to view
///
/// Note: Pattern tables are stored in CHR ROM/RAM on the cartridge and
/// should be accessed through the mapper interface, not the PPU memory viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// CPU address space ($0000-$FFFF)
    Cpu,
    /// PPU nametables ($2000-$2FFF)
    PpuNametables,
    /// PPU palette RAM ($3F00-$3F1F)
    PpuPalette,
    /// PPU OAM (Object Attribute Memory)
    PpuOam,
}

/// CPU memory region type for color coding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuMemoryRegionType {
    /// Internal RAM ($0000-$07FF)
    Ram,
    /// Stack ($0100-$01FF)
    Stack,
    /// PPU Registers ($2000-$3FFF)
    PpuRegisters,
    /// APU and I/O Registers ($4000-$4017)
    ApuIo,
    /// Cartridge space ($4020-$FFFF)
    Cartridge,
    /// Other/unmapped
    Other,
}

/// Memory viewer
///
/// Provides tools for inspecting and dumping memory contents.
pub struct MemoryViewer {
    /// Number of bytes per row in hex dump
    bytes_per_row: usize,

    /// Track modified bytes (address -> frame count when modified)
    modified_bytes: HashMap<u16, u32>,

    /// Current frame count for tracking modifications
    frame_count: u32,

    /// Number of frames to keep highlighting modified bytes
    modification_highlight_frames: u32,
}

impl MemoryViewer {
    /// Create a new memory viewer
    ///
    /// # Returns
    ///
    /// A new memory viewer instance
    pub fn new() -> Self {
        MemoryViewer {
            bytes_per_row: 16,
            modified_bytes: HashMap::new(),
            frame_count: 0,
            modification_highlight_frames: 60, // Highlight for 1 second at 60 FPS
        }
    }

    /// Determine the memory region type for a CPU address
    ///
    /// # Arguments
    ///
    /// * `addr` - The CPU address
    ///
    /// # Returns
    ///
    /// The memory region type for color coding
    #[inline(always)]
    pub fn get_cpu_region_type(addr: u16) -> CpuMemoryRegionType {
        match addr {
            0x0000..=0x07FF => {
                // Check if it's in the stack range
                if (0x0100..=0x01FF).contains(&addr) {
                    CpuMemoryRegionType::Stack
                } else {
                    CpuMemoryRegionType::Ram
                }
            }
            0x0800..=0x1FFF => {
                // RAM mirrors - map to actual address
                let mirrored_addr = addr & 0x07FF;
                if (0x0100..=0x01FF).contains(&mirrored_addr) {
                    CpuMemoryRegionType::Stack
                } else {
                    CpuMemoryRegionType::Ram
                }
            }
            0x2000..=0x3FFF => CpuMemoryRegionType::PpuRegisters,
            0x4000..=0x4017 => CpuMemoryRegionType::ApuIo,
            0x4020..=0xFFFF => CpuMemoryRegionType::Cartridge,
            _ => CpuMemoryRegionType::Other,
        }
    }

    /// Mark a byte as modified
    ///
    /// # Arguments
    ///
    /// * `addr` - The address that was modified
    pub fn mark_modified(&mut self, addr: u16) {
        self.modified_bytes.insert(addr, self.frame_count);
    }

    /// Check if a byte was recently modified
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to check
    ///
    /// # Returns
    ///
    /// `true` if the byte was modified within the highlight window
    pub fn is_recently_modified(&self, addr: u16) -> bool {
        if let Some(&modified_frame) = self.modified_bytes.get(&addr) {
            self.frame_count.saturating_sub(modified_frame) < self.modification_highlight_frames
        } else {
            false
        }
    }

    /// Advance the frame counter and clean up old modifications
    pub fn advance_frame(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);

        // Clean up old entries
        self.modified_bytes.retain(|_, &mut frame| {
            self.frame_count.saturating_sub(frame) < self.modification_highlight_frames
        });
    }

    /// Clear all modification tracking
    pub fn clear_modifications(&mut self) {
        self.modified_bytes.clear();
    }

    /// Set the number of bytes per row in hex dumps
    ///
    /// # Arguments
    ///
    /// * `bytes` - Number of bytes per row (typically 8, 16, or 32)
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is 0
    pub fn set_bytes_per_row(&mut self, bytes: usize) {
        assert!(bytes > 0, "bytes_per_row must be greater than 0");
        self.bytes_per_row = bytes;
    }

    /// Create a hex dump of CPU memory
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    /// * `start` - Start address
    /// * `length` - Number of bytes to dump
    ///
    /// # Returns
    ///
    /// A formatted hex dump string
    ///
    /// # Example
    ///
    /// ```ignore
    /// use nes_rs::debug::MemoryViewer;
    /// use nes_rs::bus::Bus;
    ///
    /// let mut bus = Bus::new();
    /// let viewer = MemoryViewer::new();
    /// let dump = viewer.dump_cpu_memory(&mut bus, 0x0000, 256);
    /// println!("{}", dump);
    /// ```
    pub fn dump_cpu_memory(&self, bus: &mut Bus, start: u16, length: usize) -> String {
        let mut output = String::new();

        let rows = length.div_ceil(self.bytes_per_row);

        for row in 0..rows {
            let addr = start.wrapping_add((row * self.bytes_per_row) as u16);
            output.push_str(&format!("${:04X}:  ", addr));

            // Hex bytes
            for col in 0..self.bytes_per_row {
                let offset = row * self.bytes_per_row + col;
                if offset < length {
                    let byte_addr = start.wrapping_add(offset as u16);
                    let byte = bus.read(byte_addr);
                    output.push_str(&format!("{:02X} ", byte));
                } else {
                    output.push_str("   ");
                }
            }

            output.push_str(" | ");

            // ASCII representation
            for col in 0..self.bytes_per_row {
                let offset = row * self.bytes_per_row + col;
                if offset < length {
                    let byte_addr = start.wrapping_add(offset as u16);
                    let byte = bus.read(byte_addr);
                    if (0x20..=0x7E).contains(&byte) {
                        output.push(byte as char);
                    } else {
                        output.push('.');
                    }
                } else {
                    output.push(' ');
                }
            }

            output.push('\n');
        }

        output
    }

    /// Create a hex dump of PPU memory
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    /// * `region` - Which PPU memory region to dump
    /// * `start` - Start offset within the region
    /// * `length` - Number of bytes to dump
    ///
    /// # Returns
    ///
    /// A formatted hex dump string
    pub fn dump_ppu_memory(
        &self,
        ppu: &Ppu,
        region: MemoryRegion,
        start: usize,
        length: usize,
    ) -> String {
        let mut output = String::new();

        let data = match region {
            MemoryRegion::PpuNametables => {
                let mut data = Vec::new();
                for i in start..std::cmp::min(start + length, ppu.nametables.len()) {
                    data.push(ppu.nametables[i]);
                }
                data
            }
            MemoryRegion::PpuPalette => {
                let mut data = Vec::new();
                for i in start..std::cmp::min(start + length, ppu.palette_ram.len()) {
                    data.push(ppu.palette_ram[i]);
                }
                data
            }
            MemoryRegion::PpuOam => {
                let mut data = Vec::new();
                for i in start..std::cmp::min(start + length, 256) {
                    data.push(ppu.read_oam(i as u8));
                }
                data
            }
            MemoryRegion::Cpu => {
                // CPU memory is not accessible through PPU
                Vec::new()
            }
        };

        let rows = data.len().div_ceil(self.bytes_per_row);

        for row in 0..rows {
            let addr = start + (row * self.bytes_per_row);
            output.push_str(&format!("${:04X}:  ", addr));

            // Hex bytes
            for col in 0..self.bytes_per_row {
                let offset = row * self.bytes_per_row + col;
                if offset < data.len() {
                    output.push_str(&format!("{:02X} ", data[offset]));
                } else {
                    output.push_str("   ");
                }
            }

            output.push_str(" | ");

            // ASCII representation
            for col in 0..self.bytes_per_row {
                let offset = row * self.bytes_per_row + col;
                if offset < data.len() {
                    let byte = data[offset];
                    if (0x20..=0x7E).contains(&byte) {
                        output.push(byte as char);
                    } else {
                        output.push('.');
                    }
                } else {
                    output.push(' ');
                }
            }

            output.push('\n');
        }

        output
    }

    /// Search for a byte pattern in CPU memory
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    /// * `pattern` - Byte pattern to search for
    /// * `start` - Start address
    /// * `end` - End address
    ///
    /// # Returns
    ///
    /// A vector of addresses where the pattern was found
    pub fn search_cpu_memory(
        &self,
        bus: &mut Bus,
        pattern: &[u8],
        start: u16,
        end: u16,
    ) -> Vec<u16> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut addr = start;

        while addr <= end.saturating_sub(pattern.len() as u16 - 1) {
            let mut found = true;

            for (i, &byte) in pattern.iter().enumerate() {
                if bus.read(addr.wrapping_add(i as u16)) != byte {
                    found = false;
                    break;
                }
            }

            if found {
                matches.push(addr);
            }

            addr = addr.wrapping_add(1);

            // Prevent infinite loop on wrap
            if addr < start {
                break;
            }
        }

        matches
    }

    /// Read a single byte from CPU memory
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    /// * `addr` - Address to read from
    ///
    /// # Returns
    ///
    /// The byte value at the specified address
    pub fn read_byte(&self, bus: &mut Bus, addr: u16) -> u8 {
        bus.read(addr)
    }

    /// Read a 16-bit word from CPU memory (little-endian)
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    /// * `addr` - Address to read from
    ///
    /// # Returns
    ///
    /// The 16-bit word at the specified address
    pub fn read_word(&self, bus: &mut Bus, addr: u16) -> u16 {
        let lo = bus.read(addr) as u16;
        let hi = bus.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Dump zero page memory ($0000-$00FF)
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// A formatted hex dump of zero page
    pub fn dump_zero_page(&self, bus: &mut Bus) -> String {
        let mut output = String::from("Zero Page ($0000-$00FF):\n");
        output.push_str(&self.dump_cpu_memory(bus, 0x0000, 0x100));
        output
    }

    /// Dump stack memory ($0100-$01FF)
    ///
    /// # Arguments
    ///
    /// * `bus` - Reference to the bus
    ///
    /// # Returns
    ///
    /// A formatted hex dump of stack
    pub fn dump_stack(&self, bus: &mut Bus) -> String {
        let mut output = String::from("Stack ($0100-$01FF):\n");
        output.push_str(&self.dump_cpu_memory(bus, 0x0100, 0x100));
        output
    }

    /// Dump palette RAM
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A formatted hex dump of palette RAM
    pub fn dump_palette(&self, ppu: &Ppu) -> String {
        let mut output = String::from("Palette RAM ($3F00-$3F1F):\n");
        output.push_str(&self.dump_ppu_memory(ppu, MemoryRegion::PpuPalette, 0, 32));
        output
    }

    /// Dump OAM (Object Attribute Memory)
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A formatted hex dump of OAM
    pub fn dump_oam(&self, ppu: &Ppu) -> String {
        let mut output = String::from("OAM (Sprite Memory):\n");
        output.push_str(&self.dump_ppu_memory(ppu, MemoryRegion::PpuOam, 0, 256));
        output
    }
}

impl Default for MemoryViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_viewer_creation() {
        let viewer = MemoryViewer::new();
        assert_eq!(viewer.bytes_per_row, 16);
    }

    #[test]
    fn test_set_bytes_per_row() {
        let mut viewer = MemoryViewer::new();

        viewer.set_bytes_per_row(8);
        assert_eq!(viewer.bytes_per_row, 8);

        viewer.set_bytes_per_row(32);
        assert_eq!(viewer.bytes_per_row, 32);
    }

    #[test]
    fn test_read_byte() {
        let mut bus = Bus::new();
        let viewer = MemoryViewer::new();

        bus.write(0x1234, 0x42);
        assert_eq!(viewer.read_byte(&mut bus, 0x1234), 0x42);
    }

    #[test]
    fn test_read_word() {
        let mut bus = Bus::new();
        let viewer = MemoryViewer::new();

        bus.write(0x1234, 0x34); // Low byte
        bus.write(0x1235, 0x12); // High byte

        assert_eq!(viewer.read_word(&mut bus, 0x1234), 0x1234);
    }

    #[test]
    fn test_search_cpu_memory() {
        let mut bus = Bus::new();
        let viewer = MemoryViewer::new();

        // Write a pattern
        bus.write(0x1000, 0xDE);
        bus.write(0x1001, 0xAD);
        bus.write(0x1002, 0xBE);
        bus.write(0x1003, 0xEF);

        // Search for it (search a smaller range to avoid mirrored regions)
        let pattern = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let matches = viewer.search_cpu_memory(&mut bus, &pattern, 0x1000, 0x2000);

        // The pattern should be found at least at 0x1000
        assert!(!matches.is_empty());
        assert!(matches.contains(&0x1000));
    }

    #[test]
    fn test_dump_cpu_memory_format() {
        let mut bus = Bus::new();
        let viewer = MemoryViewer::new();

        // Write some test data
        for i in 0..32 {
            bus.write(0x8000 + i, i as u8);
        }

        let dump = viewer.dump_cpu_memory(&mut bus, 0x8000, 32);

        // Check that dump contains address markers
        assert!(dump.contains("$8000:"));
        assert!(dump.contains("$8010:"));

        // Check that dump contains hex values
        assert!(dump.contains("00"));
        assert!(dump.contains("01"));
    }

    #[test]
    #[should_panic(expected = "bytes_per_row must be greater than 0")]
    fn test_bytes_per_row_zero_panics() {
        let mut viewer = MemoryViewer::new();
        viewer.set_bytes_per_row(0);
    }
}
