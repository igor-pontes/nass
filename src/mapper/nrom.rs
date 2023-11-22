use super::Mapper;

#[derive(Debug)]
pub struct NROM();

impl NROM {
    pub fn new() -> Self { 
        NROM() 
    }
}

impl Mapper for NROM {
    fn read_prg(&self, prg_rom: &Vec<u8>, addr: u16) -> u8 { prg_rom[addr as usize] }
    fn read_chr(&self, chr_rom: &Vec<u8>, addr: u16) -> u8 { chr_rom[addr as usize] }
    fn write_prg(&mut self, prg_rom: &mut Vec<u8>, addr: u16, val: u8) { }
    fn write_chr(&mut self, chr_rom: &mut Vec<u8>, addr: u16, val: u8) { }
}
