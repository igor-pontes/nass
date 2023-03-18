use base64::{Engine as _, engine::general_purpose}; // "Engine" as _ ??
use crate::mapper::Mapper;

use super::super::mapper;

use Mirroring::*;

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen
}

#[derive(Debug)]
pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Option<Vec<u8>>,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
}

impl Cartridge {
    fn new(prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>) -> Cartridge {
        Cartridge {
            prg_rom,
            chr_rom,
            prg_ram: Vec::new(),
            chr_ram: Vec::new(),
        }
    }

    pub fn disassemble(file: String) -> Result<Box<dyn Mapper>, &'static str> {
        // TODO: CHR_RAM not implemented correctly
        let mut bytes = Vec::<u8>::new();
        general_purpose::STANDARD.decode_vec(file, &mut bytes).unwrap();

        if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
            if bytes[7] & 0x12 == 2 {
                return Err("NES 2.0 not supported(yet).")
            }
            if bytes[6] & 0x4 == 1 {
                return Err("Trainer not supported currently.")
            }
            if bytes[7] & 0x3 != 0 {
                return Err("Console type not supported.")
            }

            let prg_rom_banks = bytes[4] as usize; // 16384
            let chr_rom_banks = bytes[5] as usize; // 8192

            //let contains_ram = if bytes[6] & 0x2 == 1 { true } else { false }; // 1 = yes (PGA_RAM) TODO
            // let prg_ram_size = bytes[8]; // TODO
            
            let prg_rom = bytes[16..16 + prg_rom_banks * 0x4000].to_vec();
            
            let mut chr_rom = None;
            if chr_rom_banks != 0 {
                let offset = 16 + prg_rom_banks * 0x4000;
                chr_rom = Some(bytes[offset..offset + chr_rom_banks * 0x2000].to_vec());
            }
            

            let c = Cartridge::new(prg_rom, chr_rom);

            let m = if bytes[6] & 8 == 8 { 
                FourScreen 
            } else { 
                if bytes[6] & 0x1 == 0 { Horizontal } else { Vertical }
            };

            mapper::create_mapper((bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4, m, c, prg_rom_banks, chr_rom_banks)

        } else {
            Err("Only NES files supported.")
        }
    }
    
    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        self.prg_rom[addr as usize]
    }
    
    pub fn read_chr_rom(&self, addr: u16) -> Option<u8> {
        match &self.chr_rom {
            Some(v) => Some(v[addr as usize]),
            _ => None
        }
    }
    
    pub fn read_prg_ram(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize]
    }

    pub fn read_chr_ram(&self, addr: u16) -> u8 {
        self.chr_ram[addr as usize]
    }

    pub fn write_prg_ram(&mut self, addr: u16, val: u8) {
        self.prg_ram[addr as usize] = val
    }

    pub fn write_chr_ram(&mut self, addr: u16, val: u8) {
        self.chr_ram[addr as usize] = val
    }

}