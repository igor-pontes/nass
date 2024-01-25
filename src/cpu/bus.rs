use std::rc::Rc;
use std::cell::RefCell;
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
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>,
    pub ppu: Rc<RefCell<PPU>>,
}

impl BUS {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>, ppu: Rc<RefCell<PPU>>) -> Self {
        // log(&format!("BUS_PPU: {:p}", ppu.as_ptr()));
        BUS {
            ram: [0; RAM_SIZE],
            mapper,
            ppu,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF] = value,
            // 0x2000 => unsafe { log(&format!("[PPU CTRL]: {}", value)); (*self.ppu).write_to_ctrl(value) },
            0x2000 => self.ppu.borrow_mut().write_to_ctrl(value),
            // 0x2001 => unsafe { log(&format!("[PPU MASK]: {}", value)); (*self.ppu).write_to_ppu_mask(value) },
            0x2001 => self.ppu.borrow_mut().write_to_ppu_mask(value),
            0x2003 => self.ppu.borrow_mut().write_to_oam_addr(value),
            0x2004 => self.ppu.borrow_mut().write_to_oam(value),
            0x2005 => self.ppu.borrow_mut().write_to_scroll(value),
            0x2006 => self.ppu.borrow_mut().write_to_ppu_addr(value),
            0x2007 => self.ppu.borrow_mut().write_data(value),

            0x2008..=0x3FFF => self.write(addr & 0x2007, value),
            0x4014 => {
                let addr = (value as u16) << 8;
                self.ppu.borrow_mut().write_to_oam_addr(0);
                for i in 0..=0xFF {
                    let value = self.read(addr+i);
                    self.write(0x2004, value);
                }
            },
            0x4020..=0xFFFF => unsafe { (*self.mapper.as_ptr()).write_prg(addr, value); },
            _ => ()
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { 
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address.");
            }
            // 0x2002 => unsafe { log(&format!("BUS_PPU: {:p}", &*(self.ppu))); (*self.ppu).status() },
            0x2002 => self.ppu.borrow_mut().status(),
            0x2007 => self.ppu.borrow_mut().read_data(),
            0x2008..=0x3FFF => self.read(addr & 0x2007),
            0x4020..=0xFFFF => unsafe { (*self.mapper.as_ptr()).read_prg(addr)},
            _ => 0
        }
    }
}
