// PPU Debugger - Debug information for the PPU
//
// Provides:
// - Nametable viewer
// - Pattern table viewer
// - Palette viewer
// - OAM viewer
// - PPU state capture

use crate::ppu::Ppu;

/// PPU state snapshot
///
/// Contains a complete snapshot of the PPU state at a specific point in time.
#[derive(Debug, Clone)]
pub struct PpuState {
    /// Current scanline (0-261)
    pub scanline: u16,

    /// Current cycle (0-340)
    pub cycle: u16,

    /// Frame counter
    pub frame: u64,

    /// PPUCTRL register ($2000)
    pub ppuctrl: u8,

    /// PPUMASK register ($2001)
    pub ppumask: u8,

    /// PPUSTATUS register ($2002)
    pub ppustatus: u8,

    /// OAMADDR register ($2003)
    pub oam_addr: u8,

    /// Current VRAM address (v)
    pub v: u16,

    /// Temporary VRAM address (t)
    pub t: u16,

    /// Fine X scroll
    pub fine_x: u8,

    /// Write latch (w)
    pub write_latch: bool,

    /// NMI pending flag
    pub nmi_pending: bool,
}

impl PpuState {
    /// Format PPUCTRL flags
    ///
    /// # Returns
    ///
    /// A string describing the PPUCTRL flags
    pub fn format_ppuctrl(&self) -> String {
        let mut flags = Vec::new();

        if self.ppuctrl & 0x80 != 0 {
            flags.push("NMI");
        }
        if self.ppuctrl & 0x20 != 0 {
            flags.push("SPR8x16");
        } else {
            flags.push("SPR8x8");
        }
        if self.ppuctrl & 0x10 != 0 {
            flags.push("BG@$1000");
        } else {
            flags.push("BG@$0000");
        }
        if self.ppuctrl & 0x08 != 0 {
            flags.push("SPR@$1000");
        } else {
            flags.push("SPR@$0000");
        }
        if self.ppuctrl & 0x04 != 0 {
            flags.push("+32");
        } else {
            flags.push("+1");
        }

        let nametable = self.ppuctrl & 0x03;
        flags.push(match nametable {
            0 => "NT$2000",
            1 => "NT$2400",
            2 => "NT$2800",
            3 => "NT$2C00",
            _ => unreachable!(),
        });

        flags.join(" ")
    }

    /// Format PPUMASK flags
    ///
    /// # Returns
    ///
    /// A string describing the PPUMASK flags
    pub fn format_ppumask(&self) -> String {
        let mut flags = Vec::new();

        if self.ppumask & 0x80 != 0 {
            flags.push("EmpB");
        }
        if self.ppumask & 0x40 != 0 {
            flags.push("EmpG");
        }
        if self.ppumask & 0x20 != 0 {
            flags.push("EmpR");
        }
        if self.ppumask & 0x10 != 0 {
            flags.push("ShowSPR");
        }
        if self.ppumask & 0x08 != 0 {
            flags.push("ShowBG");
        }
        if self.ppumask & 0x04 != 0 {
            flags.push("SPRLeft");
        }
        if self.ppumask & 0x02 != 0 {
            flags.push("BGLeft");
        }
        if self.ppumask & 0x01 != 0 {
            flags.push("Gray");
        }

        if flags.is_empty() {
            "None".to_string()
        } else {
            flags.join(" ")
        }
    }

    /// Format PPUSTATUS flags
    ///
    /// # Returns
    ///
    /// A string describing the PPUSTATUS flags
    pub fn format_ppustatus(&self) -> String {
        let mut flags = Vec::new();

        if self.ppustatus & 0x80 != 0 {
            flags.push("VBlank");
        }
        if self.ppustatus & 0x40 != 0 {
            flags.push("Spr0Hit");
        }
        if self.ppustatus & 0x20 != 0 {
            flags.push("SprOvf");
        }

        if flags.is_empty() {
            "None".to_string()
        } else {
            flags.join(" ")
        }
    }

