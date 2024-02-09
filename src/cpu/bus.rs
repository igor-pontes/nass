use std::rc::Rc;
use std::cell::RefCell;
use crate::mapper::*;
use crate::ppu::PPU;

const RAM_SIZE: usize = 0x800;

pub struct BUS {
    ram: [u8; RAM_SIZE],
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>,
    pub ppu: Rc<RefCell<PPU>>,
    pub suspend: bool,
}

impl BUS {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>, ppu: Rc<RefCell<PPU>>) -> Self {
        BUS {
            ram: [0; RAM_SIZE],
            mapper,
            ppu,
            suspend: false
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr as usize) & 0x07FF] = value,
            0x2000 => self.ppu.borrow_mut().write_to_ctrl(value),
            0x2001 => self.ppu.borrow_mut().write_to_ppu_mask(value),
            0x2003 => self.ppu.borrow_mut().write_to_oam_addr(value),
            0x2004 => self.ppu.borrow_mut().write_to_oam(value),
            0x2005 => self.ppu.borrow_mut().write_to_scroll(value),
            0x2006 => self.ppu.borrow_mut().write_to_ppu_addr(value),
            0x2007 => self.ppu.borrow_mut().write_data(value),
            0x2008..=0x3FFF => self.write(addr & 0x2007, value),
            0x4014 => {
                self.suspend = true;
                let addr = ((value as u16) & 0xFF) << 8;
                for i in 0..=0xFF {
                    let value = self.read(addr+i);
                    self.write(0x2004, value);
                }
            },
            0x4020..=0xFFFF => self.mapper.borrow_mut().write_prg(addr, value),
            _ => ()
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { 
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => 0,
            0x2002 => self.ppu.borrow_mut().read_status(),
            0x2004 => self.ppu.borrow_mut().read_oam(),
            0x2007 => self.ppu.borrow_mut().read_data(),
            0x2008..=0x3FFF => self.read(addr & 0x2007),
            0x4020..=0xFFFF => self.mapper.borrow().read_prg(addr),
            _ => 0
        }
    }
}
