use super::super::mapper::Mapper;

use Mirroring::*;

#[derive(Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen
}

#[derive(Debug)]
pub struct Cartridge {
    pub mapper: Mapper,
    pub mirroring: Mirroring, // None = Ignore mirroring
    prg_rom: Vec<u8>,
    chr_rom: Option<Vec<u8>>,
    //prg_ram: Option<Vec<u8>>,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>, // NOT SUITED LOGIC FOR NES 2.0 
}

impl Cartridge {
    //fn new(mapper: Mapper, ignore_mirroring: bool, mirroring: Mirroring, contains_ram: bool, prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>) -> Cartridge {
    fn new(mapper: Mapper, ignore_mirroring: bool, mirroring: Mirroring, prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>) -> Cartridge {
        Cartridge {
            mapper,
            mirroring: if ignore_mirroring { FourScreen } else { mirroring },
            prg_rom,
            chr_rom,
            //prg_ram: if contains_ram { Some(Vec::new()) } else { None },
            prg_ram: Vec::new(),
            chr_ram: Vec::new(),
        }
    }

    pub fn disassemble(bytes: &[u8]) -> Result<Cartridge, &'static str> {
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

            let prg_rom_size = bytes[4] as usize * 0x4000; // 16384
            let chr_rom_size = bytes[5] as usize * 0x2000; // 8192

            let mapper = match Mapper::get_mapper(((bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4) as u16) {
                Mapper::NotSupported => return Err("Mapper not suported."),
                m => m
            };

            //let contains_ram = if bytes[6] & 0x2 == 1 { true } else { false }; // 1 = yes (PGA_RAM) TODO
            // let prg_ram_size = bytes[8]; // TODO

            let prg_rom = bytes[16..16 + (prg_rom_size - 1)].to_vec();
            
            let mut chr_rom = None;
            if chr_rom_size != 0 {
                let offset = 16 + prg_rom_size;
                chr_rom = Some(bytes[offset..offset + (chr_rom_size - 1)].to_vec());
            }
            
            let mirroring = if bytes[6] & 0x1 == 0 { Horizontal } else { Vertical };


            //Ok(Cartridge::new(mapper, bytes[6] & 0x8 == 1, mirroring, contains_ram, prg_rom, chr_rom))
            Ok(Cartridge::new(mapper, bytes[6] & 0x8 == 1, mirroring, prg_rom, chr_rom))

        } else {
            Err("Only NES files supported.")
        }
    }

    // TODO: Use Mapper to read addresses. (Bank swuitching)
    pub fn read_prg(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.prg_ram[addr as usize]
        } else {
            self.prg_rom[addr as usize]
        }
    }

    pub fn read_chr(self, addr: u16) -> u8 {
        match self.chr_rom {
            Some(v) => v[addr as usize],
            _ => self.chr_ram[addr as usize]
        }
    }
}