use std::fmt;
use super::Mapper;
use wasm_bindgen::prelude::*;
pub use super::cartridge::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

pub struct NROM {
    prg_ram: [u8; 0x2000],
    chr_ram: [u8; 0x2000],
    prg_rom: Vec<u8>, 
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self { 
        NROM {
            prg_ram: [0; 0x2000],
            chr_ram: [0; 0x2000],
            prg_rom,
            chr_rom,
            mirroring,
        } 
    }
}

impl fmt::Display for NROM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NROM")
    }
}

impl Mapper for NROM {
    fn get_mirroring(&self) -> Mirroring { self.mirroring }

    fn read_chr(&self, addr: u16) -> u8 { 
        if self.chr_rom.len() == 0 {
            self.chr_ram[addr as usize] 
        } else {
            self.chr_rom[addr as usize] 
        }
    }

    fn read_prg(&self, addr: u16) -> u8 { 
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let mut addr = addr - 0x8000;
                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 { addr = addr % 0x4000; }
                self.prg_rom[addr as usize]
            },
            _ => { log(&format!("NROM: Trying to access 0x4020 - 0x6000. Address is {addr:#06x}.")); 0 }
            // _ => { log(&format!("NROM: Trying to access 0x4020 - 0x6000. Address is {addr:#06x}.")); panic!(); }
        }
    }
    fn write_prg(&mut self, addr: u16, val: u8) { 
        self.prg_ram[(addr - 0x6000) as usize] = val; 
    }

    fn write_chr(&mut self, addr: u16, val: u8) { 
        self.chr_ram[(addr) as usize] = val;
    }
}
