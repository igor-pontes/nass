mod nrom;
mod cnrom;
mod mmc1;

pub use crate::mapper::{
    nrom::NROM,
    cnrom::CNROM,
    mmc1::MMC1
};

use std::fmt::Display;

#[derive(PartialEq, Clone, Copy)]
pub enum Mirroring {
    OneScreenUpper,
    OneScreenLower,
    Vertical,
    Horizontal,
    FourScreen
}

pub type Mapper_ = Box<dyn Mapper>;

pub trait Mapper: Display {
    fn read_prg(&self, rom: *const u8, addr: u16) -> u8;
    fn read_chr(&self, rom: *const u8, addr: u16) -> u8;
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

pub fn get_mapper(prg_len: usize, chr_len: usize, prg_offset: usize, chr_offset: usize, mapper: u8, mirroring: Mirroring) -> Result<Box<dyn Mapper>, String> { 
    match mapper {
        0 => Ok(Box::new(NROM::new(prg_len, chr_len, prg_offset, chr_offset, mirroring))),
        1 => Ok(Box::new(MMC1::new(prg_len, chr_len, prg_offset, chr_offset, mirroring))),
        3 => Ok(Box::new(CNROM::new(prg_len, chr_len, prg_offset, chr_offset, mirroring))),
        _ => Err("Mapper not implemented.".to_string())
    }
}

pub fn new(bytes: &Vec<u8>) -> Result<Mapper_, String> {
    if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
        if bytes[7] & 0x12 == 2 { return Err("NES 2.0 not supported(yet).".to_string()) }

        let prg_rom_banks = bytes[4] as usize; // 16384
        // Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
        let chr_rom_banks = bytes[5] as usize; // 8192

        let four_screen = bytes[6] & 0x8 != 0;
        let vertical_mirroring = bytes[6] & 0x1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let skip_trainer = bytes[6] & 0x04 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_banks * 0x4000;

        let mapper_id = (bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4;

        let mapper = match get_mapper(prg_rom_banks * 0x4000, chr_rom_banks * 0x2000, prg_rom_start, chr_rom_start, mapper_id, mirroring) {
            Ok(mapper) => mapper,
            Err(str) => return Err(str)
        };

        Ok(mapper)
    } else {
        Err("Only NES files supported.".to_string())
    }
}
