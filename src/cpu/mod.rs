mod bus;
mod instructions;
mod cpu_status;

pub use self::bus::*;
use crate::mapper::new;
use crate::ppu::*;
use cpu_status::*;
use crate::cpu::instructions::*;

// CPU is guaranteed to receive NMI every interrupt
const CYCLES_PER_FRAME: usize = 29780;

pub struct CPU {
    a: u8, // Accumulator
    y: u8, // register y
    x: u8, // register x
    pc: u16, // Program counter
    s: u8, // Stack pointer (256-byte stack at $0100-$01FF.)
    status: CPUStatus,
    cycles_left: usize,
    cycles: usize,
    pub bus: BUS,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        let mapper = match new(rom) { 
            Ok(m) => m, 
            Err(str) => { panic!("{str}"); }
        };
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            status: CPUStatus::new(),
            // bus,
            bus: BUS::new(mapper, PPU::new()),
            cycles_left: 0,
            cycles: 0,
        }
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where 
        F: FnMut(&mut CPU),
    {
        for _ in 0..CYCLES_PER_FRAME {
            callback(self);
            {
                self.tick();
                self.bus.tick(self.cycles_left);
            }
        }
    }

    fn tick(&mut self) {
        self.cycles_left = 0;
        if let Some(Interrupt::Nmi) = self.bus.interrupt { 
            self.nmi();
            self.bus.interrupt = None;
        } else {
            let op = self.bus.read(self.pc);
            self.pc += 1;
            let (fun, addr_mode) = &CPU::OPCODES[op as usize];
            let addr = self.get_address_mode(addr_mode.clone()); 
            fun(self, addr);
            if self.bus.suspend {
                if self.cycles & 1 == 0 { 
                    self.cycles_left += 513; 
                } else { 
                    self.cycles_left += 514; 
                }
                self.bus.suspend = false;
            }
        }
        self.cycles += self.cycles_left;
    }

    pub fn reset(&mut self) {
        self.cycles_left = 7;
        self.x = 0;
        self.y = 0;
        self.a = 0;
        self.s = 0xFD;
        self.status = CPUStatus::new();
        self.pc = self.read_address(RESET_VECTOR);
    }

    pub fn get_frame_pointer(&self) -> *const u32 {
        self.bus.ppu.frame.get_pointer()
    }

    pub fn get_color(&self, index: usize) -> u32 {
        COLORS[self.bus.ppu.palette_table[index] as usize]
    }

    fn nmi(&mut self) {
        self.cycles_left = 7; 
        self.push_stack(((self.pc & 0xFF00) >> 8) as u8);
        self.push_stack((self.pc & 0x00FF) as u8);
        self.push_stack(self.status.bits() & !0x10);
        self.status.set_interrupt(true);
        self.pc = self.read_address(NMI_VECTOR);
    }

    fn get_address_mode(&mut self, addr_mode: AddrMode) -> u16 {
        match addr_mode {
            AddrMode::Rel(cycles) => { self.cycles_left += cycles & CYCLE_MASK; 0 },
            AddrMode::Acc(cycles) => { self.cycles_left += cycles & CYCLE_MASK; 0 },
            AddrMode::Impl(cycles) => { self.cycles_left += cycles & CYCLE_MASK; 0 },
            AddrMode::Imm(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let operand = self.pc;
                self.pc += 1;
                operand
            }
            AddrMode::Ind(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = (self.bus.read((addr & 0xFF00) | ((addr + 1) & 0x00FF)) as u16) * 0x100 | self.bus.read(addr) as u16;
                operand
            }
            AddrMode::Abs(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let operand = self.read_address(self.pc);
                self.pc += 2;
                operand
            }
            AddrMode::Zp(cycles) => { 
                self.cycles_left += cycles & CYCLE_MASK;
                let operand = self.bus.read(self.pc) as u16;
                self.pc += 1;
                operand
            }
            AddrMode::ZpX(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let operand = (self.bus.read(self.pc) + self.x) as u16; 
                self.pc += 1;
                operand
            }
            AddrMode::ZpY(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let operand = (self.bus.read(self.pc) + self.y) as u16;
                self.pc += 1;
                operand
            }
            AddrMode::AbsX(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = addr + self.x as u16;
                if (cycles as u8 & CYCLE_PAGE_CROSS_MASK) == 0 {
                    self.set_page_crossed(addr, operand);
                }
                operand
            }
            AddrMode::AbsY(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = addr + self.y as u16;
                if (cycles as u8 & CYCLE_PAGE_CROSS_MASK) == 0 {
                    self.set_page_crossed(addr, operand);
                }
                operand
            }
            AddrMode::IndX(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let arg = (self.bus.read(self.pc) + self.x) as u16;
                self.pc += 1;
                self.bus.read(arg & 0xFF) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100
            }
            AddrMode::IndrY(cycles) => {
                self.cycles_left += cycles & CYCLE_MASK;
                let arg = self.bus.read(self.pc) as u16;
                self.pc += 1;
                let addr = self.bus.read(arg) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100;
                let operand = addr + self.y as u16;
                if (cycles as u8 & CYCLE_PAGE_CROSS_MASK) == 0 {
                    self.set_page_crossed(addr, operand);
                }
                operand
            }
            AddrMode::None => 0
        }
    }

    fn set_page_crossed(&mut self, a: u16, b: u16) {
        if (a & 0xFF00) != (b & 0xFF00) { 
            self.cycles_left += 1; 
        }
    }

    fn push_stack(&mut self, val: u8) {
        self.bus.write(0x100 + self.s as u16, val);
        self.s -= 1;
    }
    
    fn pull_stack(&mut self) -> u8 {
        self.s += 1;
        let v = self.bus.read(0x100 + self.s as u16);
        v
    }

    fn read_address(&mut self, addr: u16) -> u16 {
        let addr = (self.bus.read(addr + 1) as u16) * 0x100 + self.bus.read(addr) as u16;
        addr
    }
}
