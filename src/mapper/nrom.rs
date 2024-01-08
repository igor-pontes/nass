use super::Mapper;
use wasm_bindgen::prelude::*;
pub use super::cartridge::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Debug)]
pub struct NROM {
    prg_ram: [u8; 0x1FFF],
    prg_rom: Vec<u8>, 
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self { 
        NROM {
            prg_ram: [0; 0x1FFF],
            prg_rom,
            chr_rom,
            mirroring,
        } 
    }
}

impl Mapper for NROM {
    fn get_mirroring(&self) -> &Mirroring { &self.mirroring }

    fn read_chr(&self, addr: u16) -> u8 { self.chr_rom[addr as usize] }

    fn read_prg(&self, addr: u16) -> u8 { 
        match addr {
            0x6000..=0x7FFF => { self.prg_ram[(addr - 0x6000) as usize] },
            0x8000..=0xFFFF => { 
                self.prg_rom[(addr - 0x8000) as usize] 
            },
            _ => { log("NROM: Trying to access 0x4020 - 0x6000."); panic!(); }
        }
    }
    fn write_prg(&mut self, addr: u16, val: u8) { 
        log("HELLO.");
        self.prg_ram[(addr - 0x6000) as usize] = val; 
    }

    fn write_chr(&mut self, addr: u16, val: u8) { log("Write to CHR RAM."); panic!() }
}
