use crate::mapper::*;
use crate::ppu::PPU;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const RAM_SIZE: usize = 0x800;

pub struct BUS {
    ram: [u8; RAM_SIZE],
    cartridge: *mut Cartridge,
    ppu: *mut PPU
}

impl BUS {
    pub fn new(cartridge: *mut Cartridge, ppu: *mut PPU) -> Self {
        BUS {
            ram: [0; RAM_SIZE],
            cartridge: cartridge as *mut Cartridge,
            ppu,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF] = value,
            0x2000 => unsafe { (*self.ppu).write_to_ctrl(value) },
            0x2001 => unsafe { log(&format!("mask: {}", value)); (*self.ppu).write_to_ppu_mask(value) },
            0x2003 => unsafe { (*self.ppu).write_to_oam_addr(value) },
            0x2004 => unsafe { (*self.ppu).write_to_oam(value) },
            0x2005 => unsafe { (*self.ppu).write_to_scroll(value) },
            0x2006 => unsafe { (*self.ppu).write_to_ppu_addr(value) },
            0x2007 => unsafe { (*self.ppu).write_to_data(value) },
            0x4014 => {
                let addr = (value as u16) << 8;
                unsafe {(*self.ppu).write_to_oam_addr(0)};
                for i in 0..=0xFF {
                    let value = self.read(addr+i);
                    self.write(0x2004, value);
                }
            },
            0x2008..=0x3FFF => self.write(addr & 0x2007, value),
            _ => ()
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { 
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address.");
            }
            0x2002 => unsafe { (*self.ppu).status() },
            0x2007 => unsafe { (*self.ppu).read_data() },
            0x2008..=0x3FFF => self.read(addr & 0x2007),
            0x8000..=0xFFFF => unsafe { (*self.cartridge).read_prg(addr - 0x8000)},
            _ => 0
        }
    }
}
