//! PPU module implementing the Game Boy graphics processor.
//!
//! This module handles VRAM management, tile rendering, and video output
//! for the Game Boy display. It includes support for LCD I/O registers
//! that control the PPU's operation.

const VRAM_BEGIN: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;
const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

// LCD I/O Register Addresses
const LCDC_ADDR: u16 = 0xFF40;
const STAT_ADDR: u16 = 0xFF41;
const SCY_ADDR: u16 = 0xFF42;
const SCX_ADDR: u16 = 0xFF43;
const LY_ADDR: u16 = 0xFF44;
const LYC_ADDR: u16 = 0xFF45;
const DMA_ADDR: u16 = 0xFF46;
const BGP_ADDR: u16 = 0xFF47;
const OBP0_ADDR: u16 = 0xFF48;
const OBP1_ADDR: u16 = 0xFF49;
const WY_ADDR: u16 = 0xFF4A;
const WX_ADDR: u16 = 0xFF4B;

#[derive(Copy, Clone)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

type Tile = [[TilePixelValue; 8]; 8];
fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

pub(crate) struct GPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    // LCD I/O Registers
    lcdc: u8, // 0xFF40 - LCD Control
    stat: u8, // 0xFF41 - LCD Status
    scy: u8,  // 0xFF42 - Scroll Y
    scx: u8,  // 0xFF43 - Scroll X
    ly: u8,   // 0xFF44 - LCD Y-Coordinate
    lyc: u8,  // 0xFF45 - LY Compare
    dma: u8,  // 0xFF46 - DMA Transfer and Start Address
    bgp: u8,  // 0xFF47 - BG Palette Data
    obp0: u8, // 0xFF48 - OBJ Palette 0 Data
    obp1: u8, // 0xFF49 - OBJ Palette 1 Data
    wy: u8,   // 0xFF4A - Window Y Position
    wx: u8,   // 0xFF4B - Window X Position
    dot_clock: u32,
}

impl GPU {
    pub(crate) fn new() -> GPU {
        GPU {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
            lcdc: 0x91, // Default value: display on, BG on
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0xFC, // Default palette
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            dot_clock: 0,
        }
    }

    /// Advance PPU timing by a batch of CPU cycles.
    pub fn tick(&mut self, cycles: u16) {
        self.dot_clock += cycles as u32;

        while self.dot_clock >= 456 {
            self.dot_clock -= 456;
            self.ly = (self.ly + 1) % 154;
            self.update_coincidence_flag();

            if self.ly >= 144 {
                // Mode 1 (V-Blank)
                self.stat = (self.stat & 0xFC) | 0x01;
            }
        }

        if self.ly < 144 {
            self.stat = (self.stat & 0xFC)
                | if self.dot_clock <= 80 {
                    0x02
                } else if self.dot_clock <= 252 {
                    0x03
                } else {
                    0x00
                };
        }
    }

