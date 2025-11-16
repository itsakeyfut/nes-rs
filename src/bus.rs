// Bus module - Memory bus implementation
//
// This module implements the main memory bus that connects the CPU to all memory-mapped
// components in the NES system. It handles address routing, memory mirroring, and
// provides interfaces for dynamic component registration.
//
// # NES Memory Map (CPU Address Space)
//
// ```text
// $0000-$07FF: 2KB Internal RAM
// $0800-$1FFF: Mirrors of RAM (3 times)
// $2000-$2007: PPU Registers
// $2008-$3FFF: Mirrors of PPU Registers (repeating every 8 bytes)
// $4000-$4017: APU and I/O Registers
// $4018-$401F: APU and I/O test functionality (usually disabled)
// $4020-$FFFF: Cartridge space (PRG-ROM, PRG-RAM, and mapper registers)
// ```

use crate::apu::Apu;
use crate::input::ControllerIO;
use crate::ppu::Ppu;

/// Trait for memory-mapped components
///
/// This trait defines the interface for components that can be mapped into the
/// CPU's address space. Components implementing this trait can handle read and
/// write operations for their specific memory ranges.
///
/// # Examples
///
/// PPU, APU, and Cartridge all implement this trait to provide their memory-mapped
/// register interfaces.
pub trait MemoryMappedDevice {
    /// Read a byte from the device
    ///
    /// Some devices have side effects on read (e.g., PPU PPUSTATUS clears flags),
    /// so this method takes &mut self.
    ///
    /// # Arguments
    /// * `addr` - The address to read from (device-specific addressing)
    ///
    /// # Returns
    /// The byte value at the specified address
    fn read(&mut self, addr: u16) -> u8;

    /// Write a byte to the device
    ///
    /// # Arguments
    /// * `addr` - The address to write to (device-specific addressing)
    /// * `data` - The byte value to write
    fn write(&mut self, addr: u16, data: u8);
}

/// Main memory bus structure
///
/// The Bus connects the CPU to all memory-mapped components in the NES system.
/// It handles address decoding, memory mirroring, and routes read/write operations
/// to the appropriate components.
///
/// # Memory Layout
///
/// - Internal RAM: 2KB of general-purpose memory with 3 mirrors
/// - PPU Registers: 8 registers mirrored throughout $2000-$3FFF
/// - APU/I/O: Audio and input/output registers
/// - Cartridge Space: Game ROM and mapper-controlled memory
pub struct Bus {
    /// Internal RAM: 2KB
    ///
    /// The NES has 2KB of internal RAM located at $0000-$07FF.
    /// This RAM is mirrored 3 times at $0800-$1FFF.
    ram: [u8; 2048],

    /// PPU (Picture Processing Unit)
    ///
    /// The PPU has 8 registers mapped at $2000-$2007, mirrored throughout $2000-$3FFF.
    ppu: Ppu,

    /// APU (Audio Processing Unit)
    ///
    /// The APU has registers mapped at $4000-$4015 and $4017 (frame counter).
    apu: Apu,

    /// Controller I/O
    ///
    /// Controller ports mapped at $4016 (Controller 1) and $4017 (Controller 2).
    /// Note: $4017 is shared - writes go to APU, reads come from controller.
    controller_io: ControllerIO,

    /// Temporary ROM storage for testing
    ///
    /// This will be replaced with proper cartridge/mapper implementation.
    /// Covers $4020-$FFFF (approximately 48KB).
    rom: [u8; 0xC000],

    // ========================================
    // OAM DMA State
    // ========================================
    /// OAM DMA pending flag
    ///
    /// When true, indicates that an OAM DMA transfer has been requested
    /// and should be executed on the next CPU step.
    dma_pending: bool,

    /// OAM DMA page address (high byte)
    ///
    /// Stores the high byte of the source address for OAM DMA.
    /// DMA transfers 256 bytes from $XX00-$XXFF to OAM.
    dma_page: u8,

    /// OAM DMA remaining cycles
    ///
    /// Tracks the number of cycles remaining for the current DMA transfer.
    /// DMA takes 513 cycles (if starting on odd CPU cycle) or 514 cycles (even).
    dma_cycles: u16,
    // Future: Dynamic component registration
    // cartridge: Option<Box<dyn MemoryMappedDevice>>,
}

