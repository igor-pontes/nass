//use crate::cpu::bus::BUS;
//use bitvec::prelude::*;

/*  https://www.nesdev.org/wiki/PPU_registers
0x0: PPUCTRL
0x1: PPUMASK
0x2: PPUSTATUS
0x3: OAMADDR
0x4: OAMDATA
0x5: PPUSCROLL
0x6: PPUADDR
0x7: PPUDATA

0x4014: OAMDMA
*/

// https://www.nesdev.org/wiki/PPU_memory_map

// OAM can be viewed as an array with 64 entries. 
// Each entry has 4 bytes: the sprite Y coordinate, 
// the sprite tile number, the sprite attribute, and the sprite X coordinate. 

use super::super::cpu::Interrupt;

//const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1
const OAM_SIZE: usize = 0x100;

#[derive(Debug)]
pub struct PPU {
    pub registers: [u8; 8],
    oam: [u8; OAM_SIZE],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            registers: [0; 8],
            oam: [0; OAM_SIZE],
        }
    }

    pub fn step(&mut self) -> Interrupt {
        unimplemented!()
    }

    pub fn read(&mut self) -> u8 {
        unimplemented!()
    }
}