    /// Format the PPU state as a string
    ///
    /// # Returns
    ///
    /// A formatted string representation of the PPU state
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Scanline: {} Cycle: {} Frame: {}\n",
            self.scanline, self.cycle, self.frame
        ));
        output.push_str(&format!(
            "PPUCTRL:   ${:02X} [{}]\n",
            self.ppuctrl,
            self.format_ppuctrl()
        ));
        output.push_str(&format!(
            "PPUMASK:   ${:02X} [{}]\n",
            self.ppumask,
            self.format_ppumask()
        ));
        output.push_str(&format!(
            "PPUSTATUS: ${:02X} [{}]\n",
            self.ppustatus,
            self.format_ppustatus()
        ));
        output.push_str(&format!("OAMADDR:   ${:02X}\n", self.oam_addr));
        output.push_str(&format!(
            "v: ${:04X} t: ${:04X} x: {} w: {}\n",
            self.v,
            self.t,
            self.fine_x,
            if self.write_latch { 1 } else { 0 }
        ));
        output.push_str(&format!(
            "NMI: {}\n",
            if self.nmi_pending { "Pending" } else { "None" }
        ));

        output
    }
}

impl std::fmt::Display for PpuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PPU[{:3},{:3}] CTRL:{:02X} MASK:{:02X} STATUS:{:02X} v:{:04X}",
            self.scanline, self.cycle, self.ppuctrl, self.ppumask, self.ppustatus, self.v
        )
    }
}

/// Sprite information from OAM
#[derive(Debug, Clone)]
pub struct SpriteInfo {
    /// Sprite index (0-63)
    pub index: u8,

    /// Y position
    pub y: u8,

    /// Tile index
    pub tile: u8,

    /// Attributes
    pub attributes: u8,

    /// X position
    pub x: u8,
}

impl SpriteInfo {
    /// Get the palette number (0-3)
    pub fn palette(&self) -> u8 {
        self.attributes & 0x03
    }

    /// Check if sprite is behind background
    pub fn behind_background(&self) -> bool {
        (self.attributes & 0x20) != 0
    }

    /// Check if sprite is flipped horizontally
    pub fn flip_horizontal(&self) -> bool {
        (self.attributes & 0x40) != 0
    }

    /// Check if sprite is flipped vertically
    pub fn flip_vertical(&self) -> bool {
        (self.attributes & 0x80) != 0
    }

    /// Format sprite info as a string
    pub fn format(&self) -> String {
        format!(
            "Sprite {:2}: Y={:3} X={:3} Tile=${:02X} Pal={} {}{}{}",
            self.index,
            self.y,
            self.x,
            self.tile,
            self.palette(),
            if self.behind_background() { "BG " } else { "" },
            if self.flip_horizontal() { "FH " } else { "" },
            if self.flip_vertical() { "FV" } else { "" }
        )
    }
}

/// PPU Debugger
///
/// Provides debugging functionality for the PPU including:
/// - State capture
/// - Nametable viewing
/// - Pattern table viewing
/// - Palette viewing
/// - OAM (sprite) viewing
pub struct PpuDebugger {}

impl PpuDebugger {
    /// Create a new PPU debugger
    ///
    /// # Returns
    ///
    /// A new PPU debugger instance
    pub fn new() -> Self {
        PpuDebugger {}
    }

    /// Capture the current PPU state
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A snapshot of the PPU state
    pub fn capture_state(&self, ppu: &Ppu) -> PpuState {
        PpuState {
            scanline: ppu.scanline(),
            cycle: ppu.cycle(),
            frame: ppu.frame_count(),
            ppuctrl: ppu.ppuctrl,
            ppumask: ppu.ppumask,
            ppustatus: ppu.ppustatus,
            oam_addr: ppu.oam_addr,
            v: ppu.v,
            t: ppu.t,
            fine_x: ppu.fine_x,
            write_latch: ppu.write_latch,
            nmi_pending: ppu.nmi_pending(),
        }
    }

    /// Get sprite information from OAM
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    /// * `index` - Sprite index (0-63)
    ///
    /// # Returns
    ///
    /// Sprite information
    pub fn get_sprite_info(&self, ppu: &Ppu, index: u8) -> SpriteInfo {
        let base = (index as usize) * 4;

        SpriteInfo {
            index,
            y: ppu.read_oam(base as u8),
            tile: ppu.read_oam((base + 1) as u8),
            attributes: ppu.read_oam((base + 2) as u8),
            x: ppu.read_oam((base + 3) as u8),
        }
    }

    /// Get all sprites from OAM
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A vector of all 64 sprites
    pub fn get_all_sprites(&self, ppu: &Ppu) -> Vec<SpriteInfo> {
        (0..64).map(|i| self.get_sprite_info(ppu, i)).collect()
    }

