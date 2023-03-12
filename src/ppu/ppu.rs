//use crate::cpu::bus::BUS;
//use bitvec::prelude::*;

// The Nintendo Entertainment System (NES) has a standard display resolution of 256 × 240 pixels.

// https://www.reddit.com/r/EmuDev/comments/evu3u2/what_does_the_nes_ppu_actually_do/
// https://austinmorlan.com/posts/nes_rendering_overview/

//  https://www.nesdev.org/wiki/PPU_registers

/* - The 15 bit registers t and v are composed this way during rendering:

yyy NN YYYYY XXXXX
||| || ||||| +++++-- coarse X scroll
||| || +++++-------- coarse Y scroll
||| ++-------------- nametable select
+++----------------- fine Y scroll


* Note that while the v register has 15 bits, the PPU memory space is only 14 bits wide. The highest bit is unused for access through $2007.

*/

// https://www.nesdev.org/wiki/PPU_scrolling 
// https://www.nesdev.org/wiki/PPU_memory_map

// OAM can be viewed as an array with 64 entries. 
// Each entry has 4 bytes: the sprite Y coordinate, the sprite tile number, the sprite attribute, and the sprite X coordinate. 

// Each palette has three colors. Each 16x16 pixel area of the background can use the backdrop color and the three colors from one of the four background palettes. 
// The choice of palette for each 16x16 pixel area is controlled by bits in the attribute table at the end of each nametable. 

use super::super::cpu::Interrupt;
use Section::*;

const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1
const OAM_SIZE: usize = 0x100;

const V_T_MASK: u16 = 0x7FFF; // 15 bit
const SCROLL_MASK: u8 = 0x0F;

// I could merge "x_scroll_set" and "msb_addr_set" into one thing maybe?

// Pattern Tables:
// Each tile in the pattern table is 16 bytes, made of two planes. 
// The first plane controls bit 0 of the color; the second plane controls bit 1.
// Any pixel whose color is 0 is background/transparent
// The pattern table is divided into two 256-tile sections: $0000-$0FFF, 
// nicknamed "left", and $1000-$1FFF, nicknamed "right".
// The value written to PPUCTRL ($2000) controls whether the background and sprites use the left half ($0000-$0FFF) 
// or the right half ($1000-$1FFF) of the pattern table.

// Nametable:
// A nametable is a 1024 byte area of memory used by the PPU to lay out backgrounds.
// Each byte in the nametable controls one 8x8 pixel character cell.
// Each nametable has 30 rows of 32 tiles each, for 960 ($3C0) bytes; the rest is used by each nametable's attribute table.

// Attribute table:
// An attribute table is a 64-byte array at the end of each nametable that controls 
// which palette is assigned to each part of the background.

// The PPU addresses a 14-bit (16kB) address space.

// The low two bits of $2000 select which of the four nametables to use.
// The first write to $2005 specifies the X scroll, in pixels.
// The second write to $2005 specifies the Y scroll, in pixels.

pub struct PPU {
    registers: [u8; 8],
    even_frame: bool,
    show_background: bool,
    show_sprites: bool,
    v_blank: bool,
    sprite_zero: bool,
    vram_addr: u16,
    temp_vram_addr: u16,
    w_toggle: bool,
    bg_section: Section, // Left = $0000-$0FFF / Right = $1000-$1FFF
    sprt_section: Section, // Left = $0000-$0FFF / Right = $1000-$1FFF
    hide_bg: bool,
    hide_sprt: bool,
    x_scroll: u8, // Only first 3 bits used. (https://www.nesdev.org/wiki/PPU_scrolling)
    oam: [u8; OAM_SIZE],
    secondary_oam: [u8; 0x20], // 8 * 4 = 32
    line: usize,
    cycle: usize,
    pub oam_dma: u8,
    vram: [u8; PPU_RAM_SIZE],
}

enum Section {
    Left,
    Right
}

// https://www.nesdev.org/wiki/PPU_rendering
// The PPU renders 262 scanlines per frame. 
// +Each scanline+ lasts for +341 PPU clock cycles+ (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles),
// with each clock cycle producing one pixel.

// - For odd frames, the cycle at the end of the scanline is skipped (this is done internally by jumping directly from (339,261) to (0,0), 
// replacing the idle tick at the beginning of the first visible scanline with the last tick of the last dummy nametable fetch)
// - For even frames, the last cycle occurs normally.

// * This behavior can be bypassed by keeping rendering disabled until after this scanline has passed

// A tile consists of 4 memory fetches, each fetch requiring 2 cycles.

// Some cartridges have a CHR ROM, which holds a fixed set of graphics tile data available to the PPU.
// Other cartridges have a CHR RAM that holds data that the CPU has copied from PRG ROM through a port on the PPU. 

