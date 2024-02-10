mod cartridge;
mod nrom;
mod cnrom;
mod mmc1;

use crate::mapper::{
    nrom::NROM,
    cnrom::CNROM,
    mmc1::MMC1
};

pub use self::cartridge::*;
use std::fmt::Display;

pub type Mapper_ = Box<dyn Mapper>;

pub trait Mapper: Display {
    fn read_prg(&self, addr: u16) -> u8;
    fn read_chr(&self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, val: u8);
    fn write_chr(&mut self, addr: u16, val: u8);
    fn get_mirroring(&self) -> Mirroring;

    fn mirror(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        let mirroring = self.get_mirroring();
        // 00b - 1-screen mirroring (nametable 0)
        // 01b - 1-screen mirroring (nametable 1)
        match (mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            (Mirroring::OneScreenLower, 1) | (Mirroring::OneScreenLower, 2) | (Mirroring::OneScreenLower, 3) => vram_index & 0x23FF,
            (Mirroring::OneScreenUpper, 0) => vram_index + 0x400,
            (Mirroring::OneScreenUpper, 2) => vram_index - 0x400,
            (Mirroring::OneScreenUpper, 3) => vram_index - 0x800,
            _ => vram_index
        }
    }
}

pub fn get_mapper(mapper: u8, prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Result<Box<dyn Mapper>, String> {
    match mapper {

        0 => Ok(Box::new(NROM::new(prg_rom, chr_rom, mirroring))),
        1 => Ok(Box::new(MMC1::new(prg_rom, chr_rom, mirroring))),
        3 => Ok(Box::new(CNROM::new(prg_rom, chr_rom, mirroring))),
        _ => Err("Mapper not implemented.".to_string())
    }
}