    /// Get visible sprites (Y position < 0xF0; sprites with Y >= 0xF0 are off-screen)
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A vector of visible sprites
    pub fn get_visible_sprites(&self, ppu: &Ppu) -> Vec<SpriteInfo> {
        self.get_all_sprites(ppu)
            .into_iter()
            .filter(|s| s.y < 0xF0)
            .collect()
    }

    /// Format palette as a string
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    ///
    /// # Returns
    ///
    /// A formatted string showing all palettes
    pub fn format_palettes(&self, ppu: &Ppu) -> String {
        let mut output = String::new();

        output.push_str("Background Palettes:\n");
        for i in 0..4 {
            output.push_str(&format!("  Palette {}: ", i));
            for j in 0..4 {
                let index = i * 4 + j;
                let color = ppu.palette_ram[index];
                output.push_str(&format!("${:02X} ", color));
            }
            output.push('\n');
        }

        output.push_str("\nSprite Palettes:\n");
        for i in 0..4 {
            output.push_str(&format!("  Palette {}: ", i));
            for j in 0..4 {
                let index = 16 + i * 4 + j;
                let color = ppu.palette_ram[index];
                output.push_str(&format!("${:02X} ", color));
            }
            output.push('\n');
        }

        output
    }

    /// Format OAM as a string
    ///
    /// # Arguments
    ///
    /// * `ppu` - Reference to the PPU
    /// * `visible_only` - If true, only show visible sprites
    ///
    /// # Returns
    ///
    /// A formatted string showing sprite information
    pub fn format_oam(&self, ppu: &Ppu, visible_only: bool) -> String {
        let mut output = String::new();

        let sprites = if visible_only {
            self.get_visible_sprites(ppu)
        } else {
            self.get_all_sprites(ppu)
        };

        output.push_str(&format!("Sprites ({} total):\n", sprites.len()));

        for sprite in sprites {
            output.push_str(&format!("  {}\n", sprite.format()));
        }

        output
    }
}

impl Default for PpuDebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_debugger_creation() {
        let _debugger = PpuDebugger::new();
    }

    #[test]
    fn test_sprite_info_palette() {
        let sprite = SpriteInfo {
            index: 0,
            y: 100,
            tile: 0x42,
            attributes: 0b00000010, // Palette 2
            x: 50,
        };

        assert_eq!(sprite.palette(), 2);
    }

    #[test]
    fn test_sprite_info_flags() {
        let sprite = SpriteInfo {
            index: 0,
            y: 100,
            tile: 0x42,
            attributes: 0b11100000, // Behind BG, flip H, flip V
            x: 50,
        };

        assert!(sprite.behind_background());
        assert!(sprite.flip_horizontal());
        assert!(sprite.flip_vertical());
    }

    #[test]
    fn test_ppu_state_format_ppuctrl() {
        let state = PpuState {
            scanline: 0,
            cycle: 0,
            frame: 0,
            ppuctrl: 0x90, // NMI enabled, BG pattern $1000
            ppumask: 0,
            ppustatus: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            fine_x: 0,
            write_latch: false,
            nmi_pending: false,
        };

        let formatted = state.format_ppuctrl();
        assert!(formatted.contains("NMI"));
        assert!(formatted.contains("BG@$1000"));
    }

    #[test]
    fn test_ppu_state_format_ppumask() {
        let state = PpuState {
            scanline: 0,
            cycle: 0,
            frame: 0,
            ppuctrl: 0,
            ppumask: 0x18, // Show sprites and background
            ppustatus: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            fine_x: 0,
            write_latch: false,
            nmi_pending: false,
        };

        let formatted = state.format_ppumask();
        assert!(formatted.contains("ShowSPR"));
        assert!(formatted.contains("ShowBG"));
    }

    #[test]
    fn test_ppu_state_format_ppustatus() {
        let state = PpuState {
            scanline: 0,
            cycle: 0,
            frame: 0,
            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0xC0, // VBlank and Sprite 0 hit
            oam_addr: 0,
            v: 0,
            t: 0,
            fine_x: 0,
            write_latch: false,
            nmi_pending: false,
        };

        let formatted = state.format_ppustatus();
        assert!(formatted.contains("VBlank"));
        assert!(formatted.contains("Spr0Hit"));
    }
}
