use super::super::cpu::CPU;
use super::super::mapper::Mapper;

use Mirroring::*;
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen
}

pub struct Cartridge {
    pub mapper: Mapper,
    pub mirroring: Mirroring, // None = Ignore mirroring
    pub prg_rom: Vec<u8>,
    pub chr_rom: Option<Vec<u8>>,
    pub prg_ram: Option<Vec<u8>>,
    pub chr_ram: Option<Vec<u8>>,
}

impl Cartridge {
    fn new(mapper: u16, ignore_mirroring: bool, mirroring: Mirroring, contains_ram: bool, prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>, chr_ram: Option<Vec<u8>>) -> Cartridge {
        Cartridge {
            mapper: Mapper::get_mapper(mapper),
            mirroring: if ignore_mirroring { FourScreen } else { mirroring },
            prg_rom,
            chr_rom,
            chr_ram,
            prg_ram: if contains_ram { Some(Vec::new()) } else { None },
        }
    }
    pub fn disassemble(bytes: &[u8]) -> Result<(), &'static str> {
        let cpu = CPU::new();
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

            let prg_rom_size = bytes[4] as usize * 16384;
            let chr_rom_size = bytes[5] as usize * 8192;
            let mapper = (bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4; // https://www.nesdev.org/wiki/Mapper

            let contains_ram = if bytes[6] & 0x2 == 1 { true } else { false }; // 1 = yes (PGA_RAM) TODO
            let prg_ram_size = bytes[8]; // TODO

            let prg_rom = bytes[16..16 + (prg_rom_size - 1)].to_vec();
            
            let mut chr_rom = None;
            if chr_rom_size != 0 {
                let offset = 16 + prg_rom_size;
                chr_rom = Some(bytes[offset..offset + (chr_rom_size - 1)].to_vec());
            }
            
            let mirroring = if bytes[6] & 0x1 == 0 { Horizontal } else { Vertical };

            let chr_ram = if chr_rom.is_none() { Some(Vec::new()) } else { None };

            let cartridge = Cartridge::new(mapper as u16, bytes[6] & 0x8 == 1, mirroring, contains_ram, prg_rom, chr_rom, chr_ram);
            
            Ok(())

        } else {
            Err("Only NES files supported.")
        }
    }
}