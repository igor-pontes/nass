use std::fmt;
use super::Mapper;
pub use super::cartridge::*;

pub struct CNROM {
    prg_rom: Vec<u8>, 
    chr_rom: Vec<u8>,
    chr_bank: u16,
    mirroring: Mirroring,
}

impl CNROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self { 
        CNROM {
            prg_rom,
            chr_rom,
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

    fn read_chr(&self, addr: u16) -> u8 { 
        self.chr_rom[(addr + self.chr_bank) as usize] 
    }

    fn read_prg(&self, addr: u16) -> u8 { 
        match addr {
            0x8000..=0xFFFF => {
                let mut addr = addr - 0x8000;
                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 { 
                    addr = addr % 0x4000; 
                }
                self.prg_rom[addr as usize]
            },
            _ => 0
        }
    }

    fn write_prg(&mut self, addr: u16, val: u8) { 
        match addr {
            0x8000..=0xFFFF => self.chr_bank = ((val as u16) & 0x3) * 0x2000,
            _ => () 
        }
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) { }
}
