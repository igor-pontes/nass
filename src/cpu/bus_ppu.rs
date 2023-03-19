use core::cell::RefCell;
use crate::mapper::Mapper;

type _Mapper = RefCell<Box<dyn Mapper>>;

const VRAM_SIZE: usize = 0x1000;
const PALETTE_SIZE: usize = 0x20;


pub struct BUSPPU<'a> {
    mapper: &'a _Mapper,
    // Each byte in the nametable controls one 8x8 pixel character cell, and each nametable has 30 rows of 32 tiles each.
    vram: [u8; VRAM_SIZE],
    palette: [u8; PALETTE_SIZE],
    nametable0: usize,
    nametable1: usize,
    nametable2: usize,
    nametable3: usize
}

impl<'a> BUSPPU<'a> {
    pub fn new(mapper: &'a _Mapper) -> BUSPPU {
        let mut b = BUSPPU {
            mapper,
            vram: [0; VRAM_SIZE],
            palette: [0; PALETTE_SIZE],
            nametable0: 0,
            nametable1: 0,
            nametable2: 0,
            nametable3: 0,
        };
        b.set_mirroring();
        b
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.mapper.borrow_mut().write_chr(addr, val);
        } else if addr < 0x3000 {
            let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr }; // if addr is a mirror
            let index = (addr & 0x3FF) as usize; // each nametable = 0x400 size
            if addr < 0x2400 {
                self.vram[self.nametable0 + index] = val;
            } else if addr < 2800 {
                self.vram[self.nametable1 + index] = val;
            } else if addr < 0x2C00 {
                self.vram[self.nametable2 + index] = val;
            } else {
                self.vram[self.nametable3 + index] = val;
            }
        } else {
            self.palette[self.get_palette_addr(addr & 0x1F)] = val;
        }
    }
        
    pub fn read(&self, addr: u16) -> u8 { 
        if addr < 0x2000 {
            self.mapper.borrow_mut().read_chr(addr)
        } else if addr < 0x3F00 {
            let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr }; // if addr is a mirror
            let index = (addr & 0x3FF) as usize; // each nametable = 0x400 size
            if addr < 0x2400 {
                self.vram[self.nametable0 + index]
            } else if addr < 2800 {
                self.vram[self.nametable1 + index]
            } else if addr < 0x2C00 {
                self.vram[self.nametable2 + index]
            } else {
                self.vram[self.nametable3 + index]
            }
        } else {
            self.palette[self.get_palette_addr(addr & 0x1F)]
        }
    }

    fn get_palette_addr(&self, addr: u16) -> usize {
        // https://www.nesdev.org/wiki/PPU_palettes#The_background_palette_hack
        let addr = addr as usize;
        if addr >= 0x10 && addr % 4 == 0 { addr & 0xF } else { addr }
    }

    fn set_mirroring(&mut self) {
        use crate::cartridge::Mirroring::*;
        match self.mapper.borrow_mut().get_mirroring() {
            Horizontal => {
                (self.nametable0, self.nametable1, self.nametable2, self.nametable3) = (0, 0, 0x400, 0x400);
            },
            Vertical => {
                (self.nametable0, self.nametable1, self.nametable2, self.nametable3) = (0, 0x800, 0, 0x800);
            },
            FourScreen => {
                (self.nametable0, self.nametable1, self.nametable2, self.nametable3) = (0, 0x400, 0x800, 0xC00);
            }
        }
    }
}