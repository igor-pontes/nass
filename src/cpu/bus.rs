/*  https://www.nesdev.org/wiki/CPU_memory_map

    0x0000-0x07FF => 0x0800(2KB) Internal RAM

    0x0800-0x0FFF => 0x0800(2KB)
    0x1000-0x17FF => 0x0800(2KB) Mirrors of $0000-$07FF
    0x1800-0x1FFF => 0x0800(2KB)

    0x2000-0x2007 => 0x0008(8B) NES PPU registers

    0x2008-0x3FFF => 0x1FF8 Mirrors of $2000-2007 (repeats every 8 bytes) 

    0x4000-0x4017 => 0x0018 NES APU and I/O registers # See https://www.nesdev.org/wiki/2A03

    0x4018-0x401F => 0x0008 APU and I/O functionality that is normally disabled. See CPU Test Mode.

    0x4020-0xFFFF => 0xBFE0 Cartridge space: PRG ROM, PRG RAM, and mapper registers.
*/

/*
    The NMOS 65xx processors have 256 bytes of stack memory, ranging
    from 100 to 1FF. The S register is a 8-bit offset to the stack
    page. In other words, whenever anything is being pushed on the
    stack, it will be stored to the address $0100+S.
*/
use super::{
    ppu::PPU,
    apu::APU,
    cartridge::Cartridge
};

const RAM_SIZE: usize = 0x10000; // 0x10000 = 0xFFFF + 1

#[derive(Debug)]
pub struct BUS {
    pub ram: [u8; RAM_SIZE], // CPU address space
    pub ppu: PPU,
    pub apu: APU,
    pub cartridge: Cartridge
}

impl BUS {
    pub fn new() -> BUS {
        BUS {
            ram: [0; RAM_SIZE],
            ppu: PPU::new(),
            apu: APU::new(),
            cartridge: Cartridge::new()
        }
    }
}