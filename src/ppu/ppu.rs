// The Nintendo Entertainment System (NES) has a standard display resolution of 256 × 240 pixels.
// https://archive.nes.science/nesdev-forums/f10/t9324.xhtml
// https://austinmorlan.com/posts/nes_rendering_overview/
// https://www.nesdev.org/wiki/PPU_registers
// https://www.nesdev.org/wiki/PPU_scrolling 
// https://www.nesdev.org/wiki/PPU_memory_map

// OAM can be viewed as an array with 64 entries. 
// Each entry has 4 bytes: the sprite Y coordinate, the sprite tile number, the sprite attribute, and the sprite X coordinate. 

use super::super::cpu::Interrupt;
use Section::*;

const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1
const OAM_SIZE: usize = 0x100;

const V_T_MASK: u16 = 0x7FFF; // 15 bit
const SCROLL_MASK: u8 = 0x0F;

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
    sprt_size: usize,
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
    frame: [u8; 0x3C0]
}

enum Section {
    Left,
    Right
}

// https://www.nesdev.org/wiki/PPU_rendering
// The PPU renders 262 scanlines per frame. 
// +Each scanline+ lasts for +341 PPU clock cycles+ (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles),
// with each clock cycle producing one pixel.

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

    // Outside of rendering, reads from or writes to $2007 will add either 1 or 32 to v depending on the VRAM increment bit set via $2000
    pub fn step(&mut self) -> Interrupt {
        use Interrupt::*;
        // Each step = 1 pixel
        // Post-render & vblank
        let mut i = NULL;
        if self.line >= 240 && self.line <= 260 {
            if self.cycle == 1 { 
                self.v_blank = true;
                i = NMI;
            }
            self.cycle += 1;
            return i
        }
        // Render & Pre-render
        if self.cycle > 0 && self.cycle <= 256 {
            // Visible dots
            // While the system palette contains a total of 64 colors, a single frame has its own palette that is a subset of the system palette. 
            // Let’s call that set of colors the frame palette
            // Pre-render only
            if self.cycle == 1 && self.line == 261 {
                self.v_blank = false;
                self.sprite_zero = false;
                // TODO: clear overflow 
            }
            let (mut bg_color, mut sprt_color) = (0, 0);
            let (x, y) = (self.cycle - 1, self.line);

            // tiles here
            if self.show_background {
                let x_fine = (x + (self.x_scroll) as usize) % 8;
                if !self.hide_bg || x >= 8 {
                    let v = self.vram_addr;
                    // The high bits of "vram_addr" are used for fine Y during rendering.
                    // Addressing nametable data only requires 12 bits, 
                    // with the high 2 CHR address lines fixed to the 0x2000 region
                    // Indexes into the Pattern Table.
                    let tile = self.read(0x2000 | (v & 0x0FFF)); // nametable?
                    // https://www.nesdev.org/wiki/PPU_pattern_tables
                    // The implementation of scrolling has two components. 
                    // + Two fine offsets, specifying what part of an 8x8 tile each pixel falls on. 
                    // + Two coarse offsets, specifying which tile. 
                    let mut addr = (tile as u16) * 16 + ((v >> 12) & 7);
                
                    match self.bg_section {
                        // 0(H)RRRRCCCCPTTT
                        // is the msb = 0? check if something goes wrong.
                        Left => { addr &= 0x0FFF },
                        Right => { addr |= 0x1000 }
                    }
                    // https://www.nesdev.org/wiki/PPU_palettes 
                    // Pixel value from tile data (0b000vv) ("tile data" = pattern table data)
                    // Fetch the low-order byte of an 8x1 pixel sliver of pattern table from $0000-$0FF7 or $1000-$1FF7.
                    // Fetch the high-order byte of this sliver from an address 8 bytes higher.
                    // Every cycle, a bit is fetched from the 4 background shift registers in order to create a pixel on screen. 
                    // Exactly which bit is fetched depends on the fine X scroll.
                    // If only the bit in the first plane is set to 1: The pixel's color index is 1.
                    bg_color = (((self.read(addr) as u16) >> (7 ^ x_fine)) & 1) as u8;  // plane 0 (pattern table)
                    // If only the bit in the second plane is set to 1: The pixel's color index is 2. (Thats why << 1)
                    bg_color |= ((((self.read(addr + 8) as u16) >> (7 ^ x_fine)) & 1) << 1) as u8;  // plane 0 + plane 1 (pattern table)
                    // indices for palette, 1 byte represents 1 tile
                    // Each byte controls the palette of a 32×32 pixel or 4×4 tile part of the nametable.
                    // With each tile being 8x8 pixels.

                    // Attr_table: (each tile) 2x2 (each tile = 16x16 pixels) or 4x4 (each tile = 8x8 pixels) => Nametable: (each tile) 8x8
                    let attr_table = self.read(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07));
                    // use second bit of each Coarse offset (= every 16 pixels of data, horizontal/vertical)
                    // 1 8x8 tile = 00 => 2 8x8 tile = 01 => 3 9x9 tile = 10 => 4 8x8 tile = 11 => ... (Horizontal)
                    let shift = ((v >> 4) & 4) | v & 2; // e.g. "000" = first quadrant; "010"; second quadrant; ...

                    // Palette number from attribute table or OAM (0b0pp00)
                    // select quadrant (16 pixel each) color
                    bg_color |= ((attr_table >> shift) & 3) << 2; // 0b0ppvv
                }
                if x_fine == 7 { self.inc_v_h(); } // increment x_fine each 8 cycle
            }

            if self.show_sprites && (!self.hide_sprt || x >= 8) {
                // scan every sprite from OAM?
                for i in 0..8 {
                    // fine_x sprite offset 
                    let sprt_x = self.oam[i * 4 + 3] as usize;
                    // if (x - sprt_x) >= 0, render until next 8x8 tile(each 8 cycle period).
                    if 0 > (x - sprt_x) || (x - sprt_x) >= 8 { continue; }

                    let (sprt_y, sprt_tile, sprt_attr) = (self.oam[i * 4 + 0] as usize, self.oam[i * 4 + 1] as usize, self.oam[i * 4 + 2] as usize);
                    let x_shift = (x - sprt_x) % 8;
                    let y_offset = (y - sprt_y) % self.sprt_size;

                    // https://www.nesdev.org/wiki/PPU_OAM
                    if (sprt_attr & 0x40) == 0 { x_shift ^= 7 } 
                    if (sprt_attr & 0x80) != 0 { y_offset ^= self.sprt_size - 1 }

                    let mut addr = 0;
                    if self.sprt_size == 8 {
                        addr = (sprt_tile * 16 + sprt_y) as u16;
                        match self.sprt_section {
                            Left => addr &= 0x0FFF,
                            Right => addr |= 0x1000
                        }
                    } else {
                        // tile next column = next 8 pixels of the 16 pixel sprite.
                        y_offset = (y_offset & 7) | ((y_offset & 8) << 1);
                        let tile = sprt_tile as u16;
                        /* 
                        76543210
                        ||||||||
                        |||||||+- Bank ($0000 or $1000) of tiles
                        +++++++-- Tile number of top of sprite (0 to 254; bottom half gets the next tile)
                        */
                        addr =  (tile >> 1) * 32 + y_offset as u16; // 00tttttt000000 (Pattern T. address)
                        addr = (tile & 1) << 12;
                    }
                    // Pixel value from tile data (0b000vv)
                    // (x_shif) = which bit from current pattern table...
                    sprt_color = (((self.read(addr) as u16) >> (7 ^ x_shift)) & 1) as u8;
                    sprt_color |= ((((self.read(addr + 8) as u16) >> (7 ^ x_shift)) & 1) << 1) as u8;
                    // + 0x10 to get sprite palette
                    sprt_color += 0x10; // 0b100vv
                    // Palette number from attribute table or OAM (only need top-left quadrant)
                    sprt_color += ((sprt_attr as u8) & 3) << 2; // 0b1ppcc

                    if !self.sprite_zero && self.show_background && i == 0 {
                        self.sprite_zero = true;
                    }

                    break;
                } 
            }
        }
        // vert(v) = vert(t)each tick (Pre-render only)
        if self.cycle >= 280 && self.cycle <= 304 && self.line == 261 {
            // I dont think we need to assign this every tick.
            self.vram_addr = (self.vram_addr & 0x41F) | self.temp_vram_addr;
        }

        // Pre-render only
        if self.cycle == 339 && self.line == 261 && !self.even_frame {
            // skip to (0,0) if odd frame and on pre-render line        
            // dont need to set "even_frame" to true.
            self.cycle = 0;
            self.line = 0;
        }

        // ?
        if self.cycle == 340 {
            self.cycle = 0;
            self.line += 1;
        }

        self.cycle += 1;
        return i
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
                // The low 3 bits of X sent to $2005 (first write) control the fine pixel offset within the 8x8 tile.
                // The low 3 bits goes into the separate x register, which just selects one of 8 pixels coming out of a set of shift registers. 
                // This fine X value does not change during rendering; the only thing that changes it is a $2005 first write.
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
    // Outside of rendering, reads from or writes to $2007 will add either 1 or 32 to v depending on the VRAM increment bit set via $2000. 
    // During rendering (on the pre-render line and the visible lines 0-239, provided either background or sprite rendering is enabled), 
    // it will update v in an odd way, triggering a coarse X increment and a Y increment simultaneously (with normal wrapping behavior)
    fn get_increment(&self) -> u8 {
        if self.registers[0] & 0x4 == 4 { 32 } else { 1 }
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
        if (val & 0x20) == 0x20 { self.sprt_size = 16; } else { self.sprt_size = 8; }
    }

    fn set_mask(&mut self, val: u8) {
        // Sprite 0 hit does not trigger in any area where the background or sprites are hidden. <-
        // Disabling rendering  =  clear both bits 3 and 4
        self.registers[1] = val;
        // Each clock cycle = 1 pixel
        // Show background in leftmost 8 pixels of screen
        if val & 0x2 == 0x2 { self.hide_bg = false; } else { self.hide_bg = true; }
        // Show sprites in leftmost 8 pixels of screen
        if val & 0x4 == 0x4 { self.hide_sprt = false; } else { self.hide_sprt = true; }
        if val & 0x8 == 0x8 { self.show_background = true; } else { self.show_background = false; }
        if val & 0x10 == 0x10 { self.show_sprites = true; } else { self.show_sprites = false; }
    }

}