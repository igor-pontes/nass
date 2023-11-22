use {
    crate::{ cpu::*, mapper::*, ppu::*},
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// https://www.nesdev.org/wiki/Status_flags#I:_Interrupt_Disable
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Interrupt {
    NMI,
    IRQ,
    BRK,
    DISABLED
}

pub struct Emulator {
    cpu: CPU,
    ppu: PPU,
    cartridge: Cartridge,
    interrupt: Interrupt,
    frame: Vec<String>
}

impl Emulator {
    pub fn new(rom: &Vec<u8>) -> Self {
        let Ok(mut cartridge) = Cartridge::new(rom) else { panic!() };
        let ptr = &mut cartridge as *mut Cartridge;
        let mut ppu = PPU::new(ptr);
        Emulator { 
            cpu: CPU::new(BUS::new(ptr, &mut ppu as *mut PPU)),
            ppu,
            cartridge,
            interrupt: Interrupt::DISABLED,
            frame: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset()
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.interrupt);
        self.ppu.step(&mut self.interrupt);
        self.ppu.step(&mut self.interrupt);
        self.ppu.step(&mut self.interrupt);
    }
}
