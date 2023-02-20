/*  https://www.nesdev.org/wiki/CPU_memory_map

    0x0000-0x07FF => 0x0800(2KB) Internal RAM - https://www.nesdev.org/wiki/Sample_RAM_map

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

/*
    The CPU expects interrupt vectors in a fixed place at the end of the cartridge space:
    $FFFA–$FFFB = NMI vector
    $FFFC–$FFFD = Reset vector
    $FFFE–$FFFF = IRQ/BRK vector
*/

use super::super::{
    ppu::*,
    apu::*,
    cartridge::*
};

const RAM_SIZE: usize = 0x800; // 0x10000 = 0xFFFF + 1
//const MEMORY_SIZE: usize = 0x10000; // 0x10000 = 0xFFFF + 1

#[derive(Debug)]
pub struct BUS {
    ram: [u8; 0x800],
    cartridge: Cartridge,
    ppu: PPU,
    apu: APU,
}

impl BUS {
    pub fn new(cartridge: Cartridge) -> BUS {
        BUS {
            ram: [0; RAM_SIZE],
            cartridge,
            ppu: PPU::new(),
            apu: APU::new(),
        }
    }
    pub fn read(&self, addr: u16) -> u8 { 
        if addr < 0x2000 { // Mirrors of $0000–$07FF 
            self.ram[(addr & 0x7FF) as usize]
        } else if addr < 0x4000 { // Mirrors of $2000–$2007
            self.ppu.registers[((addr & 0x0007)) as usize]
        } else if addr < 0x4018 {
            self.apu.registers[(addr & 0x17) as usize]
        } else if addr < 0x6000 {
            //"Not implemented"
            0x0
        } else {
            self.cartridge.read_prg(addr)
        }
    }
}