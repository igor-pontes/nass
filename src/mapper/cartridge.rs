use crate::mapper::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
    pub mapper: Box<dyn Mapper>,
    pub mirroring: Mirroring
}

impl Cartridge {
    pub fn new(bytes: &Vec<u8>) -> Result<Cartridge, String> {
        if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
            if bytes[7] & 0x12 == 2 { return Err("NES 2.0 not supported(yet).".to_string()) }

            let prg_rom_banks = bytes[4] as usize; // 16384
            let chr_rom_banks = bytes[5] as usize; // 8192

            let four_screen = bytes[6] & 0x8 != 0;
            let vertical_mirroring = bytes[6] & 0x1 != 0;
            let mirroring = match (four_screen, vertical_mirroring) {
                (true, _) => Mirroring::FourScreen,
                (false, true) => Mirroring::Vertical,
                (false, false) => Mirroring::Horizontal,
            };

            let skip_trainer = bytes[6] & 0b100 != 0;

            let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
            let chr_rom_start = prg_rom_start + prg_rom_banks * 0x4000;

            let prg_rom = bytes[prg_rom_start..16 + prg_rom_banks * 0x4000].to_vec();
            let chr_rom = bytes[chr_rom_start..chr_rom_start + chr_rom_banks * 0x2000].to_vec();

            let mapper = match get_mapper((bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4) {
                Ok(mapper) => mapper,
                Err(str) => return Err(str)
            };

            Ok(Cartridge {
                prg_rom,
                prg_ram: Vec::with_capacity(0),
                chr_rom,
                chr_ram: Vec::with_capacity(0),
                mapper,
                mirroring
            })

        } else {
            Err("Only NES files supported.".to_string())
        }
    }

    pub fn read_prg(&self, addr: u16) -> u8 {
        self.mapper.read_prg(&self.prg_rom, addr)
    }

    pub fn read_chr(&mut self, addr: u16) -> u8 {
        self.mapper.read_prg(&mut self.chr_rom, addr)
    }
}
