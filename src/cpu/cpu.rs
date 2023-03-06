use std::thread::LocalKey;

use super::{
    bus::BUS,
    instructions::{
        AddressingMode::*,
        *
    },
};
use int_enum::IntEnum;
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
pub enum Interrupt {
    NMI,
    IRQ,
    BRK,
    NULL
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
    i: Interrupt
}

impl CPU {

    // https://en.wikibooks.org/wiki/NES_Programming/Initializing_the_NES
    pub fn new(bus: BUS) -> CPU {
        use Interrupt::NULL;
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD, // = 0x01FD (descending stack)
            p: 0x34,
            bus,
            cycle: 0,
            i: NULL
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.read_address(RESET_VECTOR);
    }

    pub fn step(&mut self) {
        use Interrupt::*;
        // Interrupts can't overlap here, we are running sequencially.
        match self.i {
            NMI => {
                self.interrupt(NMI);
                self.i = NULL;
                return; 
            },
            IRQ => {
                self.interrupt(IRQ);
                self.i = NULL;
                return;
            },
            _ => ()
        }

        let op = self.bus.read(self.pc);
        self.pc += 1;

        let cycle_len = OP_CYCLES[op as usize];

        if self.execute_implied(op) || self.execute_relative(op) || self.operation1(op) || self.operation2(op) || self.operation0(op) {

        }

        // PPU
        for _ in 0..3 {
            let i = self.bus.ppu.step();
            self.interrupt(i);
        }
    }