impl PPU {
    pub fn new() -> PPU {
        PPU {
            registers: [0; 8],
            even_frame: true,
            show_background: false,
            show_sprites: false,
            v_blank: false,
            sprite_zero: false,
            w_toggle: false,
            cycle: 0,
            line: 261,
            oam_dma: 0, // needed? maybe not. 
            oam: [0; OAM_SIZE],
            secondary_oam: [0; 0x20],
            vram: [0; PPU_RAM_SIZE],
            vram_addr: todo!(),
            temp_vram_addr: todo!(),
            bg_section: Left,
            sprt_section: Left,
            x_scroll: 0,
            hide_bg: false,
            hide_sprt: false
        }
    }

    pub fn step(&mut self) -> Interrupt {
        use Interrupt::*;
        // Post-render & vblank
        if self.line >= 240 && self.line <= 260 {
            let mut i = NULL;
            if self.cycle == 1 { 
                self.v_blank = true;
                i = NMI;
            }
            self.cycle += 1;
            return i
        }

        // Render & Pre-render
        // Visible dots
        if self.cycle > 0 && self.cycle <= 256 {
            // Pre-render only
            if self.cycle == 1 && self.line == 261 {
                self.v_blank = false;
                self.sprite_zero = false;
                // TODO: clear overflow 
            }
            // tiles here
            if self.show_background {
                let x = (((self.cycle - 1) % 8) / 8) == 1;
                if self.hide_bg || x {
                    let tile_addr = self.read(0x2000 | (self.vram_addr & 0x0FFF));
                    // https://www.nesdev.org/wiki/PPU_pattern_tables
                    let mut tile = (tile_addr as u16) * 16 + ((self.vram_addr >> 12) & 7);
                    match self.bg_section {
                        // 0HRRRRCCCCPTTT
                        // 00111111111111
                        Left => { tile &= 0x1FFF },
                        Right => { tile &= 0x3FFF }
                    }
                }
            }
        }
        // vert(v) = vert(t)each tick (Pre-render only)
        if self.cycle >= 280 && self.cycle <= 304 && self.line == 261 {
            // I dont think we need to assign this everytime.
            self.vram_addr = (self.vram_addr & 0x41F) | self.temp_vram_addr;
        }

        // Pre-render only
        if self.cycle == 339 && self.line == 261 && !self.even_frame {
            // skip to (0,0) if odd frame and on pre-render line        
            // dont need to set "even_frame" to true.
            self.cycle = 0;
            self.line = 0;
        }

        if self.cycle == 340 {
            self.cycle = 0;
            self.line += 1;
        }

        self.cycle += 1;
        return NULL
    }

