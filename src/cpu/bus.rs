use crate::ppu::PPU;
use crate::mapper::*;
use Interrupt::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Interrupt {
    Nmi,
    Irq,
}

const RAM_SIZE: usize = 0x800;

pub struct BUS {
    ram: [u8; RAM_SIZE],
    mapper: Mapper_,
    pub ppu: PPU,
    pub suspend: bool,
    pub interrupt: Option<Interrupt>,
}

impl BUS {
    pub fn new(mapper: Mapper_, ppu: PPU) -> Self {
        BUS {
            ram: [0; RAM_SIZE],
            mapper,
            ppu,
            suspend: false,
            interrupt: None,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr as usize) & 0x07FF] = value,
            0x2000 => if self.ppu.write_to_ctrl(value) { self.interrupt = Some(Nmi) },
            0x2001 => self.ppu.mask.update(value),
            0x2003 => self.ppu.oam_addr = value,
            0x2004 => self.ppu.write_to_oam(value),
            0x2005 => self.ppu.write_to_scroll(value),
            0x2006 => self.ppu.write_to_ppu_addr(value),
            0x2007 => self.ppu.write_data(value, &mut self.mapper),
            0x2008..=0x3FFF => self.write(addr & 0x2007, value),
            0x4014 => {
                self.suspend = true;
                let addr = ((value as u16) & 0xFF) << 8;
                for i in 0..=0xFF {
                    let value = self.read(addr+i);
                    self.write(0x2004, value);
                }
            },
            0x4020..=0xFFFF => self.mapper.write_prg(addr, value),
            _ => ()
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { 
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => 0,
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam(),
            0x2007 => self.ppu.read_data(&self.mapper),
            0x2008..=0x3FFF => self.read(addr & 0x2007),
            0x4020..=0xFFFF => self.mapper.read_prg(addr),
            _ => 0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..(3) {
            if self.ppu.tick(&mut self.mapper) { 
                self.interrupt = Some(Nmi); 
            } 
        }
    }
}
