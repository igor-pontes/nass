use crate::{ cpu::*, mapper::*, ppu::COLORS };

pub struct Emulator {
    cpu: Option<CPU>,
    rom: Vec<u8>,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator { 
            cpu: None,
            rom: Vec::new(),
        }
    }

    pub fn disassemble(&mut self) {
        let mapper = match new(&self.rom) { 
            Ok(m) => m, 
            Err(str) => { panic!("{str}"); }
        };
        self.cpu = Some(CPU::new(self.rom.as_ptr(), mapper));
    }

    pub fn get_color(&self, index: usize) -> u32 {
        match self.cpu.as_ref() {
            Some(cpu) => COLORS[cpu.bus.ppu.palette_table[index] as usize],
            None => { panic!("Emulator not initialized."); }
        }
    }

    pub fn set_len(&mut self, value: usize) {
        unsafe { self.rom.set_len(value); }
    }

    pub fn get_rom_pointer(&self) -> *const u8 {
        self.rom.as_ptr()
    }

    pub fn set_button(&mut self, value: u8) {
        match self.cpu.as_mut() {
            Some(cpu) => cpu.bus.joypad.set_button(value),
            None => { panic!("Emulator not initialized."); }
        };
    }

    pub fn reset(&mut self) {
        match self.cpu.as_mut() {
            Some(cpu) => cpu.reset(),
            None => { panic!("Emulator not initialized."); }
        };
    }

    pub fn get_frame_pointer(&self) -> *const u32 {
        match self.cpu.as_ref() {
            Some(cpu) => cpu.bus.ppu.frame.get_pointer(),
            None => { panic!("Emulator not initialized."); }
        }
    }

    pub fn step(&mut self) { 
        match self.cpu.as_mut() {
            Some(cpu) => cpu.run(),
            None => { panic!("Emulator not initialized."); }
        }
    }
}