    /// Update STAT bit 2 (coincidence flag) to reflect whether LY == LYC.
    fn update_coincidence_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
    }

    pub fn read_vram(&self, address: usize) -> u8 {
        self.vram[address]
    }

    pub fn write_vram(&mut self, index: usize, value: u8) {
        self.vram[index] = value;
        // If our index is greater than 0x1800, we're not writing to the tile set storage
        // so we can just return.
        if index >= 0x1800 {
            return;
        }

        // Tiles rows are encoded in two bytes with the first byte always
        // on an even address. Bitwise ANDing the address with 0xffe
        // gives us the address of the first byte.
        // For example: `12 & 0xFFFE == 12` and `13 & 0xFFFE == 12`
        let normalized_index = index & 0xFFFE;

        // First we need to get the two bytes that encode the tile row.
        let byte1 = self.vram[normalized_index];
        let byte2 = self.vram[normalized_index + 1];

        // A tiles is 8 rows tall. Since each row is encoded with two bytes a tile
        // is therefore 16 bytes in total.
        let tile_index = index / 16;
        // Every two bytes is a new row
        let row_index = (index % 16) / 2;

        // Now we're going to loop 8 times to get the 8 pixels that make up a given row.
        for pixel_index in 0..8 {
            // To determine a pixel's value we must first find the corresponding bit that encodes
            // that pixels value:
            // 1111_1111
            // 0123 4567
            //
            // As you can see the bit that corresponds to the nth pixel is the bit in the nth
            // position *from the left*. Bits are normally indexed from the right.
            //
            // To find the first pixel (a.k.a pixel 0) we find the left most bit (a.k.a bit 7). For
            // the second pixel (a.k.a pixel 1) we first the second most left bit (a.k.a bit 6) and
            // so on.
            //
            // We then create a mask with a 1 at that position and 0s everywhere else.
            //
            // Bitwise ANDing this mask with our bytes will leave that particular bit with its
            // original value and every other bit with a 0.
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            // If the masked values are not 0 the masked bit must be 1. If they are 0, the masked
            // bit must be 0.
            //
            // Finally we can tell which of the four tile values the pixel is. For example, if the least
            // significant byte's bit is 1 and the most significant byte's bit is also 1, then we
            // have tile value `Three`.
            let value = match (lsb != 0, msb != 0) {
                (true, true) => TilePixelValue::Three,
                (false, true) => TilePixelValue::Two,
                (true, false) => TilePixelValue::One,
                (false, false) => TilePixelValue::Zero,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }

    /// Read an LCD I/O register
    ///
    /// # Arguments
    /// * `addr` - The address of the register (0xFF40-0xFF4B)
    ///
    /// # Returns
    /// The value of the register, or 0 if the address is not a valid LCD register
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            LCDC_ADDR => self.lcdc,
            STAT_ADDR => self.stat,
            SCY_ADDR => self.scy,
            SCX_ADDR => self.scx,
            LY_ADDR => self.ly,
            LYC_ADDR => self.lyc,
            DMA_ADDR => self.dma,
            BGP_ADDR => self.bgp,
            OBP0_ADDR => self.obp0,
            OBP1_ADDR => self.obp1,
            WY_ADDR => self.wy,
            WX_ADDR => self.wx,
            _ => 0,
        }
    }

    /// Write to an LCD I/O register
    ///
    /// # Arguments
    /// * `addr` - The address of the register (0xFF40-0xFF4B)
    /// * `value` - The value to write
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            LCDC_ADDR => {
                let was_enabled = self.lcdc & 0x80 != 0;
                let is_enabled = value & 0x80 != 0;
                self.lcdc = value;

                if was_enabled && !is_enabled {
                    // Disabling the LCD resets LY, the internal dot clock, and
                    // the STAT mode bits to Mode 0.
                    self.ly = 0;
                    self.dot_clock = 0;
                    self.stat &= 0xFC;
                    self.update_coincidence_flag();
                } else if !was_enabled && is_enabled {
                    // Re-enabling the LCD restarts scanning from line 0.
                    self.update_coincidence_flag();
                }
            }
            STAT_ADDR => self.stat = (value & 0x78) | (self.stat & 0x87),
            SCY_ADDR => self.scy = value,
            SCX_ADDR => self.scx = value,
            LY_ADDR => {
                // LY is read-only in hardware; writes reset it to 0.
                self.ly = 0;
                self.update_coincidence_flag();
            }
            LYC_ADDR => {
                self.lyc = value;
                self.update_coincidence_flag();
            }
            DMA_ADDR => self.dma = value,
            BGP_ADDR => self.bgp = value,
            OBP0_ADDR => self.obp0 = value,
            OBP1_ADDR => self.obp1 = value,
            WY_ADDR => self.wy = value,
            WX_ADDR => self.wx = value,
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_write_preserves_mode_bits() {
        let mut gpu = GPU::new();
        // Advance dot_clock past 80 to transition into Mode 3 (0b11)
        gpu.tick(100);
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x03, 0x03);

        // Attempt to overwrite STAT with all 1s
        gpu.write_register(STAT_ADDR, 0xFF);

        // Mode bits (0-1) must remain Mode 3 (0b11)
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x03, 0x03);
    }

    #[test]
    fn stat_write_preserves_coincidence_bit() {
        let mut gpu = GPU::new();
        // Manually set bit 2 on internal stat to simulate a match
        gpu.stat |= 0x04;
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x04);

        // Attempt to overwrite STAT with 0x00
        gpu.write_register(STAT_ADDR, 0x00);

        // Bit 2 must remain set to 1
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x04);
    }

    #[test]
    fn stat_write_updates_interrupt_enable_bits() {
        let mut gpu = GPU::new();
        // Write interrupt enables (bits 3-6)
        gpu.write_register(STAT_ADDR, 0b0111_1000);

        assert_eq!(gpu.read_register(STAT_ADDR) & 0x78, 0b0111_1000);
    }

    #[test]
    fn coincidence_flag_sets_when_ly_reaches_lyc() {
        let mut gpu = GPU::new();
        gpu.write_register(LYC_ADDR, 0x10);

        // Tick forward by 16 scanlines (456 cycles * 16 = 7296 cycles)
        gpu.tick(456 * 16);

        assert_eq!(gpu.read_register(LY_ADDR), 0x10);
        // Bit 2 (0x04) of STAT must be 1
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x04);
    }

    #[test]
    fn coincidence_flag_clears_on_non_matching_line() {
        let mut gpu = GPU::new();
        gpu.write_register(LYC_ADDR, 0x10);

        // Tick forward by 15 scanlines (LY = 0x0F)
        gpu.tick(456 * 15);

        assert_eq!(gpu.read_register(LY_ADDR), 0x0F);
        // Bit 2 (0x04) of STAT must be 0
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x00);
    }

    #[test]
    fn coincidence_flag_updates_immediately_on_lyc_write() {
        let mut gpu = GPU::new();
        // Tick to line 20
        gpu.tick(456 * 20);

        assert_eq!(gpu.read_register(LY_ADDR), 20);
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x00);

        // Write matching value directly to LYC
        gpu.write_register(LYC_ADDR, 20);

        // Bit 2 must update immediately without needing to tick
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x04);
    }

    #[test]
    fn coincidence_flag_clears_on_lyc_write_that_breaks_match() {
        let mut gpu = GPU::new();
        // Default: LY = 0, LYC = 0. Manually set coincidence for setup
        gpu.stat |= 0x04;

        // Write a non-matching value to LYC
        gpu.write_register(LYC_ADDR, 0x50);

        // Bit 2 must clear immediately
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x00);
    }

    #[test]
    fn coincidence_reevaluated_after_lcd_reset() {
        let mut gpu = GPU::new();

        // Advance to line 10 so LY (10) != LYC (0), clearing coincidence (bit 2 = 0)
        gpu.tick(456 * 10);
        assert_eq!(gpu.read_register(LY_ADDR), 10);
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x00);

        // Disable LCD (LCDC bit 7 -> 0)
        gpu.write_register(LCDC_ADDR, 0x11); // 0x91 & !0x80 = 0x11
        assert_eq!(gpu.read_register(LY_ADDR), 0);

        // Re-enable LCD (LCDC bit 7 -> 1)
        gpu.write_register(LCDC_ADDR, 0x91);

        // LY is reset to 0, matching default LYC (0) -> coincidence bit 2 must be 1
        assert_eq!(gpu.read_register(STAT_ADDR) & 0x04, 0x04);
    }
}