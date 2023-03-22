// The Nintendo Entertainment System (NES) has a standard display resolution of 256 × 240 pixels.
// OAM can be viewed as an array with 64 entries. 
// Each entry has 4 bytes: the sprite Y coordinate, the sprite tile number, the sprite attribute, and the sprite X coordinate. 

use {
    crate::cpu::BUSPPU,
    super::super::cpu::Interrupt,
    Section::*,
    super::color::*,
    crate::scene::Scene,
};

// Debug
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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

pub struct PPU<'a> {
    even_frame: bool,
    show_background: bool,
    show_sprites: bool,
    v_blank: bool,
    sprite_zero: bool,
    overflow: bool,
    going_across: bool,
    sprt_size: usize,
    vram_addr: u16,
    temp_vram_addr: u16,
    w_toggle: bool,
    bg_section: Section, // Left = $0000-$0FFF / Right = $1000-$1FFF
    sprt_section: Section, // Left = $0000-$0FFF / Right = $1000-$1FFF
    hide_bg: bool,
    hide_sprt: bool,
    x_scroll: u8, // Only first 3 bits used. (https://www.nesdev.org/wiki/PPU_scrolling)
    oam: [u8; 0x100], // 64 entries (256 / 4)
    secondary_oam: [usize; 8], // (entries for "oam")
    line: usize,
    cycle: usize,
    frame: [[Color; 0x100]; 0xF0],
    scene: Scene,
    bus: BUSPPU<'a>,
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

// impl<'a> introduces a new lifetime parameter for the whole impl block. 
// It is then used in the type: impl<'a> Type<'a> { .. }
impl<'a> PPU<'a> {
    //pub fn new(bus: &'a BUS<'a>) -> PPU<'a> {
    pub fn new(bus: BUSPPU, scene: Scene) -> PPU {
        PPU {
            even_frame: true,
            show_background: false,
            show_sprites: false,
            v_blank: false,
            overflow: false,
            sprite_zero: false,
            w_toggle: false,
            cycle: 1,
            line: 261,
            oam: [0; 0x100],
            secondary_oam: [0; 8],
            vram_addr: 0,
            temp_vram_addr: 0,
            bg_section: Left,
            sprt_section: Left,
            sprt_size: 0,
            x_scroll: 0,
            hide_bg: false,
            hide_sprt: false,
            frame: [[Color::new(); 0x100]; 0xF0],
            going_across: true,
            scene,
            bus
        }
    }

