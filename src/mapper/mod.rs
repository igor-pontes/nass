mod cartridge;
mod nrom;
use crate::mapper::nrom::NROM;
pub use self::cartridge::*;
use std::fmt::Debug;

pub trait Mapper: Debug {
    fn read_prg(&self, prg_rom: &Vec<u8>, addr: u16) -> u8;
    fn read_chr(&self, chr_rom: &Vec<u8>, addr: u16) -> u8;
    fn write_prg(&mut self, prg_rom: &mut Vec<u8>, addr: u16, val: u8);
    fn write_chr(&mut self, chr_rom: &mut Vec<u8>, addr: u16, val: u8);
}

pub fn get_mapper(mapper: u8) -> Result<Box<dyn Mapper>, String> {
    match mapper {
        0 => Ok(Box::new(NROM::new())),
        _ => Err("Mapper not implemented.".to_string())
    }
}
