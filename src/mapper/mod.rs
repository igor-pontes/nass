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

pub trait Mapper: Display {
    fn read_prg(&self, addr: u16) -> u8;
    fn read_chr(&self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, val: u8);
    fn write_chr(&mut self, addr: u16, val: u8);
    fn get_mirroring(&self) -> Mirroring;
}

pub fn get_mapper(mapper: u8, prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Result<Box<dyn Mapper>, String> {
    match mapper {
        0 => Ok(Box::new(NROM::new(prg_rom, chr_rom, mirroring))),
        1 => Ok(Box::new(MMC1::new(prg_rom, chr_rom, mirroring))),
        3 => Ok(Box::new(CNROM::new(prg_rom, chr_rom, mirroring))),
        _ => Err("Mapper not implemented.".to_string())
    }
}
