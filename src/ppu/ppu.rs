//use crate::cpu::bus::BUS;
//use bitvec::prelude::*;

/*  https://www.nesdev.org/wiki/PPU_registers
0x0: PPUCTRL (various flags controlling PPU operation)

0x1: PPUMASK (This register controls the rendering of sprites and backgrounds, as well as colour effects. )

0x2: PPUSTATUS (This register reflects the state of various functions inside the PPU. It is often used for determining timing. 
    To determine when the PPU has reached a given pixel of the screen, put an opaque (non-transparent) pixel of sprite 0 there. )

0x3: OAMADDR (Write the address of OAM you want to access here.)

0x4: OAMDATA (Write OAM data here. Writes will increment OAMADDR after the write; 
    reads during vertical or forced blanking return the value from OAM at that address but do not increment. )

0x5: PPUSCROLL (This register is used to change the scroll position, that is, 
    to tell the PPU which pixel of the nametable selected through PPUCTRL should be at the top left corner of the rendered screen)

0x6: PPUADDR (After reading PPUSTATUS to reset the address latch, write the 16-bit address of VRAM you want to access here)

0x7: PPUDATA ()

0x4014: OAMDMA
*/

// https://www.nesdev.org/wiki/PPU_memory_map

// OAM can be viewed as an array with 64 entries. 
// Each entry has 4 bytes: the sprite Y coordinate, 
// the sprite tile number, the sprite attribute, and the sprite X coordinate. 

// Each palette has three colors. Each 16x16 pixel area of the background can use the backdrop color and the three colors from one of the four background palettes. 
// The choice of palette for each 16x16 pixel area is controlled by bits in the attribute table at the end of each nametable. 

use super::super::cpu::Interrupt;

const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1
const OAM_SIZE: usize = 0x100;

const fetch_first_two_tiles_cycle: usize = 321;
const unused_cycle: usize = 337;

enum Status {
    PreRender,
    PostRender, // 240 scanline
    Render,
    VerticalBlank // 241-260 scanlines (241 = vblank NMI set)
}

pub struct PPU {
    pub registers: [u8; 8],
    status: Status,
    even_frame: bool,
    interrupt: Interrupt,
    cycle: usize,
    pub oam_dma: u8,
    oam: [u8; OAM_SIZE],
    ram: [u8; PPU_RAM_SIZE]
}

// https://www.nesdev.org/wiki/PPU_rendering
// The PPU renders 262 scanlines per frame. 
// +Each scanline+ lasts for +341 PPU clock cycles+ (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles),
// with each clock cycle producing one pixel.

// Pre-render scanline (-1 or 261)
//  - dummy scanline, whose sole purpose is to fill the "shift registers" with the data for the first two tiles of the next scanline. 
// Visible scanlines (0-239)
//  - contain the graphics to be displayed on the screen. (includes the rendering of both the background and the sprites.)
// Post-render scanline (240) 
//  - The PPU just idles during this scanline.
// Vertical blanking lines (241-260)
//  - VBlank flag of the PPU is set at tick 1 (the second tick) of scanline 241, where the VBlank NMI also occurs.

// - For odd frames, the cycle at the end of the scanline is skipped (this is done internally by jumping directly from (339,261) to (0,0), 
// replacing the idle tick at the beginning of the first visible scanline with the last tick of the last dummy nametable fetch)
// - For even frames, the last cycle occurs normally.
// * This behavior can be bypassed by keeping rendering disabled until after this scanline has passed
// (A "frame" contains all states.)

// At the beginning of each scanline, the data for the first two tiles is already loaded into the shift registers (and ready to be rendered), 
// so the first tile that gets fetched is Tile 3. 

// A tile consists of 4 memory fetches, each fetch requiring 2 cycles.

// Some cartridges have a CHR ROM, which holds a fixed set of graphics tile data available to the PPU.
// Other cartridges have a CHR RAM that holds data that the CPU has copied from PRG ROM through a port on the PPU. 

impl PPU {
    pub fn new() -> PPU {
        use Interrupt::*;
        PPU {
            registers: [0; 8], // 
            status: Status::PreRender,
            even_frame: true,
            interrupt: NULL,
            cycle: 0,
            oam_dma: 0,
            oam: [0; OAM_SIZE],
            ram: [0; PPU_RAM_SIZE]
        }
    }

    pub fn step(&mut self) -> Interrupt {
        use Status::*;
        match self.status {
            PreRender => {
                if self.cycle == 1 {
                    self.clear_ppustatus();
                }
                if self.cycle == fetch_first_two_tiles_cycle {
                    // TODO
                }
                if self.cycle == unused_cycle {
                    // TODO
                }
            },
            Render => {
                // program should not access PPU memory during this time, unless rendering is turned off.
            },
            PostRender => {},
            VerticalBlank => {},
        }
        if self.even_frame { self.even_frame = false } else { self.even_frame = true }
        if self.cycle == 340 { self.cycle = 0; } else { self.cycle += 8; } // is this right?
        self.interrupt
    }

    pub fn read(&mut self) -> u8 {
        unimplemented!()
    }
    
    fn clear_ppustatus(&mut self) {
        // = 01111111
        self.registers[2] & 0x1F;
    }

}