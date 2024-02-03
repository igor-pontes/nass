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

#[derive(Debug)]
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

impl Mapper for NROM {
    fn get_mirroring(&self) -> Mirroring { self.mirroring }

    fn read_chr(&self, addr: u16) -> u8 { 
        if addr <= 0x1fff  {
            // error(&format!("NROM: Trying to access 0x0000 - 0x1FFFF. Address is {addr:#06x}."));
            // panic!();
        }
        self.chr_rom[addr as usize] 
    }

    fn read_prg(&self, addr: u16) -> u8 { 
        match addr {
            0x6000..=0x7FFF => { self.prg_ram[(addr - 0x6000) as usize] },
            0x8000..=0xFFFF => { self.prg_rom[(addr - 0x8000) as usize] },
            _ => { log(&format!("NROM: Trying to access 0x4020 - 0x6000. Address is {addr:#06x}.")); 0 }
        }
    }
    fn write_prg(&mut self, addr: u16, val: u8) { 
        log(&format!("[NROM] write_prg | addr: {:#06x} | val: {:#06x}", addr, val));
        self.prg_ram[(addr - 0x6000) as usize] = val; 
    }

    fn write_chr(&mut self, addr: u16, val: u8) { 
        // log(&format!("[NROM] write_chr | addr: {:#06x} | val: {:#06x}", addr, val));
        self.chr_ram[(addr) as usize] = val;
    }
}
