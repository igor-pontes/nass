use crate::{ cpu::*, mapper::*, ppu::*};

pub struct Emulator {
    pub cpu: CPU,
}

impl Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        let mapper = match new(rom) { 
            Ok(m) => m, 
            Err(str) => { panic!("{str}"); }
        };
        Emulator { 
            cpu: CPU::new(BUS::new(mapper, PPU::new())),
        }
    }
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn get_frame_pointer(&self) -> *const u32 {
        self.cpu.bus.ppu.frame.get_pointer()
    }

    pub fn get_color(&self, index: usize) -> u32 {
        COLORS[self.cpu.bus.ppu.palette_table[index] as usize]
    }
}
