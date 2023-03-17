use core::cell::RefCell;
use crate::mapper::Mapper;

type _Mapper = RefCell<Box<dyn Mapper>>;

const NAMETABLE_SIZE: usize = 0x1000;
const PALETTE_SIZE: usize = 0x20;

pub struct BUSPPU<'a> {
    mapper: &'a _Mapper,
    // Each byte in the nametable controls one 8x8 pixel character cell, and each nametable has 30 rows of 32 tiles each.
    nametable: [u8; NAMETABLE_SIZE],
    palette: [u8; PALETTE_SIZE] // Enum of colors maybe?
}

impl<'a> BUSPPU<'a> {
    pub fn new(mapper: &_Mapper) -> BUSPPU {
        BUSPPU {
            mapper,
            nametable: [0; NAMETABLE_SIZE],
            palette: [0; PALETTE_SIZE]
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.mapper.borrow_mut().write_chr(addr-0x2000, val);
        } else if addr < 0x3000 {
            let addr = (addr & 0xFFF) as usize;
            self.nametable[addr] = val;
        } else if addr < 0x3F00 {
            let addr = (addr & 0xEFF) as usize;
            self.nametable[addr] = val;
        } else {
            let addr = (addr & 0xE0) as usize;
            self.palette[addr] = val;
        }
    }

    pub fn read(&self, addr: u16) -> u8 { 
        if addr < 0x2000 {
            self.mapper.borrow_mut().read_chr(addr-0x2000)
        } else if addr < 0x3000 {
            let addr = (addr & 0xFFF) as usize;
            self.nametable[addr]
        } else if addr < 0x3F00 {
            let addr = (addr & 0xEFF) as usize;
            self.nametable[addr]
        } else {
            let addr = (addr & 0xE0) as usize;
            self.palette[addr]
        }
    }
}