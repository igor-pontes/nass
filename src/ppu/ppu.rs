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

const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1
const OAM_SIZE: usize = 0x100;

const V_T_MASK: u16 = 0x7FFF;
const X_SCROLL_MASK: u8 = 0x07;

enum Status {
    PreRender,
    PostRender, // 240 scanline
    Render,
    VerticalBlank // 241-260 scanlines (241 = vblank NMI set)
}

// I could merge "x_scroll_set" and "msb_addr_set" into one thing maybe?

pub struct PPU {
    registers: [u8; 8],
    status: Status,
    even_frame: bool,
    show_background: bool,
    show_sprites: bool,
    v_blank: bool,
    sprite: bool,
    increment_vram: u8,
    // Internal registers
    w_toggle: bool,
    addr_v: u16, // only 15 is used (Current VRAM address)
    addr_t: u16, // only 15 is used (Temporary VRAM address) - can also be thought of as the address of the top left onscreen tile.
    x_scroll: u8, // only 3 bits used 

    interrupt: Interrupt,
    cycle: usize,
    pub oam_dma: u8,
    oam: [u8; OAM_SIZE],
    vram: [u8; PPU_RAM_SIZE],
}

// https://www.nesdev.org/wiki/PPU_rendering
// The PPU renders 262 scanlines per frame. 
// +Each scanline+ lasts for +341 PPU clock cycles+ (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles),
// with each clock cycle producing one pixel.

// - For odd frames, the cycle at the end of the scanline is skipped (this is done internally by jumping directly from (339,261) to (0,0), 
// replacing the idle tick at the beginning of the first visible scanline with the last tick of the last dummy nametable fetch)
// - For even frames, the last cycle occurs normally.

// * This behavior can be bypassed by keeping rendering disabled until after this scanline has passed
// (A "frame" contains all states.)

// A tile consists of 4 memory fetches, each fetch requiring 2 cycles.

// Some cartridges have a CHR ROM, which holds a fixed set of graphics tile data available to the PPU.
// Other cartridges have a CHR RAM that holds data that the CPU has copied from PRG ROM through a port on the PPU. 

impl PPU {
    pub fn new() -> PPU {
        use Interrupt::*;
        PPU {
            registers: [0; 8],
            status: Status::PreRender,
            even_frame: true,
            show_background: false,
            show_sprites: false,
            v_blank: false,
            sprite: false,
            increment_vram: 1,
            w_toggle: false,
            addr_t: 0,
            addr_v: 0,
            x_scroll: 0,
            interrupt: NULL,
            cycle: 0,
            oam_dma: 0, // needed? maybe not. 
            oam: [0; OAM_SIZE],
            vram: [0; PPU_RAM_SIZE],
        }
    }

    pub fn step(&mut self) -> Interrupt {
        use Status::*;
        match self.status {
            PreRender => {
                if self.cycle == 1 {
                    self.clear_status();
                }
            },
            Render => {
                // program should not access PPU memory during this time, unless rendering is turned off.
                // https://www.nesdev.org/wiki/PPU_sprite_evaluation
            },
            PostRender => {},
            VerticalBlank => {},
        }
        if self.even_frame { self.even_frame = false } else { self.even_frame = true }
        if self.cycle == 340 { self.cycle = 0; } else { self.cycle += 8; } // is this right?
        self.interrupt
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 8 {
            let addr = (addr & 0x7) as usize;
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

        // reads during vertical or forced blanking return the value from OAM at that address but do not increment.

        // Writes to OAMDATA during rendering (on the pre-render line and the visible lines 0–239, provided either sprite or background rendering is enabled) do not modify values in OAM, 
        // but do perform a glitchy increment of OAMADDR
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
            self.registers[reg_n] = (register & 0xF0) | (val & 0x0F);
            self.w_toggle = false;
        } else {
            self.registers[reg_n] = (register & 0x0F) | (val << 4);
            self.w_toggle = true;
        }
    } 

    fn set_vram(&mut self, val: u8) {
        // VRAM reading and writing shares the same internal address register that rendering uses. So after loading data into video memory, 
        // the program should reload the scroll position afterwards with PPUSCROLL and PPUCTRL (bits 1…0) writes in order to avoid wrong scrolling.

        // When the screen is turned off by disabling the background/sprite rendering flag with the PPUMASK or during vertical blank, 
        // you can read or write data from VRAM through this port. 
        if (!self.show_background && !self.show_sprites) || self.v_blank {
            let ppu_addr = self.registers[6] as usize;
            self.vram[ppu_addr] = val;
            // After access, the video memory address will increment by an amount determined by bit 2 of $2000.
            // after access = after read?
            self.registers[6] += self.increment_vram; // hopefully no overflow...
        }
    }
    
    fn get_vram(&self) -> u8 {
        // TODO: buffer?

        // When reading while the VRAM address is in the range 0–$3EFF (i.e., before the palettes), the read will return the contents of an internal read buffer. 
        // This internal buffer is updated only when reading PPUDATA, and so is preserved across frames. After the CPU reads and gets the contents of the internal buffer, 
        // the PPU will immediately update the internal buffer with the byte at the current VRAM address. 
        
        if (!self.show_background && !self.show_sprites) || self.v_blank {
            let ppu_addr = self.registers[6] as usize;
            self.vram[ppu_addr] 
        } else {
            0 // hmm... if statement here kinda useless then...
        }
    }

    fn set_controller(&mut self) {
        // TODO: PPU control register (PPUCTRL)

    }

    fn set_mask(&mut self, val: u8) {
        // TODO: PPU mask register (PPUMASK), call on write.

        // A value of $1E or %00011110 enables all rendering, with no color effects. A value of $00 or %00000000 disables all rendering. 
        // It is usually best practice to write this register only during "vblank", to prevent partial-frame visual artifacts.

        // If either of bits 3 or 4 is enabled, at any time outside of the vblank interval the PPU will be making continual use to the PPU address and data bus to fetch tiles to render,
        // as well as internally fetching sprite data from the OAM

        // If you wish to make changes to PPU memory outside of vblank (via $2007), you must set both of these bits to 0 to disable rendering and prevent conflicts.

        // -> Sprite 0 hit does not trigger in any area where the background or sprites are hidden. <-

        // Disabling rendering  =  clear both bits 3 and 4
        if val & 0x08 == 0x08 {
            self.show_background = true;
        } else {
            self.show_background = false;
        }
        
        if val & 0x10 == 0x10 {
            self.show_sprites = true;
        } else {
            self.show_sprites = false;
        }
    }

    fn set_status(&mut self) {
        // Sprite 0 Hit:  Set when a nonzero pixel of sprite 0 overlaps
        // a nonzero background pixel; cleared at dot 1 of the pre-render (clear_ppustatus)
        // line.  Used for raster timing. (Maybe a separate function called "set_sprite_hit"?)
    }

    fn clear_status(&mut self) {
        // Not cleared until the end of the next vertical blank.
        self.v_blank = false;
        self.sprite = false;
        self.registers[2] &= 0x1F; // 00011111
    }

}