use { 
    std::cell::RefCell,
    std::rc::Rc,
    crate::{ cpu::*, mapper::*, ppu::* },
};

const CYCLES_PER_FRAME: usize = 29780 * 4;

// https://www.nesdev.org/wiki/Status_flags#I:_Interrupt_Disable
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Interrupt {
    NMI,
    IRQ,
    DISABLED
}

pub struct Emulator {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
    interrupt: Interrupt,
}

impl Emulator {
    pub fn new(rom: &Vec<u8>) -> Self {
        let mapper = match new(rom) { 
            Ok(m) => Rc::new(RefCell::new(m)), 
            Err(str) => { panic!("{str}"); }
        };
        let ppu = Rc::new(RefCell::new(PPU::new(mapper.clone())));
        Emulator { 
            cpu: CPU::new(BUS::new(mapper.clone(), ppu.clone())),
            ppu,
            interrupt: Interrupt::DISABLED,
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
        let mut cycles = 0; 
        while cycles < CYCLES_PER_FRAME {
            if self.cpu.bus.suspend { 
                if self.cpu.odd_cycle { self.cpu.cycle += 513; } else { self.cpu.cycle += 514; }
                self.cpu.bus.suspend = false;
            }
            self.cpu.step(&mut self.interrupt);
            for _ in 0..3 {
                self.ppu.borrow_mut().step();
                if self.ppu.borrow().nmi_ocurred { 
                    self.interrupt = Interrupt::NMI;
                    self.ppu.borrow_mut().nmi_ocurred = false;
                    break 
                }
            } 
            cycles += 1;
        }
    }
}
