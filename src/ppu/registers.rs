// PPU register handling

use super::Ppu;

impl Ppu {
    /// Read from a PPU register
    ///
    /// # Arguments
    ///
    /// * `register` - The register number (0-7)
    ///
    /// # Returns
    ///
    /// The value read from the register
    ///
    /// # Register Behaviors
    ///
    /// - PPUSTATUS ($2002): Returns status, clears VBlank flag and address latch
    /// - OAMDATA ($2004): Returns OAM data at current OAM address
    /// - PPUDATA ($2007): Returns buffered PPU data (palette reads are immediate)
    /// - Write-only registers: Return 0
    pub(super) fn read_register(&mut self, register: u16) -> u8 {
        match register {
            0 => {
                // $2000: PPUCTRL - Write only, return 0
                0
            }
            1 => {
                // $2001: PPUMASK - Write only, return 0
                0
            }
            2 => {
                // $2002: PPUSTATUS - Read only
                // Reading PPUSTATUS has side effects:
                // 1. Clears bit 7 (VBlank flag) after reading
                // 2. Resets the address latch used by PPUSCROLL and PPUADDR
                // 3. Race condition: If read on the exact cycle VBlank is set,
                //    suppresses NMI generation
                let status = self.ppustatus;

                // Clear VBlank flag (bit 7)
                self.ppustatus &= 0x7F;

                // Reset address latch (w register)
                self.write_latch = false;

                // Race condition handling: If PPUSTATUS is read on the same cycle
                // that VBlank flag is set (scanline 241, cycle 1), suppress the NMI
                if self.vblank_just_set {
                    self.nmi_pending = false;
                }

                status
            }
            3 => {
                // $2003: OAMADDR - Write only, return 0
                0
            }
            4 => {
                // $2004: OAMDATA - Read/Write
                // Read from OAM at current OAM address
                self.oam[self.oam_addr as usize]
            }
            5 => {
                // $2005: PPUSCROLL - Write only, return 0
                0
            }
            6 => {
                // $2006: PPUADDR - Write only, return 0
                0
            }
            7 => {
                // $2007: PPUDATA - Read/Write
                // Reading from PPUDATA is buffered for addresses $0000-$3EFF
                // Palette reads ($3F00-$3FFF) are immediate but still update the buffer

                let addr = self.v & 0x3FFF;
                let value;

                if addr >= 0x3F00 {
                    // Palette reads are immediate (not buffered)
                    value = self.read_ppu_memory(addr);
                    // But still update the buffer with nametable data "underneath"
                    // This reads from the mirrored nametable address
                    self.read_buffer = self.read_ppu_memory(addr & 0x2FFF);
                } else {
                    // Normal reads are buffered
                    value = self.read_buffer;
                    self.read_buffer = self.read_ppu_memory(addr);
                }

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment) & 0x3FFF;

                value
            }
            _ => {
                // Should not reach here due to masking, but return 0 as fallback
                0
            }
        }
    }

    /// Write to a PPU register
    ///
    /// # Arguments
    ///
    /// * `register` - The register number (0-7)
    /// * `data` - The value to write
    ///
    /// # Register Behaviors
    ///
    /// - PPUCTRL ($2000): Stores control flags and updates nametable select in t
    /// - PPUMASK ($2001): Stores mask flags
    /// - OAMADDR ($2003): Sets OAM address
    /// - OAMDATA ($2004): Writes to OAM and increments address
    /// - PPUSCROLL ($2005): Sets scroll position (requires 2 writes, updates t and x)
    /// - PPUADDR ($2006): Sets PPU address (requires 2 writes, updates t then v)
    /// - PPUDATA ($2007): Writes to PPU memory and increments v
    /// - Read-only registers: Writes are ignored
    pub(super) fn write_register(&mut self, register: u16, data: u8) {
        match register {
            0 => {
                // $2000: PPUCTRL - Write only
                let old_nmi_enable = (self.ppuctrl & 0x80) != 0;
                let new_nmi_enable = (data & 0x80) != 0;

                self.ppuctrl = data;

                // Update nametable select bits in t register
                // t: ...GH.. ........ <- d: ......GH
                // (bits 10-11 of t from bits 0-1 of data)
                self.t = (self.t & 0xF3FF) | (((data as u16) & 0x03) << 10);

                // NMI enable/disable handling
                // If NMI is being enabled and VBlank flag is already set, trigger NMI
                // (unless this is the exact cycle VBlank is being set - that's handled separately)
                if !old_nmi_enable && new_nmi_enable {
                    // Enabling NMI
                    if (self.ppustatus & 0x80) != 0 && !self.vblank_just_set {
                        self.nmi_pending = true;
                    }
                } else if old_nmi_enable && !new_nmi_enable {
                    // Disabling NMI - suppress any pending NMI
                    self.nmi_pending = false;
                }

                self.prev_nmi_enable = new_nmi_enable;
            }
            1 => {
                // $2001: PPUMASK - Write only
                self.ppumask = data;
            }
            2 => {
                // $2002: PPUSTATUS - Read only, ignore writes
            }
            3 => {
                // $2003: OAMADDR - Write only
                self.oam_addr = data;
            }
            4 => {
                // $2004: OAMDATA - Read/Write
                // Write to OAM at current OAM address
                self.oam[self.oam_addr as usize] = data;

                // Increment OAM address
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                // $2005: PPUSCROLL - Write×2
                // This register uses complex bit manipulation to update the internal
                // scroll registers (t and fine_x)

                if !self.write_latch {
                    // First write: X scroll
                    // t: ....... ...ABCDE <- d: ABCDEFGH
                    // x:              FGH <- d: ABCDEFGH
                    self.t = (self.t & 0xFFE0) | ((data as u16) >> 3);
                    self.fine_x = data & 0x07;
                    self.write_latch = true;
                } else {
                    // Second write: Y scroll
                    // t: FGH..AB CDE..... <- d: ABCDEFGH
                    self.t = (self.t & 0x8FFF) | (((data as u16) & 0x07) << 12);
                    self.t = (self.t & 0xFC1F) | (((data as u16) & 0xF8) << 2);
                    self.write_latch = false;
                }
            }
            6 => {
                // $2006: PPUADDR - Write×2
                // First write: High byte of address
                // Second write: Low byte of address

                if !self.write_latch {
                    // First write: high byte
                    // t: .CDEFGH ........ <- d: ..CDEFGH
                    // t: X...... ........ <- 0
                    self.t = (self.t & 0x80FF) | (((data as u16) & 0x3F) << 8);
                    self.write_latch = true;
                } else {
                    // Second write: low byte
                    // t: ....... ABCDEFGH <- d: ABCDEFGH
                    // v: <...all bits...> <- t: <...all bits...>
                    self.t = (self.t & 0xFF00) | (data as u16);
                    self.v = self.t;
                    self.write_latch = false;
                }
            }
            7 => {
                // $2007: PPUDATA - Read/Write
                // Write to PPU memory at current address (v)
                self.write_ppu_memory(self.v, data);

                // Increment address based on PPUCTRL bit 2
                let increment = if self.ppuctrl & 0x04 != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment) & 0x3FFF;
            }
            _ => {
                // Should not reach here due to masking, but ignore as fallback
            }
        }
    }
}
