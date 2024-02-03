use { 
    std::cell::RefCell,
    std::rc::Rc,
    crate::{ cpu::*, mapper::*, ppu::* },
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)] 
    fn log(s: &str); 
}

// CPU cyles per frame
const CYCLES_PER_FRAME: usize = 29780;

// https://www.nesdev.org/wiki/Status_flags#I:_Interrupt_Disable
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Interrupt {
    NMI,
    IRQ,
    BRK,
    DISABLED
}

pub struct Emulator {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
    interrupt: Interrupt,
    cycles: usize,
    debug_cycles: usize,
}

impl Emulator {
    pub fn new(rom: &Vec<u8>) -> Self {
        let mapper = match new(rom) { 
            Ok(m) => Rc::new(RefCell::new(m)), 
            Err(str) => { log(&str); panic!(); }
        };
        let ppu = Rc::new(RefCell::new(PPU::new(mapper.clone())));
        Emulator { 
            cpu: CPU::new(BUS::new(mapper.clone(), ppu.clone())),
            ppu,
            interrupt: Interrupt::DISABLED,
            cycles: 0,
            debug_cycles: 1,
        }
    }
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn get_frame_pointer(&self) -> *const u8 {
        self.ppu.borrow().frame.get_pointer()
    }

    pub fn get_palette_pointer(&self) -> *const u8 {
        self.ppu.borrow().palette_table.as_ptr()
    }

    pub fn step(&mut self) {
        while self.cycles <= CYCLES_PER_FRAME {
            if self.debug_cycles == 0 {
                // log("END OF DEBUG.");
                // panic!();
            }

            if self.cpu.bus.suspend { 
                if self.cpu.odd_cycle {
                    self.cpu.cycle += 513;
                } else {
                    self.cpu.cycle += 514;
                }
                self.cpu.bus.suspend = false;
            }
            if self.ppu.borrow().nmi_ocurred {
                self.interrupt = Interrupt::NMI;
                self.ppu.borrow_mut().nmi_ocurred = false;
            }
            self.cpu.step(&mut self.interrupt);
            self.ppu.borrow_mut().step(); 
            self.ppu.borrow_mut().step(); 
            self.ppu.borrow_mut().step(); 

            if self.cycles == CYCLES_PER_FRAME { 
                self.cycles = 0; 
                self.debug_cycles -= 1;
                break; 
            }
            self.cycles += 1;
        }
    }
}