    pub fn read(&self, addr: u16) -> u8 {
        // TODO
        if addr < 8 {
            let addr = (addr & 0x7) as usize;
            // TODO: apparently... need to clear "w_toggle" and "v_blank" here...
            // if addr == 2 { self.clear_ppustatus() } hmm....
            if addr == 4 { 
                self.get_oam_data()
            } else {
                self.registers[addr]
            } 
        } else {
            self.oam_dma
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        // TODO
        // OAMADDR is set to 0 during each of ticks 257–320 (the sprite tile loading interval) of the pre-render and visible scanlines. 
        // This also means that at the end of a normal complete rendered frame, OAMADDR will always have returned to 0.
        if addr < 8 {
            let addr = (addr & 0x7) as usize;
            self.registers[addr] = val;
        } else {
            self.oam_dma = val;
        }
    }
    
    fn get_oam_data(&self) -> u8 {
        // reads during vertical or forced blanking return the value from OAM at that address but do not increment.
        let oam_addr = self.registers[3] as usize;
        self.oam[oam_addr]
    }
    
    pub fn set_oam_data(&mut self, val: u8) {
        // OBS: Because changes to OAM should normally be made only during vblank, writing through OAMDATA is only effective for partial updates (it is too slow), and as described above, partial writes cause corruption. 
        // Most games will use the DMA feature through OAMDMA instead.
        let oam_addr = self.registers[3] as usize;
        self.oam[oam_addr] = val;
        self.registers[3] += 1; // hopefully no overflow...
    }

    pub fn reset_oam_addr(&mut self) {
        self.registers[3] = 0;
    }

    // $2005(PPUSCROLL) and $2006(PPUADDR) share a common write toggle w, so that the first write has one behaviour, and the second write has another. 
    // After the second write, the toggle is reset to the first write behaviour.
    // https://www.nesdev.org/wiki/PPU_scrolling
    fn write_twice(&mut self, reg_n: usize, val: u8) {
        let register = self.registers[reg_n];
        if self.w_toggle {
            if reg_n == 5 {
                let abcde = (val & 0xF8) as u16;
                let fgh = (val & 0x07) as u16;
                let t = self.temp_vram_addr & 0xC1F;
                // maybe 12 wrong?
                self.temp_vram_addr = t | abcde << 2 | fgh << 12;
            }
            if reg_n == 6 {
                let t = self.temp_vram_addr & 0x7F00;
                self.temp_vram_addr = t | val as u16;
                self.vram_addr = self.temp_vram_addr;
            }
            self.registers[reg_n] = (register & 0xF0) | (val & 0x0F);
            self.w_toggle = false;
        } else {
            if reg_n == 5 {
                let c_x_scroll = (val & 0xF8) >> 3;
                self.temp_vram_addr = (self.temp_vram_addr & 0xFFE0) | c_x_scroll as u16;
                self.x_scroll = val & 3;
            }
            if reg_n == 6 {
                let cdefgh = ((val & 0x3F) as u16) << 8;
                let t = self.temp_vram_addr & 0xFF; // not 0x40FF because bit Z(msb) is cleared.
                self.temp_vram_addr = t | cdefgh;
            }
            self.registers[reg_n] = (register & 0x0F) | (val << 4);
            self.w_toggle = true;
        }
    } 

    // Tile increments
    fn inc_v_h(&mut self) {
        // The coarse X component of v needs to be incremented when the next tile is reached.
        // https://www.nesdev.org/wiki/PPU_scrolling
        if (self.vram_addr & 0x001F) == 31 { // if coarse X == 31
            self.vram_addr &= !0x001F;       // coarse X = 0
            self.vram_addr ^= 0x0400;        // switch horizontal nametable
        } else { 
            self.vram_addr += 1              // increment coarse X
        }
    }

    fn inc_v_v(&mut self) {
        // Row 29 is the last row of tiles in a nametable.
        // To wrap to the next nametable when incrementing coarse Y from 29, the vertical nametable is switched by toggling bit 11, and coarse Y wraps to row 0.
        // 0x7000 = 11100000000
        // 0x0800 = 00010000000
        if (self.vram_addr & 0x7000) != 0x7000 { // if fine Y < 7
            self.vram_addr += 0x1000; // increment fine Y
        } else {
            self.vram_addr &= !0x7000; // fine Y = 0
            let mut y = (self.vram_addr & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                self.vram_addr ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0; // coarse Y = 0, nametable not switched
            } else {
                y += 1; // increment coarse Y
            }
            self.vram_addr = (self.vram_addr & !0x03E0) | (y << 5); // put coarse Y back into v
        }
    }

    // VRAM increment
    fn get_increment(&self) -> u8 {
        let inc = self.registers[0] & 0x4;
        if inc == 4 { 32 } else { 1 }
    }

    // VRAM address increment per CPU read/write of PPUDATA.
    fn set_vram(&mut self, val: u8) {
        // VRAM reading and writing shares the same internal address register that rendering uses. So after loading data into video memory, 
        // the program should reload the scroll position afterwards with PPUSCROLL and PPUCTRL (bits 1…0) writes in order to avoid wrong scrolling.

        // When the screen is turned off by disabling the background/sprite rendering flag with the PPUMASK or during vertical blank, 
        // you can read or write data from VRAM through this port. 
        if (!self.show_background && !self.show_sprites) || self.v_blank {
            let ppu_addr = self.registers[6] as usize;
            self.vram[ppu_addr] = val;
            // Is self.get_increment() supposed to be here?
            self.registers[6] += self.get_increment(); // hopefully no overflow... 
        }
    }
    
    fn is_rendering(&self) -> bool {
        self.show_background || self.show_sprites
    }

    fn get_vram(&mut self) -> u8 {
        // TODO: buffer?

        // When reading while the VRAM address is in the range 0–$3EFF (i.e., before the palettes), the read will return the contents of an internal read buffer. 
        // This internal buffer is updated only when reading PPUDATA, and so is preserved across frames. After the CPU reads and gets the contents of the internal buffer, 
        // the PPU will immediately update the internal buffer with the byte at the current VRAM address. 
        
        todo!();
        if (!self.show_background && !self.show_sprites) || self.v_blank {
            let ppu_addr = self.registers[6] as usize;
            self.registers[6] += self.get_increment(); // hopefully no overflow...
            self.vram[ppu_addr]
        } else {
            0
        }
    }

    fn set_controller(&mut self, val: u8) {
        // TODO: PPU control register (PPUCTRL)
        self.registers[0] = val;
       // check if "<< 8" is right later.
        self.temp_vram_addr = (self.temp_vram_addr & 0x73FF) | (((val & 0x3) as u16) << 8);
        if (val & 0x10) == 0x10 { self.bg_section = Right; } else { self.bg_section = Left; }
        if (val & 8) == 8 { self.sprt_section = Right; } else { self.sprt_section = Left; }
    }

    fn set_mask(&mut self, val: u8) {
        // Sprite 0 hit does not trigger in any area where the background or sprites are hidden. <-
        // Disabling rendering  =  clear both bits 3 and 4
        self.registers[1] = val;
        // Each clock cycle = 1 pixel
        // Show background in leftmost 8 pixels of screen
        if val & 0x2 == 0x2 { self.hide_bg = true; } else { self.hide_bg = false; }
        // Show sprites in leftmost 8 pixels of screen
        if val & 0x4 == 0x4 { self.hide_sprt = true; } else { self.hide_sprt = false; }
        if val & 0x8 == 0x8 { self.show_background = true; } else { self.show_background = false; }
        if val & 0x10 == 0x10 { self.show_sprites = true; } else { self.show_sprites = false; }
    }

}