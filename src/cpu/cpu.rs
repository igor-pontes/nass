use super::{
    bus::BUS,
    instructions::{
        AddressingMode::*,
        *
    },
};
use { int_enum::IntEnum, ImplicitOps::*};
// https://en.wikipedia.org/wiki/MOS_Technology_6502

pub const CYCLES_PER_FRAME: usize = 1786830/60; // (Cycles / seconds) / (Frames / seconds) = (Cycles / Frames) 1786830/60 = 
// (1/1786830) * 10^9 =~ 560 ns per cycle (60 fps)
// Every cycle on 6502 is either a read or a write cycle.

// The PPU renders 262 scanlines per frame. 
// Each scanline lasts for 341 PPU clock cycles (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles), 
// with each clock cycle producing one pixel.
// https://www.nesdev.org/wiki/PPU_rendering

// MOS 6502 implementation 
// It is common practice on a 6502 to initialize the stack pointer to $FF

//  The P register can be read by pushing it on the stack (with PHP or
//  by causing an interrupt). https://www.nesdev.org/6502_cpu.txt

const NMI_VECTOR: u16 = 0xFFFA; 
const RESET_VECTOR: u16 = 0xFFFC; // INIT CODE
const IRQ_VECTOR: u16 = 0xFFFE; // IRQ OR BRK

#[derive(PartialEq)]
enum Interrupt {
    NMI,
    IRQ,
    BRK,
}

pub struct CPU {
    a: u8, // Accumulator (general purpose?)
    x: u8, // general purpose register x?
    y: u8, // general purpose register y?
    pub pc: u16, // Program counter
    s: u8, // Stack pointer (It indexes into a 256-byte stack at $0100-$01FF.)
    p: u8, // Status Register
    bus: BUS,
    pub cycle: usize,
}

impl CPU {
    // https://en.wikibooks.org/wiki/NES_Programming/Initializing_the_NES
    pub fn new(bus: BUS) -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD, // = 0x01FD (descending stack)
            p: 0x34,
            bus,
            cycle: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.read_address(RESET_VECTOR);
    }

    pub fn step(&mut self) {
        ()
    }

    fn interrupt(&mut self, i: Interrupt) {
        use Interrupt::*;
        //  This flag(0x10) is used to distinguish software (BRK) interrupts from hardware interrupts (IRQ or NMI).
        //  The B flag is always set except when the P register is being
        //  pushed on stack when jumping to an interrupt routine to
        //  process only a hardware interrupt.

        // IRQ will be executed only when the I flag is clear.
        // IRQ and BRK both set the I flag(0x04), whereas the NMI does not affect its state.

        // instructions (PHP and BRK) push a value with bit 4 set to 1. 
        // 0x04 = I flag
    
        if (0x04 & self.p) == 0x04 && i != NMI && i != BRK {
            // IRQ can't execute if I flag isn't clear.
            return;
        }

        // https://www.nesdev.org/6502_cpu.txt
        if i == BRK {
            // skip only 1 instruction 'cause we already processed the instruction BRK (2 bytes)
            self.pc += 1;
        }

        self.push_stack((self.pc >> 8) as u8);
        self.push_stack(self.pc as u8);
        
        let f = self.p | 0x20 | if i == BRK { 0x10 } else { 0x0 };
        self.push_stack(f);

        // "Side effects after pushing" - https://www.nesdev.org/wiki/Status_flags#The_B_flag
        self.p = f | if i == BRK { 0x04 } else { 0x0 };
        
        self.cycle += 6;
    }

    fn execute_implied(&mut self, opcode: u8) -> bool {
        // unwrap not good... temporary.
        match ImplicitOps::from_int(opcode).unwrap() {
            BRK => {
                self.interrupt(Interrupt::BRK);
            },
            RTI => {
                self.p = self.pull_stack() & 0xCF;
                self.pc = self.pull_stack() as u16 | self.pull_stack() as u16 * 0x100;
            },
            RTS => {
                self.pc = self.pull_stack() as u16 | self.pull_stack() as u16 * 0x100;
                self.pc += 1;
            },
            PHP => {
                self.push_stack(self.p | 0x30);
            },
            CLC => {
                self.p &= 0xFE;
            },
            PLP => {
                // TODO "ignore?"
                self.p = self.pull_stack();
            },
            SEC => {
                self.p |= 0x01;
            },
            PHA => {
                self.push_stack(self.a);
            },
            CLI => {
                self.p &= 0xFB;
            },
            PLA => {
                self.a = self.pull_stack();
                self.set_zn(self.a);
            },
            SEI => {
                self.p |= 0x04;
            },
            DEY => {
                self.y -= 1;
                self.set_zn(self.y);
            },
            TYA => {
                self.a = self.y;
                self.set_zn(self.a);
            },
            TAY => {
                self.y = self.a;
                self.set_zn(self.y);
            },
            CLV => {
                self.p &= 0x40;
            },
            INY => {
                self.y += 1;
                self.set_zn(self.y)
            },
            CLD => {
                self.p &= 0xF7;
            },
            INX => {
                self.x += 1;
                self.set_zn(self.x);
            },
            SED => {
                // Carry = 0x01 / Zero 0x02 / I = 0x04 / Dec = 0x08 / B = 0x10 / 1 = 0x20 / Over = 0x40 / Neg = 0x80
                self.p |= 0x08;
            },
            _ => return false
        }
        return true
    }

    fn set_zn(&mut self, val: u8) {
        if val == 0 {
            self.p |= 0x02;
        }
        if val & 0x80 == 0x80 {
            self.p |= 0x80;
        }
    }

    fn push_stack(&mut self, val: u8) {
        self.bus.write(0x100 + self.s as u16, val);
        self.s -= 1;
    }
    
    fn pull_stack(&mut self) -> u8 {
        let v = self.bus.read(0x100 + self.s as u16);
        self.s += 1;
        v
    }

    fn read_address(&self, addr: u16) -> u16 {
        (self.bus.read(addr + 1) as u16) * 0x100 + self.bus.read(addr) as u16
    }
    
}