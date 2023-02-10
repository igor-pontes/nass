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

const PPU_RAM_SIZE: usize = 0x4000; // 0x4000 = 0x3FFF + 1

#[derive(Debug)]
pub struct PPU{
    pub registers: [u8; 8],
    ram: [u8; PPU_RAM_SIZE],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            registers: [0; 8],
            ram: [0; PPU_RAM_SIZE],
        }
    }
}