    fn set_interrupt(&mut self, i: Interrupt) {
        self.i = i;
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
    
        if (0x04 & self.p) == 0x04 && i == IRQ {
            // IRQ can't execute if "I" flag isn't clear.
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

    fn execute_relative(&mut self, opcode: u8) -> bool { 
        /*  match case here would be wasteful...
            every relative operation does the same sequence of instructions... 

            h  - b
            10 - 00010000
            30 - 00110000
            50 - 01010000
            70 - 01110000
            90 - 10010000
            B0 - 10110000
            D0 - 11010000
            F0 - 11110000
            1F - 00011111 (Relative Branch Mask)
            20 - 00100000 (Flag 0 or 1,  )
        */ 

        let (c, z, d, o, n) = (0x1 & self.p, 0x2 & self.p, 0x8 & self.p, 0x40 & self.p , 0x80 & self.p);
        let mut opcode = opcode & 0x1F;
        if opcode == 0x10 {
            opcode &= 0x20;
            if (opcode >> 5) == c || (opcode >> 4) == z || (opcode >> 2) == d || (opcode << 1) == o || (opcode << 2) == n {
                let offset = self.bus.read(self.pc);
                let new_pc = self.pc + offset as u16;
                self.set_page_crossed(self.pc, new_pc, 2);
                self.pc = new_pc;
                self.cycle += 1; // skipcycle? why?
            } else {
                // branch condition not met
                self.pc += 1;
            }
            true
        } else {
            false
        }
    }

    fn set_page_crossed(&mut self, a: u16, b: u16, inc: u8) {
        // https://wiki.osdev.org/Paging
        // The 6502 CPU groups memory together into 256 byte pages. 
        // Where one page begins and the other ends is commonly referred to as a page boundary.
        // If offset is in the same page, no additional instructions needed.
        if a & 0xFF00 != b & 0xFF00 {
            // skip "inc" cycles
        }
    }

    fn operation1(&mut self, opcode: u8) -> bool {
        use Operation1::*;
        if opcode & OP_MASK == 1 {
            let mut value = 0;
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst_mode = match Operation1::from_int((opcode & INST_MODE_MASK) >> INST_MODE_SHIFT) {
                Ok(inst) => inst,
                _ => return false
            };
            let peek = |a| self.bus.read(a) as u16 ;

            match ADDR_1[addr_mode as usize]  {
                IndirectX => {
                    let addr = peek(self.pc) + self.x as u16;
                    // wrapping with 0xFF because sum can be more than 0xFF
                    value = peek(peek(addr & 0xFF) + peek((addr + 1) & 0xFF) * 0x100);
                    self.pc += 1;
                },
                Zeropage => {
                    // Data from the zero page (location on ram $0000-$00FF)
                    // https://www.nesdev.org/wiki/Sample_RAM_map 
                    value = peek(peek(self.pc)); // dont need to wrap with "0xFF" (no sum)
                    self.pc += 1;
                },
                Immediate => {
                    value = peek(self.pc);
                    self.pc += 1;
                },
                Absolute => {
                    value = self.read_address(self.pc);
                    self.pc += 2;
                },
                IndirectY => {
                    let addr = peek(self.pc) as u16;
                    value = peek(peek(addr) + peek((addr + 1) & 0xFF) * 0x100 + self.y as u16);
                    self.pc += 1;
                },
                ZeropageX => {
                    let addr = peek(self.pc) + self.x as u16;
                    value = peek(addr & 0xFF);
                    self.pc += 1;
                },
                AbsoluteY => {
                    let y = self.y as u16;
                    value = self.read_address(self.pc);
                    // STA do not increment 1 cycle if page crossed.
                    if inst_mode != STA {
                        self.set_page_crossed(value, value + y, 1);
                    }
                    value += y;
                    self.pc += 2;
                },
                AbsoluteX => {
                    let x = self.x as u16;
                    value = self.read_address(self.pc);
                    // STA do not increment 1 cycle if page crossed.
                    if inst_mode != STA {
                        self.set_page_crossed(value, value + x, 1);
                    }
                    value += x;
                    self.pc += 2;
                },
                _ => {}
            };
            match inst_mode {
                ORA => {
                    self.a |= value as u8;
                    self.set_zn(self.a);
                },
                AND => {
                    self.a &= value as u8;
                    self.set_zn(self.a);
                },
                EOR => {
                    self.a ^= value as u8;
                    self.set_zn(self.a);
                },
                ADC => {
                    let c = self.p & 0x1;
                    let sum = (self.a + c) as u16 + value;
                    //self.p &= ((sum & 0x100) >> 8) as u8;
                    self.set_c(sum);
                    // http://www.c-jump.com/CIS77/CPU/Overflow/lecture.html (Overflow Condition (signed))
                    // When two signed 2's complement numbers are added, overflow is detected if:
                    //    both operands are positive and the result is negative, or
                    //    both operands are negative and the result is positive.
                    //let v = ((self.a as u16 ^ sum) & (value ^ sum) & 0x80) as u8 >> 1;
                    //self.p |= v;
                    self.set_v(self.a as u16, value, sum);
                    self.a = sum as u8;
                    self.set_zn(self.a);
                },
                STA => {
                    self.bus.write(value, self.a);
                },
                LDA => {
                    self.a = value as u8;
                    self.set_zn(self.a);
                },
                CMP => {
                    let a = self.a as u16 | 0x8000; // little trick to simplify overflow calculation (sorry Rust)
                    let sum = a as u16 - value;
                    self.p &= (((sum & 0x100) >> 8) ^ 0x1) as u8;
                    self.set_zn(sum as u8);
                },
                SBC => {
                    // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
                    // Thus, adding the twos complement is the same as subtracting. (With the exception of the carry bit, 
                    // which is affected by the extra 256. 

                    // http://teaching.idallen.com/dat2343/10f/notes/040_overflow.txt
                    // if c set => borrow
                    let c = (self.p & 0x1 ^ 0x1) as u16; // NOT(c)
                    let a = self.a as u16 | 0x8000;
                    let sum = (a - value) - c;

                    // 256 - N = twos complement of N
                    // 255 - N = ones complement of N

                    // overflow condition for unsigned-integer arithmetic
                    // (twos-complement)
                    // 80 + (-48) = 32 (Correct answer. But this sets c = 1(0x100)...)
                    self.p &= (((sum & 0x100) >> 8) ^ 0x1) as u8;

                    self.p |= ((a as u16 ^ sum) & (!value ^ sum) & 0x80) as u8 >> 1;

                    self.a = sum as u8;
                    self.set_zn(self.a);
                },
            };
            true
        } else {
            false
        }
    }

    fn operation2(&mut self, opcode: u8) -> bool {
        false
    }

    fn operation0(&mut self, opcode: u8) -> bool {
        false
    }

    fn execute_implied(&mut self, opcode: u8) -> bool {
        use ImplicitOps::*;
        let implied = match ImplicitOps::from_int(opcode) {
            Ok(i) => i,
            _ => return false
        };

        match implied {
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

    fn set_v(&mut self, a: u16, b: u16, sum: u16) {
        self.p |= ((a as u16 ^ sum) & (b ^ sum) & 0x80) as u8 >> 1;
    }

    fn set_c(&mut self, sum: u16) {
        self.p &= ((sum & 0x100) >> 8) as u8;
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