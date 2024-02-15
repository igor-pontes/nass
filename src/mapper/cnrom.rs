use std::fmt;
use super::*;

pub struct CNROM {
    chr_bank: u16,
    mirroring: Mirroring,
    prg_offset: usize,
    prg_len: usize,
    chr_offset: usize,
}

impl CNROM {
    pub fn new(prg_len: usize, _: usize, prg_offset: usize, chr_offset: usize,  mirroring: Mirroring) -> Self { 
        CNROM {
            prg_offset,
            prg_len,
            chr_offset,
            chr_bank: 0,
            mirroring,
        } 
    }
}

impl fmt::Display for CNROM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CNROM")
    }
}

impl Mapper for CNROM {
    fn get_mirroring(&self) -> Mirroring { self.mirroring }

    fn read_chr(&self, rom: *const u8, addr: u16) -> u8 { 
        let addr = (addr + self.chr_bank) as usize;
        unsafe { *(rom.wrapping_add(self.chr_offset + (addr as usize))) } 
    }

    fn read_prg(&self, rom: *const u8, addr: u16) -> u8 { 
        match addr {
            0x8000..=0xFFFF => {
                let mut addr = addr - 0x8000;
                if self.prg_len == 0x4000 && addr >= 0x4000 { 
                    addr = addr % 0x4000; 
                }
                unsafe { *(rom.wrapping_add(self.prg_offset + (addr as usize))) } 
            },
            _ => 0
        }
    }

    fn write_prg(&mut self, addr: u16, val: u8) { 
        match addr {
            0x8000..=0xFFFF => {
                self.chr_bank = ((val as u16) & 0x3) * 0x2000;
            },
            _ => () 
        }
    }

    fn write_chr(&mut self, _: u16, _: u8) {}
}