impl Bus {
    /// Create a new bus instance with zero-initialized memory
    ///
    /// # Returns
    /// A new Bus with all memory initialized to zero
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// ```
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            ppu: Ppu::new(),
            apu: Apu::new(),
            controller_io: ControllerIO::new(),
            rom: [0; 0xC000],
            dma_pending: false,
            dma_page: 0,
            dma_cycles: 0,
        }
    }

    /// Read a byte from the bus
    ///
    /// Routes the read operation to the appropriate memory region or device
    /// based on the address. Handles mirroring for RAM and PPU registers.
    ///
    /// # Arguments
    /// * `addr` - The 16-bit address to read from
    ///
    /// # Returns
    /// The byte value at the specified address
    ///
    /// # Memory Regions
    ///
    /// - $0000-$1FFF: Internal RAM (2KB) with mirroring
    /// - $2000-$3FFF: PPU registers (8 bytes) with mirroring
    /// - $4000-$4017: APU and I/O registers
    /// - $4018-$401F: APU/I/O test mode (usually returns open bus)
    /// - $4020-$FFFF: Cartridge space
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// let value = bus.read(0x0000); // Read from RAM
    /// ```
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // Internal RAM: $0000-$07FF (2KB)
            // Mirrored at: $0800-$0FFF, $1000-$17FF, $1800-$1FFF
            // Total range: $0000-$1FFF
            0x0000..=0x1FFF => {
                // Mirror using mask: only keep lowest 11 bits (0x07FF = 2KB)
                let ram_addr = (addr & 0x07FF) as usize;
                self.ram[ram_addr]
            }

            // PPU Registers: $2000-$2007 (8 bytes)
            // Mirrored throughout: $2008-$3FFF (repeating every 8 bytes)
            // Total range: $2000-$3FFF
            0x2000..=0x3FFF => {
                // Route to PPU - mirroring is handled inside the PPU
                self.ppu.read(addr)
            }

            // APU and I/O Registers: $4000-$4017
            0x4000..=0x4017 => {
                match addr {
                    // APU registers: $4000-$4015
                    0x4000..=0x4015 => self.apu.read(addr),

                    // $4016: Controller 1 (R/W)
                    0x4016 => self.controller_io.read(addr),

                    // $4017: Controller 2 (R) / APU Frame Counter (W)
                    // Reads return controller 2 data
                    0x4017 => self.controller_io.read(addr),

                    _ => 0,
                }
            }

            // APU/I/O Test Mode: $4018-$401F
            // Usually disabled on retail NES hardware
            0x4018..=0x401F => {
                // Return open bus value (typically last value on bus)
                0
            }

            // Cartridge Space: $4020-$FFFF
            // This includes PRG-ROM, PRG-RAM, and mapper registers
            0x4020..=0xFFFF => {
                // TODO: Route to cartridge/mapper when registered
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0
                }
            }
        }
    }

    /// Write a byte to the bus
    ///
    /// Routes the write operation to the appropriate memory region or device
    /// based on the address. Handles mirroring for RAM and PPU registers.
    ///
    /// # Arguments
    /// * `addr` - The 16-bit address to write to
    /// * `data` - The byte value to write
    ///
    /// # Memory Regions
    ///
    /// - $0000-$1FFF: Internal RAM (2KB) with mirroring
    /// - $2000-$3FFF: PPU registers (8 bytes) with mirroring
    /// - $4000-$4017: APU and I/O registers
    /// - $4018-$401F: APU/I/O test mode (usually ignored)
    /// - $4020-$FFFF: Cartridge space (writes may affect mapper state)
    ///
    /// # Note on ROM writes
    /// Writes to ROM addresses ($8000-$FFFF) don't modify ROM data but can
    /// trigger mapper functionality (bank switching, etc.)
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write(0x0000, 0x42); // Write to RAM
    /// ```
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Internal RAM: $0000-$07FF (2KB)
            // Mirrored at: $0800-$0FFF, $1000-$17FF, $1800-$1FFF
            // Total range: $0000-$1FFF
            0x0000..=0x1FFF => {
                // Mirror using mask: only keep lowest 11 bits (0x07FF = 2KB)
                let ram_addr = (addr & 0x07FF) as usize;
                self.ram[ram_addr] = data;
            }

            // PPU Registers: $2000-$2007 (8 bytes)
            // Mirrored throughout: $2008-$3FFF (repeating every 8 bytes)
            // Total range: $2000-$3FFF
            0x2000..=0x3FFF => {
                // Route to PPU - mirroring is handled inside the PPU
                self.ppu.write(addr, data);
            }

            // APU and I/O Registers: $4000-$4017
            0x4000..=0x4017 => {
                match addr {
                    // APU registers: $4000-$4013
                    0x4000..=0x4013 => self.apu.write(addr, data),

                    // $4014: OAM DMA - Trigger DMA transfer
                    // Writing to this register initiates a transfer of 256 bytes
                    // from CPU memory ($XX00-$XXFF) to PPU OAM memory.
                    // The written value is the high byte of the source address.
                    0x4014 => {
                        self.dma_page = data;
                        self.dma_pending = true;
                        // DMA cycles will be set when the transfer begins
                    }

                    // $4015: APU Status/Control
                    0x4015 => self.apu.write(addr, data),

                    // $4016: Controller 1 strobe (W) / Controller 1 data (R)
                    0x4016 => self.controller_io.write(addr, data),

                    // $4017: APU Frame Counter (W) / Controller 2 (R)
                    // Writes go to APU frame counter
                    0x4017 => self.apu.write(addr, data),

                    _ => {}
                }
            }

            // APU/I/O Test Mode: $4018-$401F
            // Usually disabled on retail NES hardware
            0x4018..=0x401F => {
                // Ignore writes to this region
            }

            // Cartridge Space: $4020-$FFFF
            // Writes here may trigger mapper functionality (e.g., bank switching)
            0x4020..=0xFFFF => {
                // TODO: Route to cartridge/mapper when registered
                // For now, allow writes to our temporary ROM array for testing
                let rom_addr = addr.wrapping_sub(0x4020) as usize;
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr] = data;
                }
            }
        }
    }

    /// Load ROM data into cartridge space
    ///
    /// This is a temporary helper method for testing. It will be replaced
    /// with proper cartridge/mapper loading in the future.
    ///
    /// # Arguments
    /// * `data` - Slice of bytes to load into ROM
    /// * `offset` - Starting address offset (relative to $4020)
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// let rom_data = vec![0x4C, 0x00, 0x80]; // JMP $8000
    /// bus.load_rom(&rom_data, 0x3FE0); // Load at $8000
    /// ```
    pub fn load_rom(&mut self, data: &[u8], offset: usize) {
        let end = (offset + data.len()).min(self.rom.len());
        self.rom[offset..end].copy_from_slice(&data[..(end - offset)]);
    }

    /// Read a 16-bit word from the bus (little-endian)
    ///
    /// Reads two consecutive bytes and combines them into a 16-bit value.
    /// The first byte is the low byte, the second is the high byte (little-endian).
    ///
    /// # Arguments
    /// * `addr` - The address of the low byte
    ///
    /// # Returns
    /// A 16-bit value constructed from two consecutive bytes
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write(0x0000, 0x34);
    /// bus.write(0x0001, 0x12);
    /// assert_eq!(bus.read_u16(0x0000), 0x1234);
    /// ```
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a 16-bit word to the bus (little-endian)
    ///
    /// Writes a 16-bit value as two consecutive bytes.
    /// The low byte is written first, then the high byte (little-endian).
    ///
    /// # Arguments
    /// * `addr` - The address to write the low byte to
    /// * `data` - The 16-bit value to write
    ///
    /// # Example
    /// ```
    /// use nes_rs::Bus;
    /// let mut bus = Bus::new();
    /// bus.write_u16(0x0000, 0x1234);
    /// assert_eq!(bus.read(0x0000), 0x34);
    /// assert_eq!(bus.read(0x0001), 0x12);
    /// ```
    pub fn write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }

    // ========================================
    // OAM DMA Methods
    // ========================================

    /// Check if OAM DMA transfer is pending or in progress
    ///
    /// # Returns
    ///
    /// True if DMA is pending or in progress, false otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::Bus;
    ///
    /// let mut bus = Bus::new();
    /// bus.write(0x4014, 0x02); // Trigger DMA from $0200
    /// assert!(bus.is_dma_active());
    /// ```
    pub fn is_dma_active(&self) -> bool {
        self.dma_pending || self.dma_cycles > 0
    }

    /// Execute OAM DMA transfer
    ///
    /// This method should be called by the CPU to execute the DMA transfer.
    /// It performs the transfer and returns the number of cycles consumed.
    ///
    /// DMA transfers 256 bytes from CPU address space ($XX00-$XXFF) to PPU OAM.
    /// The transfer takes 513 cycles if starting on an odd CPU cycle, or
    /// 514 cycles if starting on an even CPU cycle.
    ///
    /// # Arguments
    ///
    /// * `cpu_cycle` - The current CPU cycle count (used to determine alignment)
    ///
    /// # Returns
    ///
    /// The number of cycles consumed by the DMA transfer
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::Bus;
    ///
    /// let mut bus = Bus::new();
    /// // Setup source data
    /// for i in 0..256 {
    ///     bus.write(0x0200 + i, i as u8);
    /// }
    ///
    /// // Trigger DMA
    /// bus.write(0x4014, 0x02);
    ///
    /// // Execute DMA
    /// let cycles = bus.execute_dma(0);
    /// assert_eq!(cycles, 514); // Even cycle start = 514 cycles
    /// ```
    pub fn execute_dma(&mut self, cpu_cycle: u64) -> u16 {
        if !self.dma_pending {
            return 0;
        }

        // Calculate DMA cycles
        // DMA takes:
        // - 1 dummy cycle (wait cycle)
        // - +1 additional cycle if starting on an even CPU cycle
        // - 512 cycles for the actual transfer (256 reads + 256 writes)
        // Total: 513 cycles (odd start) or 514 cycles (even start)
        let alignment_cycle = if cpu_cycle.is_multiple_of(2) { 1 } else { 0 };
        let total_cycles = 513 + alignment_cycle;

        // Perform the DMA transfer
        // Read 256 bytes from $XX00-$XXFF and write to OAM
        let base_addr = (self.dma_page as u16) << 8;

        for offset in 0..256u16 {
            let source_addr = base_addr.wrapping_add(offset);
            let data = self.read(source_addr);
            self.ppu.write(0x2004, data);
        }

        // Clear DMA pending flag
        self.dma_pending = false;
        self.dma_cycles = 0;

        total_cycles
    }

    /// Get the DMA page (for testing)
    pub fn get_dma_page(&self) -> u8 {
        self.dma_page
    }

    // ========================================
    // Controller Input
    // ========================================

    /// Update controller 1 state
    ///
    /// Sets the button states for controller 1. This should be called by the
    /// application layer when input events occur (e.g., keyboard or gamepad input).
    ///
    /// # Arguments
    ///
    /// * `controller` - The new controller state with button states
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::{Bus, input::Controller};
    ///
    /// let mut bus = Bus::new();
    /// let mut controller = Controller::new();
    /// controller.button_a = true;
    /// controller.start = true;
    /// bus.set_controller1(controller);
    /// ```
    pub fn set_controller1(&mut self, controller: crate::input::Controller) {
        self.controller_io.set_controller1(controller);
    }

    /// Update controller 2 state
    ///
    /// Sets the button states for controller 2. This should be called by the
    /// application layer when input events occur (e.g., keyboard or gamepad input).
    ///
    /// # Arguments
    ///
    /// * `controller` - The new controller state with button states
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::{Bus, input::Controller};
    ///
    /// let mut bus = Bus::new();
    /// let mut controller = Controller::new();
    /// controller.button_b = true;
    /// controller.select = true;
    /// bus.set_controller2(controller);
    /// ```
    pub fn set_controller2(&mut self, controller: crate::input::Controller) {
        self.controller_io.set_controller2(controller);
    }

    // ========================================
    // PPU Synchronization
    // ========================================

    /// Synchronize PPU with CPU cycles
    ///
    /// Executes the PPU for the number of cycles corresponding to CPU cycles.
    /// The PPU runs at 3 times the speed of the CPU (3 PPU cycles per CPU cycle).
    ///
    /// # Arguments
    ///
    /// * `cpu_cycles` - Number of CPU cycles to synchronize
    ///
    /// # Returns
    ///
    /// `true` if a frame was completed during execution, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::Bus;
    ///
    /// let mut bus = Bus::new();
    ///
    /// // After CPU executes an instruction that took 2 cycles
    /// let frame_complete = bus.tick_ppu(2);
    /// if frame_complete {
    ///     // A frame is ready for display
    /// }
    /// ```
    pub fn tick_ppu(&mut self, cpu_cycles: u8) -> bool {
        let mut frame_complete = false;

        // Execute 3 PPU cycles for each CPU cycle
        let ppu_cycles = cpu_cycles as u16 * 3;

        for _ in 0..ppu_cycles {
            if self.ppu.step() {
                frame_complete = true;
            }
        }

        frame_complete
    }

    /// Check if PPU has a pending NMI
    ///
    /// The CPU should check this after each instruction to handle NMI interrupts.
    ///
    /// # Returns
    ///
    /// `true` if the PPU has generated an NMI that needs to be handled
    ///
    /// # Example
    ///
    /// ```
    /// use nes_rs::Bus;
    ///
    /// let mut bus = Bus::new();
    ///
    /// // After ticking the PPU
    /// if bus.ppu_nmi_pending() {
    ///     // CPU should handle NMI interrupt
    ///     bus.clear_ppu_nmi();
    /// }
    /// ```
    pub fn ppu_nmi_pending(&self) -> bool {
        self.ppu.nmi_pending()
    }

    /// Clear the PPU NMI pending flag
    ///
    /// The CPU should call this after handling an NMI interrupt.
    pub fn clear_ppu_nmi(&mut self) {
        self.ppu.clear_nmi();
    }

    /// Get a reference to the PPU for direct access
    ///
    /// This is useful for accessing PPU state like frame buffer, scanline, etc.
    ///
    /// # Returns
    ///
    /// A reference to the PPU
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    /// Get a mutable reference to the PPU for direct access
    ///
    /// This is useful for operations that need to modify PPU state directly.
    ///
    /// # Returns
    ///
    /// A mutable reference to the PPU
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    /// Get a reference to the RAM contents (for save states)
    ///
    /// # Returns
    ///
    /// A reference to the internal RAM (2KB)
    pub fn ram_contents(&self) -> &[u8; 2048] {
        &self.ram
    }

    /// Restore RAM contents (for save states)
    ///
    /// # Arguments
    ///
    /// * `data` - The RAM contents to restore (2KB)
    ///
    /// # Panics
    ///
    /// Panics if the data slice is not exactly 2048 bytes
    pub fn restore_ram_contents(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 2048, "RAM data must be exactly 2048 bytes");
        self.ram.copy_from_slice(data);
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

    // ========================================
    // Bus Initialization Tests
    // ========================================

    #[test]
    fn test_bus_initialization() {
        let mut bus = Bus::new();
        // Verify RAM is zero-initialized
        assert_eq!(bus.read(0x0000), 0, "RAM start should be zero");
        assert_eq!(bus.read(0x07FF), 0, "RAM end should be zero");
        assert_eq!(bus.read(0x0400), 0, "RAM middle should be zero");
    }

    #[test]
    fn test_bus_default() {
        let mut bus1 = Bus::new();
        let mut bus2 = Bus::default();
        // Both should have same initial state
        assert_eq!(bus1.read(0x0000), bus2.read(0x0000));
    }

    // ========================================
    // RAM Tests ($ 0000-$1FFF)
    // ========================================

    #[test]
    fn test_ram_read_write() {
        let mut bus = Bus::new();
        bus.write(0x0000, 0x42);
        assert_eq!(bus.read(0x0000), 0x42);
    }

    #[test]
    fn test_ram_multiple_writes() {
        let mut bus = Bus::new();
        // Write to different RAM addresses
        bus.write(0x0000, 0x11);
        bus.write(0x0100, 0x22);
        bus.write(0x0200, 0x33);
        bus.write(0x07FF, 0x44);

        assert_eq!(bus.read(0x0000), 0x11);
        assert_eq!(bus.read(0x0100), 0x22);
        assert_eq!(bus.read(0x0200), 0x33);
        assert_eq!(bus.read(0x07FF), 0x44);
    }

    #[test]
    fn test_ram_mirroring_first_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0000
        bus.write(0x0000, 0x42);
        // Read from first mirror at $0800
        assert_eq!(
            bus.read(0x0800),
            0x42,
            "First mirror should reflect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_second_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0100
        bus.write(0x0100, 0x55);
        // Read from second mirror at $1100
        assert_eq!(
            bus.read(0x1100),
            0x55,
            "Second mirror should reflect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_third_mirror() {
        let mut bus = Bus::new();
        // Write to base RAM at $0200
        bus.write(0x0200, 0x88);
        // Read from third mirror at $1A00
        assert_eq!(
            bus.read(0x1A00),
            0x88,
            "Third mirror should reflect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_bidirectional() {
        let mut bus = Bus::new();
        // Write to mirror
        bus.write(0x0800, 0x99);
        // Read from base RAM
        assert_eq!(
            bus.read(0x0000),
            0x99,
            "Mirror write should affect base RAM"
        );

        // Write to another mirror
        bus.write(0x1500, 0xAA);
        // Read from base RAM
        assert_eq!(
            bus.read(0x0500),
            0xAA,
            "Mirror write should affect base RAM"
        );
    }

    #[test]
    fn test_ram_mirroring_all_regions() {
        let mut bus = Bus::new();
        // Test that all four regions are properly mirrored
        let test_addr = 0x0123; // Arbitrary address within 2KB

        bus.write(test_addr, 0xAB);

        assert_eq!(bus.read(test_addr), 0xAB, "Base RAM");
        assert_eq!(bus.read(test_addr + 0x0800), 0xAB, "First mirror");
        assert_eq!(bus.read(test_addr + 0x1000), 0xAB, "Second mirror");
        assert_eq!(bus.read(test_addr + 0x1800), 0xAB, "Third mirror");
    }

    #[test]
    fn test_ram_boundary_addresses() {
        let mut bus = Bus::new();
        // Test boundary addresses of RAM
        bus.write(0x0000, 0x11); // Start of RAM
        bus.write(0x07FF, 0x22); // End of RAM
        bus.write(0x0800, 0x33); // Start of first mirror
        bus.write(0x1FFF, 0x44); // End of third mirror

        assert_eq!(bus.read(0x0000), 0x33, "Start should mirror to $0800");
        assert_eq!(bus.read(0x07FF), 0x44, "End should mirror to $1FFF");
    }

    // ========================================
    // PPU Register Tests ($2000-$3FFF)
    // ========================================

    #[test]
    fn test_ppu_register_range() {
        let mut bus = Bus::new();
        // PPU registers should return 0 (stub implementation)
        assert_eq!(bus.read(0x2000), 0, "PPUCTRL");
        assert_eq!(bus.read(0x2001), 0, "PPUMASK");
        assert_eq!(bus.read(0x2002), 0, "PPUSTATUS");
        assert_eq!(bus.read(0x2007), 0, "PPUDATA");
    }

    #[test]
    fn test_ppu_register_mirroring() {
        let mut bus = Bus::new();
        // PPU registers repeat every 8 bytes
        assert_eq!(bus.read(0x2000), bus.read(0x2008), "$2000 mirrors at $2008");
        assert_eq!(bus.read(0x2000), bus.read(0x2010), "$2000 mirrors at $2010");
        assert_eq!(bus.read(0x2007), bus.read(0x200F), "$2007 mirrors at $200F");
        assert_eq!(bus.read(0x2000), bus.read(0x3FF8), "$2000 mirrors at $3FF8");
    }

    #[test]
    fn test_ppu_register_write_does_not_crash() {
        let mut bus = Bus::new();
        // Writes to PPU registers should not crash (even if stubbed)
        bus.write(0x2000, 0x80);
        bus.write(0x2001, 0x1E);
        bus.write(0x2006, 0x20);
        bus.write(0x2007, 0x00);
    }

    #[test]
    fn test_ppu_mirror_write() {
        let mut bus = Bus::new();
        // Writes to mirrored PPU addresses should not crash
        bus.write(0x2008, 0x80); // Mirror of $2000
        bus.write(0x3000, 0x00); // Deep in mirror region
        bus.write(0x3FFF, 0xFF); // Last address in PPU region
    }

    // ========================================
    // APU and I/O Tests ($4000-$401F)
    // ========================================

    #[test]
    fn test_apu_registers() {
        let mut bus = Bus::new();
        // APU registers should return 0 (stub implementation)
        assert_eq!(bus.read(0x4000), 0, "SQ1_VOL");
        assert_eq!(bus.read(0x4015), 0, "SND_CHN");
    }

    #[test]
    fn test_apu_write_does_not_crash() {
        let mut bus = Bus::new();
        // Writes to APU registers should not crash
        bus.write(0x4000, 0x30);
        bus.write(0x4015, 0x0F);
    }

    #[test]
    fn test_io_test_region() {
        let mut bus = Bus::new();
        // Test region $4018-$401F should return 0
        assert_eq!(bus.read(0x4018), 0);
        assert_eq!(bus.read(0x401F), 0);
    }

    #[test]
    fn test_io_test_region_write_ignored() {
        let mut bus = Bus::new();
        // Writes to test region should be ignored
        bus.write(0x4018, 0xFF);
        bus.write(0x401F, 0xFF);
        // Should still read as 0
        assert_eq!(bus.read(0x4018), 0);
        assert_eq!(bus.read(0x401F), 0);
    }

    // ========================================
    // Cartridge Space Tests ($4020-$FFFF)
    // ========================================

    #[test]
    fn test_rom_read_write() {
        let mut bus = Bus::new();
        // Write to ROM space (for testing)
        bus.write(0x8000, 0x4C);
        assert_eq!(bus.read(0x8000), 0x4C);
    }

    #[test]
    fn test_load_rom() {
        let mut bus = Bus::new();
        let rom_data = vec![0x4C, 0x00, 0x80]; // JMP $8000

        // Load at offset 0x3FE0 (which maps to $8000)
        bus.load_rom(&rom_data, 0x3FE0);

        assert_eq!(bus.read(0x8000), 0x4C);
        assert_eq!(bus.read(0x8001), 0x00);
        assert_eq!(bus.read(0x8002), 0x80);
    }

    #[test]
    fn test_rom_boundary() {
        let mut bus = Bus::new();
        bus.write(0x4020, 0x11); // First address in cartridge space
        bus.write(0xFFFF, 0x22); // Last address in memory map

        assert_eq!(bus.read(0x4020), 0x11);
        assert_eq!(bus.read(0xFFFF), 0x22);
    }

    // ========================================
    // 16-bit Read/Write Tests
    // ========================================

    #[test]
    fn test_read_u16() {
        let mut bus = Bus::new();
        bus.write(0x0000, 0x34); // Low byte
        bus.write(0x0001, 0x12); // High byte

        let value = bus.read_u16(0x0000);
        assert_eq!(value, 0x1234, "Should read little-endian 16-bit value");
    }

    #[test]
    fn test_write_u16() {
        let mut bus = Bus::new();
        bus.write_u16(0x0000, 0x1234);

        assert_eq!(bus.read(0x0000), 0x34, "Low byte should be first");
        assert_eq!(bus.read(0x0001), 0x12, "High byte should be second");
    }

    #[test]
    fn test_u16_roundtrip() {
        let mut bus = Bus::new();
        let test_value = 0xABCD;

        bus.write_u16(0x0100, test_value);
        let read_value = bus.read_u16(0x0100);

        assert_eq!(
            read_value, test_value,
            "16-bit roundtrip should preserve value"
        );
    }

    #[test]
    fn test_u16_across_pages() {
        let mut bus = Bus::new();
        // Write 16-bit value that crosses page boundary
        bus.write_u16(0x00FF, 0x5678);

        assert_eq!(bus.read(0x00FF), 0x78);
        assert_eq!(bus.read(0x0100), 0x56);
        assert_eq!(bus.read_u16(0x00FF), 0x5678);
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_stack_operations() {
        let mut bus = Bus::new();
        // Stack is at $0100-$01FF

        // Push values (typical stack usage)
        bus.write(0x01FD, 0x11);
        bus.write(0x01FC, 0x22);
        bus.write(0x01FB, 0x33);

        assert_eq!(bus.read(0x01FD), 0x11);
        assert_eq!(bus.read(0x01FC), 0x22);
        assert_eq!(bus.read(0x01FB), 0x33);
    }

    #[test]
    fn test_zero_page_operations() {
        let mut bus = Bus::new();
        // Zero page is $0000-$00FF
        bus.write(0x0000, 0xAA);
        bus.write(0x00FF, 0xBB);

        assert_eq!(bus.read(0x0000), 0xAA);
        assert_eq!(bus.read(0x00FF), 0xBB);
    }

    #[test]
    fn test_interrupt_vectors() {
        let mut bus = Bus::new();
        // Set interrupt vectors
        bus.write_u16(0xFFFA, 0x9000); // NMI vector
        bus.write_u16(0xFFFC, 0x8000); // RESET vector
        bus.write_u16(0xFFFE, 0xA000); // IRQ/BRK vector

        assert_eq!(bus.read_u16(0xFFFA), 0x9000);
        assert_eq!(bus.read_u16(0xFFFC), 0x8000);
        assert_eq!(bus.read_u16(0xFFFE), 0xA000);
    }

    #[test]
    fn test_memory_independence() {
        let mut bus = Bus::new();
        // Write to different memory regions
        bus.write(0x0000, 0x11); // RAM
        bus.write(0x2000, 0x22); // PPU
        bus.write(0x4000, 0x33); // APU
        bus.write(0x8000, 0x44); // ROM

        // Verify they don't interfere with each other
        assert_eq!(bus.read(0x0000), 0x11);
        assert_eq!(bus.read(0x8000), 0x44);
    }

    #[test]
    fn test_comprehensive_memory_map() {
        let mut bus = Bus::new();

        // Test each major memory region
        bus.write(0x0010, 0x01); // RAM
        bus.write(0x0810, 0x02); // RAM mirror
        bus.write(0x2003, 0x03); // PPU
        bus.write(0x300B, 0x04); // PPU mirror
        bus.write(0x4005, 0x05); // APU
        bus.write(0x8050, 0x06); // ROM

        assert_eq!(bus.read(0x0010), 0x02, "RAM mirror should overwrite");
        assert_eq!(bus.read(0x0810), 0x02, "RAM mirror bidirectional");
        assert_eq!(bus.read(0x8050), 0x06, "ROM independent");
    }

    // ========================================
    // OAM DMA Tests ($4014)
    // ========================================

    #[test]
    fn test_oam_dma_trigger() {
        let mut bus = Bus::new();

        // Initially no DMA should be active
        assert!(!bus.is_dma_active());

        // Write to $4014 to trigger DMA
        bus.write(0x4014, 0x02);

        // DMA should now be pending
        assert!(bus.is_dma_active());
        assert_eq!(bus.get_dma_page(), 0x02);
    }

    #[test]
    fn test_oam_dma_transfer_from_page_02() {
        let mut bus = Bus::new();

        // Setup source data in RAM at $0200-$02FF
        for i in 0..256 {
            bus.write(0x0200 + i, i as u8);
        }

        // Trigger DMA from page $02
        bus.write(0x4014, 0x02);
        assert!(bus.is_dma_active());

        // Execute DMA
        let cycles = bus.execute_dma(0); // Start on even cycle

        // Check cycles (even cycle start = 514 cycles)
        assert_eq!(cycles, 514, "DMA should take 514 cycles on even start");

        // Verify DMA is no longer active
        assert!(!bus.is_dma_active());

        // Verify all 256 bytes were transferred to OAM
        for i in 0..256 {
            assert_eq!(
                bus.ppu.read_oam(i as u8),
                i as u8,
                "OAM byte {} should match source data",
                i
            );
        }
    }

    #[test]
    fn test_oam_dma_transfer_from_page_03() {
        let mut bus = Bus::new();

        // Setup source data in RAM at $0300-$03FF
        for i in 0..256 {
            bus.write(0x0300 + i, (255 - i) as u8);
        }

        // Trigger DMA from page $03
        bus.write(0x4014, 0x03);

        // Execute DMA
        let cycles = bus.execute_dma(1); // Start on odd cycle

        // Check cycles (odd cycle start = 513 cycles)
        assert_eq!(cycles, 513, "DMA should take 513 cycles on odd start");

        // Verify all 256 bytes were transferred correctly
        for i in 0..256 {
            assert_eq!(
                bus.ppu.read_oam(i as u8),
                (255 - i) as u8,
                "OAM byte {} should match source data",
                i
            );
        }
    }

    #[test]
    fn test_oam_dma_cycle_alignment_even() {
        let mut bus = Bus::new();

        // Setup dummy data
        for i in 0..256 {
            bus.write(0x0200 + i, 0xAA);
        }

        bus.write(0x4014, 0x02);

        // Execute on even cycle (0, 2, 4, etc.)
        let cycles = bus.execute_dma(0);
        assert_eq!(cycles, 514, "Even cycle start should take 514 cycles");

        let cycles = bus.execute_dma(2);
        assert_eq!(cycles, 0, "DMA already completed, should return 0");
    }

    #[test]
    fn test_oam_dma_cycle_alignment_odd() {
        let mut bus = Bus::new();

        // Setup dummy data
        for i in 0..256 {
            bus.write(0x0200 + i, 0xBB);
        }

        bus.write(0x4014, 0x02);

        // Execute on odd cycle (1, 3, 5, etc.)
        let cycles = bus.execute_dma(1);
        assert_eq!(cycles, 513, "Odd cycle start should take 513 cycles");
    }

    #[test]
    fn test_oam_dma_from_different_pages() {
        let mut bus = Bus::new();

        // Test DMA from various pages
        for page in [0x00, 0x01, 0x02, 0x03, 0x07, 0x80, 0xFF] {
            // Setup source data
            let base = (page as u16) << 8;
            for i in 0..256 {
                bus.write(base + i, page);
            }

            // Trigger DMA
            bus.write(0x4014, page);
            bus.execute_dma(0);

            // Verify transfer
            for i in 0..256 {
                assert_eq!(
                    bus.ppu.read_oam(i as u8),
                    page,
                    "OAM transfer from page ${:02X} failed at offset {}",
                    page,
                    i
                );
            }
        }
    }

    #[test]
    fn test_oam_dma_overwrites_existing_data() {
        let mut bus = Bus::new();

        // Fill OAM with initial data via OAMDATA register
        bus.write(0x2003, 0x00); // Set OAM address to 0
        for _i in 0..256 {
            bus.write(0x2004, 0xFF);
        }

        // Verify OAM is filled with 0xFF
        assert_eq!(bus.ppu.read_oam(0), 0xFF);
        assert_eq!(bus.ppu.read_oam(128), 0xFF);

        // Setup new data in RAM
        for i in 0..256 {
            bus.write(0x0200 + i, i as u8);
        }

        // Execute DMA
        bus.write(0x4014, 0x02);
        bus.execute_dma(0);

        // Verify OAM was overwritten
        for i in 0..256 {
            assert_eq!(
                bus.ppu.read_oam(i as u8),
                i as u8,
                "OAM should be overwritten by DMA"
            );
        }
    }

    #[test]
    fn test_oam_dma_multiple_transfers() {
        let mut bus = Bus::new();

        // First transfer
        for i in 0..256 {
            bus.write(0x0200 + i, 0x11);
        }
        bus.write(0x4014, 0x02);
        bus.execute_dma(0);

        assert_eq!(bus.ppu.read_oam(0), 0x11);
        assert_eq!(bus.ppu.read_oam(255), 0x11);

        // Second transfer
        for i in 0..256 {
            bus.write(0x0300 + i, 0x22);
        }
        bus.write(0x4014, 0x03);
        bus.execute_dma(0);

        assert_eq!(bus.ppu.read_oam(0), 0x22);
        assert_eq!(bus.ppu.read_oam(255), 0x22);

        // Third transfer
        for i in 0..256 {
            bus.write(0x0400 + i, 0x33);
        }
        bus.write(0x4014, 0x04);
        bus.execute_dma(1);

        assert_eq!(bus.ppu.read_oam(0), 0x33);
        assert_eq!(bus.ppu.read_oam(255), 0x33);
    }

    #[test]
    fn test_oam_dma_does_not_affect_other_memory() {
        let mut bus = Bus::new();

        // Setup data in various memory regions that won't overlap with DMA source
        bus.write(0x0100, 0xAA); // RAM region
        bus.write(0x0500, 0xBB); // Different RAM region
        bus.write(0x07FF, 0xCC); // RAM end

        // Setup DMA source (use 0x0200-0x02FF)
        for i in 0..256 {
            bus.write(0x0200 + i, i as u8);
        }

        // Execute DMA
        bus.write(0x4014, 0x02);
        bus.execute_dma(0);

        // Verify other memory regions are unchanged
        // (0x0200-0x02FF were used as source, so they should be unchanged)
        assert_eq!(bus.read(0x0100), 0xAA, "RAM $0100 unchanged");
        assert_eq!(bus.read(0x0500), 0xBB, "RAM $0500 unchanged");
        assert_eq!(bus.read(0x07FF), 0xCC, "RAM end unchanged");
    }

    #[test]
    fn test_oam_dma_from_ram_mirror() {
        let mut bus = Bus::new();

        // Write data to RAM mirror region ($0800-$0FFF mirrors $0000-$07FF)
        for i in 0..256 {
            bus.write(0x0800 + i, (i ^ 0xFF) as u8);
        }

        // Trigger DMA from page $08 (which mirrors page $00)
        bus.write(0x4014, 0x08);
        bus.execute_dma(0);

        // Verify transfer (should read from mirrored RAM)
        for i in 0..256 {
            assert_eq!(
                bus.ppu.read_oam(i as u8),
                (i ^ 0xFF) as u8,
                "DMA from RAM mirror failed at offset {}",
                i
            );
        }
    }

    #[test]
    fn test_oam_dma_edge_case_page_ff() {
        let mut bus = Bus::new();

        // Setup data at $FF00-$FFFF (top of address space)
        for i in 0..256 {
            bus.write(0xFF00 + i, 0x42);
        }

        // Trigger DMA from page $FF
        bus.write(0x4014, 0xFF);
        bus.execute_dma(0);

        // Verify transfer
        for i in 0..256 {
            assert_eq!(bus.ppu.read_oam(i as u8), 0x42, "DMA from page $FF failed");
        }
    }

    #[test]
    fn test_oam_dma_is_not_active_initially() {
        let bus = Bus::new();
        assert!(!bus.is_dma_active());
    }

    #[test]
    fn test_oam_dma_execute_without_trigger() {
        let mut bus = Bus::new();

        // Try to execute DMA without triggering it
        let cycles = bus.execute_dma(0);

        // Should return 0 cycles (no DMA to execute)
        assert_eq!(cycles, 0);
    }

    #[test]
    fn test_oam_dma_sprite_data_structure() {
        let mut bus = Bus::new();

        // Setup sprite data in memory (4 bytes per sprite, 64 sprites)
        // Sprite 0: Y=10, Tile=0x20, Attr=0x00, X=50
        bus.write(0x0200, 10); // Y
        bus.write(0x0201, 0x20); // Tile
        bus.write(0x0202, 0x00); // Attributes
        bus.write(0x0203, 50); // X

        // Sprite 1: Y=20, Tile=0x21, Attr=0x01, X=60
        bus.write(0x0204, 20);
        bus.write(0x0205, 0x21);
        bus.write(0x0206, 0x01);
        bus.write(0x0207, 60);

        // Fill rest with zeros
        for i in 8..256 {
            bus.write(0x0200 + i, 0x00);
        }

        // Execute DMA
        bus.write(0x4014, 0x02);
        bus.execute_dma(0);

        // Verify sprite 0 data
        assert_eq!(bus.ppu.read_oam(0), 10, "Sprite 0 Y position");
        assert_eq!(bus.ppu.read_oam(1), 0x20, "Sprite 0 tile");
        assert_eq!(bus.ppu.read_oam(2), 0x00, "Sprite 0 attributes");
        assert_eq!(bus.ppu.read_oam(3), 50, "Sprite 0 X position");

        // Verify sprite 1 data
        assert_eq!(bus.ppu.read_oam(4), 20, "Sprite 1 Y position");
        assert_eq!(bus.ppu.read_oam(5), 0x21, "Sprite 1 tile");
        assert_eq!(bus.ppu.read_oam(6), 0x01, "Sprite 1 attributes");
        assert_eq!(bus.ppu.read_oam(7), 60, "Sprite 1 X position");
    }

    #[test]
    fn test_oam_dma_typical_game_usage() {
        let mut bus = Bus::new();

        // Typical game usage pattern:
        // 1. Game prepares sprite data in RAM page (e.g., $0200-$02FF)
        for sprite_num in 0u8..64 {
            let base = 0x0200 + (sprite_num as u16 * 4);
            bus.write(base, sprite_num * 4); // Y position
            bus.write(base + 1, sprite_num); // Tile number
            bus.write(base + 2, 0x00); // Attributes
            bus.write(base + 3, sprite_num * 4); // X position
        }

        // 2. During VBlank, game triggers OAM DMA
        bus.write(0x4014, 0x02);
        assert!(bus.is_dma_active());

        // 3. CPU executes DMA transfer
        let cycles = bus.execute_dma(1); // Assume we're on odd cycle

        // 4. Verify DMA completed
        assert!(!bus.is_dma_active());
        assert_eq!(cycles, 513);

        // 5. Verify all sprite data transferred correctly
        for sprite_num in 0u8..64 {
            let oam_offset = sprite_num * 4;
            assert_eq!(
                bus.ppu.read_oam(oam_offset),
                sprite_num * 4,
                "Sprite {} Y position",
                sprite_num
            );
            assert_eq!(
                bus.ppu.read_oam(oam_offset + 1),
                sprite_num,
                "Sprite {} tile",
                sprite_num
            );
        }
    }

    // ========================================
    // Controller Integration Tests
    // ========================================

    #[test]
    fn test_bus_set_controller1() {
        use crate::input::Controller;
        let mut bus = Bus::new();

        // Create controller with some buttons pressed
        let mut controller = Controller::new();
        controller.button_a = true;
        controller.start = true;

        // Set controller state via bus
        bus.set_controller1(controller);

        // Read controller state via memory-mapped I/O
        bus.write(0x4016, 0x01); // Strobe on
        bus.write(0x4016, 0x00); // Strobe off

        assert_eq!(bus.read(0x4016), 0x01); // A pressed
        assert_eq!(bus.read(0x4016), 0x00); // B not pressed
        assert_eq!(bus.read(0x4016), 0x00); // Select not pressed
        assert_eq!(bus.read(0x4016), 0x01); // Start pressed
    }

    #[test]
    fn test_bus_set_controller2() {
        use crate::input::Controller;
        let mut bus = Bus::new();

        // Create controller with different buttons pressed
        let mut controller = Controller::new();
        controller.button_b = true;
        controller.select = true;

        // Set controller state via bus
        bus.set_controller2(controller);

        // Read controller state via memory-mapped I/O
        bus.write(0x4016, 0x01); // Strobe on
        bus.write(0x4016, 0x00); // Strobe off

        assert_eq!(bus.read(0x4017), 0x00); // A not pressed
        assert_eq!(bus.read(0x4017), 0x01); // B pressed
        assert_eq!(bus.read(0x4017), 0x01); // Select pressed
        assert_eq!(bus.read(0x4017), 0x00); // Start not pressed
    }

    #[test]
    fn test_bus_both_controllers_independent() {
        use crate::input::Controller;
        let mut bus = Bus::new();

        // Set different states for each controller
        let mut controller1 = Controller::new();
        controller1.button_a = true;
        controller1.up = true;
        bus.set_controller1(controller1);

        let mut controller2 = Controller::new();
        controller2.button_b = true;
        controller2.down = true;
        bus.set_controller2(controller2);

        // Read both controllers
        bus.write(0x4016, 0x01);
        bus.write(0x4016, 0x00);

        // Controller 1
        assert_eq!(bus.read(0x4016), 0x01); // A pressed
        assert_eq!(bus.read(0x4016), 0x00); // B not pressed
        assert_eq!(bus.read(0x4016), 0x00); // Select not pressed
        assert_eq!(bus.read(0x4016), 0x00); // Start not pressed
        assert_eq!(bus.read(0x4016), 0x01); // Up pressed

        // Controller 2
        assert_eq!(bus.read(0x4017), 0x00); // A not pressed
        assert_eq!(bus.read(0x4017), 0x01); // B pressed
        assert_eq!(bus.read(0x4017), 0x00); // Select not pressed
        assert_eq!(bus.read(0x4017), 0x00); // Start not pressed
        assert_eq!(bus.read(0x4017), 0x00); // Up not pressed
        assert_eq!(bus.read(0x4017), 0x01); // Down pressed
    }

    #[test]
    fn test_bus_controller_update_during_gameplay() {
        use crate::input::Controller;
        let mut bus = Bus::new();

        // Simulate first frame - A button pressed
        let mut controller = Controller::new();
        controller.button_a = true;
        bus.set_controller1(controller);

        bus.write(0x4016, 0x01);
        bus.write(0x4016, 0x00);
        assert_eq!(bus.read(0x4016), 0x01); // A pressed

        // Simulate second frame - A released, B pressed
        let mut controller = Controller::new();
        controller.button_b = true;
        bus.set_controller1(controller);

        bus.write(0x4016, 0x01);
        bus.write(0x4016, 0x00);
        assert_eq!(bus.read(0x4016), 0x00); // A not pressed
        assert_eq!(bus.read(0x4016), 0x01); // B pressed
    }
}
