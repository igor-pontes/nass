use crate::cartridge::Cartridge;
use super::nrom::NROM;

pub trait Mapper {
    fn read_prg(&self, addr: u16) -> u8;
    fn read_chr(&self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, val: u8);
    fn write_chr(&mut self, addr: u16, val: u8);
}

pub fn crate_mapper(mapper: u8, cartridge: Cartridge, prg_banks: usize, chr_banks: usize) -> Result<Box<dyn Mapper>, &'static str> {
    match mapper {
        0 => Ok(Box::new(NROM::new(cartridge))),
        _ => Err("Mapper not implemented.")
    }
}
