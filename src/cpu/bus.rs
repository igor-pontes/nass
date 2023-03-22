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

use std::cell::RefCell;

use crate::mapper::Mapper;
use super::super::ppu::*;

const RAM_SIZE: usize = 0x800;
//const MEMORY_SIZE: usize = 0x10000; // 0x10000 = 0xFFFF + 1

type _Mapper = RefCell<Box<dyn Mapper>>;

pub struct BUS<'a> {
    ram: [u8; RAM_SIZE],
    mapper: &'a _Mapper,
    pub ppu: PPU<'a>,
    ppu_registers: [u8; 8],
}

impl<'a> BUS<'a> {
    pub fn new(mapper: &'a _Mapper, ppu: PPU<'a>) -> BUS<'a> {
        BUS {
            ram: [0; RAM_SIZE],
            mapper,
            ppu_registers: [0; 8],
            ppu,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram[(addr & 0x7FF) as usize] = val;
        } else if addr < 0x4000 { // Mirrors of $2000–$2007
            let reg = (addr & 7) as usize;
            let reg_data = self.ppu_registers[reg];
            if reg == 0 { self.ppu_registers[0] = val; self.ppu.set_controller(val); }
            else if reg == 1 { self.ppu_registers[1] = val; self.ppu.set_mask(val); }
            else if reg == 2 { () } // Read only.
            else if reg == 5  { self.ppu_registers[5] = self.ppu.set_scroll(reg_data, val as u16); }
            else if reg == 6 { self.ppu_registers[6] = self.ppu.set_address(reg_data, val as u16); }
            else if reg == 7 { self.ppu_registers[7] = val; self.ppu.set_data(val); }
            else { self.ppu_registers[reg] = val; }
        } else if addr == 0x4014 {
            self.ppu_registers[3] = 0;
            for i in 0..0xFF {
                let val = self.read(((val as u16) << 8) | i);
                self.ppu.set_oam_data(self.ppu_registers[3] as usize, val);
                // Writes will increment OAMADDR after the write;
                self.ppu_registers[3] += 1; 
            }
            self.ppu_registers[3] = 0;
        } else if addr < 0x6000 {
            ()
        } else {
            self.mapper.borrow_mut().write_prg(addr, val);
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { 
        if addr < 0x2000 { // Mirrors of $0000–$07FF 
            self.ram[(addr & 0x7FF) as usize]
        } else if addr < 0x4000 { // Mirrors of $2000–$2007
            let reg = (addr & 7) as usize;
            if reg == 2 { self.ppu.set_status(); 0 }
            else if reg == 4 { self.ppu.get_oam_data(self.ppu_registers[4] as usize) }
            else if reg == 7 { self.ppu.get_data() }
            else { self.ppu_registers[reg] }
        } else if addr < 0x4018 {
            if addr == 0x4016 || addr == 0x4017 {
                0 // Inputs
            } else { 
                0 
            }
        } else {
            //  Battery Backed Save or Work RAM not implemented.
            self.mapper.borrow_mut().read_prg(addr)
        }
    }

}