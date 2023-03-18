use crate::cartridge::Mirroring;

use super::{super::Cartridge, Mapper};

pub struct NROM {
    pub mirroring: Mirroring,
    cartridge: Cartridge
}

impl NROM {
    pub fn new(cartridge: Cartridge, mirroring: Mirroring) -> Self {
        NROM { mirroring, cartridge }
    }
}

impl Mapper for NROM {

    fn read_prg(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.cartridge.read_prg_ram(addr - 0x6000)
        } else {
            self.cartridge.read_prg_rom(addr - 0x8000)
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        match self.cartridge.read_chr_rom(addr) {
            Some(v) => v,
            _ => self.cartridge.read_chr_ram(addr)
        }
    }

    fn write_prg(&mut self, addr: u16, val: u8) {
        self.cartridge.write_prg_ram(addr, val)
    }

    fn write_chr(&mut self, addr: u16, val: u8) {
        self.cartridge.write_chr_ram(addr, val)
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

}