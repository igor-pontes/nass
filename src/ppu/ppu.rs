//use crate::cpu::bus::BUS;
//use bitvec::prelude::*;

// The Nintendo Entertainment System (NES) has a standard display resolution of 256 × 240 pixels.

// https://www.reddit.com/r/EmuDev/comments/evu3u2/what_does_the_nes_ppu_actually_do/
// https://austinmorlan.com/posts/nes_rendering_overview/

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

// I could merge "x_scroll_set" and "msb_addr_set" into one thing maybe?

pub struct PPU {
    registers: [u8; 8],
    status: Status,
    even_frame: bool,
    show_background: bool,
    show_sprites: bool,
    v_blank: bool,
    increment_vram: u8,
    x_scroll_set: bool,
    msb_addr_set: bool,
    interrupt: Interrupt,
    cycle: usize,
    oam_dma: u8,
    oam: [u8; OAM_SIZE],
    vram: [u8; PPU_RAM_SIZE]
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
            increment_vram: 1,
            x_scroll_set: false,
            msb_addr_set: false,
            interrupt: NULL,
            cycle: 0,
            oam_dma: 0,
            oam: [0; OAM_SIZE],
            vram: [0; PPU_RAM_SIZE]
        }
    }

    pub fn step(&mut self) -> Interrupt {
        use Status::*;
        match self.status {
            PreRender => {
                if self.cycle == 1 {
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
    
    fn set_oam_data(&mut self, val: u8) {
        // OBS: Because changes to OAM should normally be made only during vblank, writing through OAMDATA is only effective for partial updates (it is too slow), and as described above, partial writes cause corruption. 
        // Most games will use the DMA feature through OAMDMA instead.

        // reads during vertical or forced blanking return the value from OAM at that address but do not increment.

        // Writes to OAMDATA during rendering (on the pre-render line and the visible lines 0–239, provided either sprite or background rendering is enabled) do not modify values in OAM, 
        // but do perform a glitchy increment of OAMADDR
        let oam_addr = self.registers[3] as usize;
        self.oam[oam_addr] = val;
        self.registers[3] += 1; // hopefully no overflow...
    }

    // Both "set_scroll" and "set_ppuaddr" do the same thing. Maybe merge into a single function?
    fn set_scroll(&mut self, val: u8) {
        let oam_scroll = self.registers[5];
        if self.x_scroll_set {
            self.registers[5] = (oam_scroll & 0xF0) | (val & 0x0F);
            self.x_scroll_set = false;
        } else {
            self.registers[5] = (oam_scroll & 0x0F) | (val << 4);
            self.x_scroll_set = true;
        }
    } 
    fn set_ppuaddr(&mut self, val: u8) {
        // The CPU writes to VRAM through a pair of registers on the PPU. First it loads an address into PPUADDR, and then it writes repeatedly to PPUDATA to fill VRAM.
        // Valid addresses are $0000–$3FFF; higher addresses will be mirrored down.
        let ppu_addr = self.registers[6];
        if self.msb_addr_set {
            self.registers[6] = (ppu_addr & 0xF0) | (val & 0x0F);
            self.msb_addr_set = false;
        } else {
            self.registers[6] = (ppu_addr & 0x0F) | (val << 4);
            self.msb_addr_set = true;
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

    fn set_mask(&mut self) {
        // TODO: PPU mask register (PPUMASK), call on write.
        // This register controls the rendering of sprites and backgrounds, as well as colour effects.
        // set both "show_background" and "show_sprite" here.

        // If either of bits 3 or 4 is enabled, at any time outside of the vblank interval the PPU will be making continual use to the PPU address and data bus to fetch tiles to render,
        // as well as internally fetching sprite data from the OAM

        // If you wish to make changes to PPU memory outside of vblank (via $2007), you must set both of these bits to 0 to disable rendering and prevent conflicts.
        // Sprite 0 hit does not trigger in any area where the background or sprites are hidden.
    }

    fn set_status(&mut self) {
        // Sprite 0 Hit:  Set when a nonzero pixel of sprite 0 overlaps
        // a nonzero background pixel; cleared at dot 1 of the pre-render (clear_ppustatus)
        // line.  Used for raster timing. (Maybe a separate function called "set_sprite_hit"?)
    }

    fn clear_vblank(&mut self) {
        // ?
        // Reading the status register will clear bit 7 (how to implement this behaviour?) and also the address latch used by PPUSCROLL and PPUADDR. (Ignore this for now...)
        // It does not clear the sprite 0 hit or overflow bit.
        self.registers[2] &= 0x1F;
    }

    fn clear_status(&mut self) {
        // Not cleared until the end of the next vertical blank.
        // 00011111
        self.registers[2] &= 0x1F;
    }

}