    // Outside of rendering, reads from or writes to $2007 will add either 1 or 32 to v depending on the VRAM increment bit set via $2000
    pub fn step(&mut self) -> Interrupt {
        use Interrupt::*;
        if self.even_frame { self.even_frame = false; } else { self.even_frame = true; }
        // Each dot = 1 pixel
        // Post-render & vblank
        let mut interrupt = NULL;
        
        if self.line >= 240 && self.line <= 260 {
            if self.cycle == 1 && self.line == 240 {
                for x in 0..256 {
                    for y in 0..240 {
                        let value = &self.frame[x as usize][y as usize].to_hex();
                        self.scene.set_pixel(x, y, value);
                    }
                }
            }
            if self.cycle == 1 && self.line == 241 { 
                self.v_blank = true;
                interrupt = NMI;
            }
            if self.cycle == 340 {
                self.cycle = 0;
                self.line += 1;
            }
            self.cycle += 1;
            return interrupt
        }
        
        // Render & Pre-render
        if self.cycle == 1 && self.line == 261 {
            self.v_blank = false;
            self.sprite_zero = false;
            self.overflow = false;
        }

        if self.cycle > 0 && self.cycle < 256 && self.line != 261 { // 256?
            // Visible dots
            // While the system palette contains a total of 64 colors, a single frame has its own palette that is a subset of the system palette. 
            // Let’s call that set of colors the frame palette
            let (mut bg_color, mut sprt_color, mut bg_opaque, mut sprt_opaque, mut sprt_foreground) = (0, 0, false, true, true);
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
                    // Exactly which bit is fetched depends on the fine X scroll.
                    // If only the bit in the first plane is set to 1: The pixel's color index is 1.
                    // "7 ^ x_fine" = subtracting
                    bg_color = (((self.read(addr) as u16) >> (7 ^ x_fine)) & 1) as u8;  // plane 0 (pattern table)
                    // If only the bit in the second plane is set to 1: The pixel's color index is 2. (Thats why << 1)
                    bg_color |= ((((self.read(addr + 8) as u16) >> (7 ^ x_fine)) & 1) << 1) as u8;  // plane 0 + plane 1 (pattern table)
                    // indices for palette, 1 byte represents 1 tile
                    // Each byte controls the palette of a 32×32 pixel or 4×4 tile part of the nametable.
                    // With each tile being 8x8 pixels.
             
                    bg_opaque = if bg_color != 0 { true } else { false };
             
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
                for i in self.secondary_oam {
                    if 0 > (x as i32 - self.oam[i * 4 + 3] as i32) || (x as i32 - self.oam[i * 4 + 3] as i32) >= 8 { continue; }
                    // fine_x sprite offset 
                    let sprt_x = self.oam[i * 4 + 3] as usize;
                    // if (x - sprt_x) >= 0, render until next 8x8 tile(each 8 cycle period).
             
                    let (sprt_y, sprt_tile, sprt_attr) = (self.oam[i * 4 + 0] as usize, self.oam[i * 4 + 1] as usize, self.oam[i * 4 + 2] as usize);
                    let mut x_shift = (x - sprt_x) % 8;
                    let mut y_offset = (y - sprt_y) % self.sprt_size;
             
                    // https://www.nesdev.org/wiki/PPU_OAM
                    if (sprt_attr & 0x40) == 0 { x_shift ^= 7 } 
                    if (sprt_attr & 0x80) != 0 { y_offset ^= self.sprt_size - 1 }
             
                    //let mut addr = 0;
                    let addr = if self.sprt_size == 8 {
                        let addr = (sprt_tile * 16 + sprt_y) as u16;
                        match self.sprt_section {
                            Left => addr & 0x0FFF,
                            Right => addr | 0x1000
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
                        // addr = (tile >> 1) * 32 + y_offset as u16; // 00tttttt000000 (Pattern T. address)
                        // addr = (tile & 1) << 12;
                        let addr = (tile >> 1) * 32 + y_offset as u16; // 00tttttt000000 (Pattern T. address)
                        addr | (tile & 1) << 12
                    };
             
                    // Sprites with lower OAM indices are drawn in front.
                    // (For example, sprite 0 is in front of sprite 1, which is in front of sprite 63.)
                    // Pixel value from tile data (0b000vv)
                    // (x_shif) = which bit from current pattern table...
                    sprt_color = (((self.read(addr) as u16) >> (7 ^ x_shift)) & 1) as u8;
                    sprt_color |= ((((self.read(addr + 8) as u16) >> (7 ^ x_shift)) & 1) << 1) as u8;
                    
                    // review
                    if sprt_color == 0 {
                        sprt_opaque = false;
                        continue;
                    } else {
                        sprt_opaque = true;
                    };
             
                    // + 0x10 to get sprite palette
                    sprt_color += 0x10; // 0b100vv
                    // Palette number(byte 2) from OAM.
                    sprt_color += ((sprt_attr as u8) & 3) << 2; // 0b1ppcc
             
                    // https://www.nesdev.org/wiki/PPU_OAM#Byte_2
                    // 0x20 = Priority (0: in front of background; 1: behind background)
                    sprt_foreground = if sprt_attr & 0x20 == 0x20 { false } else { true };
             
                    // Set when a nonzero pixel of sprite 0 overlaps
                    // a nonzero background pixel; cleared at dot 1 of the pre-render
                    // line. Used for raster timing.
                    if !self.sprite_zero && self.show_background && i == 0 && sprt_opaque && bg_opaque {
                        self.sprite_zero = true;
                    };
             
                    break;
                } 
            }
         
            let mut palette_addr = bg_color;
             
            if !bg_opaque {
                if sprt_opaque { palette_addr = sprt_color; } else { palette_addr = 0; }
            } else {
                if sprt_foreground { palette_addr = sprt_color; }
            }
            //let test = COLORS[self.read(palette_addr as u16) as usize];
            //log(&format!("x: {} | y: {}", x, y));
            //log(&format!("TEST: {}", test));
            // broken
            self.frame[x][y].decode(COLORS[self.read(palette_addr as u16) as usize]);
        }
        
        if self.cycle == 256 && self.show_background {
            self.inc_v_v();
        }

        if self.cycle == 257 && self.show_background && self.show_sprites {
            self.vram_addr &= !0x41f;
            self.vram_addr |= self.temp_vram_addr & 0x41f;
        }

        // During each visible scanline this secondary OAM is first cleared, 
        // and then a linear search of the entire primary OAM is carried out to find sprites that are within y range for the next scanline 
        // (the sprite evaluation phase)
        log(&format!("line: {} | cycle: {}", self.line, self.cycle));
        if self.cycle == 321 {
            self.secondary_oam = [0; 8]; // reset array of entries
            let mut range = 8;
            if self.sprt_size == 16 { range = 16; }
            let mut j = 0;
            for i in 0..64 {
                let diff = self.line as i32 - (self.oam[(i * 4) as usize]) as i32;
                if 0 <= diff && diff < range {
                    if j >= 8 {
                        self.overflow = true;
                        break;
                    }
                    self.secondary_oam[j] = i;
                    j += 1;
                }
            }
            if self.line < 261 { self.line += 1; }
            //self.cycle = 0;
            //log(&format!("{:?}", self.secondary_oam));
        }
         
        // Pre-render only
        if self.cycle == 339 && self.line == 261 && !self.even_frame {
            log("not even frame and line 261");
            self.cycle = 0;
            self.line = 0;
        }

        if self.cycle == 340 { self.cycle = 0; self.line = 0; }

        self.cycle += 1;
        return interrupt
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    pub fn set_data(&mut self, value: u8) {
        self.bus.write(self.vram_addr, value);
        self.vram_addr += if self.going_across { 1 } else { 32 };
    }

    pub fn get_data(&mut self) -> u8 {
        let data = self.bus.read(self.vram_addr);
        self.vram_addr += if self.going_across { 1 } else { 32 };
        data
    }

    pub fn get_oam_data(&self, oam_addr: usize) -> u8 {
        // reads during vertical or forced blanking return the value from OAM at that address but do not increment.
        self.oam[oam_addr]
    }
    
    pub fn set_oam_data(&mut self, oam_addr: usize, val: u8) {
        self.oam[oam_addr] = val;
    }

    pub fn set_address(&mut self, reg: u8, val: u16) -> u8 {
        if self.w_toggle {
            self.temp_vram_addr = self.temp_vram_addr & 0x7F00 | val;
            self.vram_addr = self.temp_vram_addr;
            self.w_toggle = false;
            (val as u8) & 0x0F
        } else {
            self.temp_vram_addr = self.temp_vram_addr & 0xFF | (val & 0x3F) << 8;
            self.w_toggle = true;
            (reg & 0x0F) | (val as u8) << 4
        }
    }

    pub fn set_scroll(&mut self, reg: u8, val: u16) -> u8 {
        if self.w_toggle {
            self.temp_vram_addr = self.temp_vram_addr & 0xC1F | (val & 0xF8) << 2 | (val & 0x07) << 12;
            self.w_toggle = true;
            (val as u8) & 0x0F
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & 0xFFE0) | (val & 0xF8) >> 3;
            self.x_scroll = (val & 3) as u8;
            self.w_toggle = true;
            (reg & 0x0F) | (val as u8) << 4
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
    
    pub fn set_status(&mut self) {
        self.v_blank = false;
    }

    pub fn set_controller(&mut self, val: u8) {

        self.temp_vram_addr = (self.temp_vram_addr & 0x73FF) | (((val & 0x3) as u16) << 8); // check "<< 8" later.
        if (val & 4) == 4 { self.going_across = false; } else { self.going_across = true; }
        if (val & 0x10) == 0x10 { self.bg_section = Right; } else { self.bg_section = Left; }
        if (val & 8) == 8 { self.sprt_section = Right; } else { self.sprt_section = Left; }
        if (val & 0x20) == 0x20 { self.sprt_size = 16; } else { self.sprt_size = 8; }
    }

    pub fn set_mask(&mut self, val: u8) {
        // Sprite 0 hit does not trigger in any area where the background or sprites are hidden. <-
        // Disabling rendering  =  clear both bits 3 and 4
        // Each clock cycle = 1 pixel
        // Show background in leftmost 8 pixels of screen
        if val & 2 == 2 { self.hide_bg = false; } else { self.hide_bg = true; }
        // Show sprites in leftmost 8 pixels of screen
        if val & 4 == 4 { self.hide_sprt = false; } else { self.hide_sprt = true; }
        if val & 8 == 8 { self.show_background = true; } else { self.show_background = false; }
        if val & 0x10 == 0x10 { self.show_sprites = true; } else { self.show_sprites = false; }
    }
}