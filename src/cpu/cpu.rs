use super::{
    bus::BUS,
    instructions::{
        AddressingMode::*,
        *
    },
};
use int_enum::IntEnum;
use wasm_bindgen::prelude::*;
// https://en.wikipedia.org/wiki/MOS_Technology_6502

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


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

#[derive(PartialEq, Clone, Copy)]
pub enum Interrupt {
    NMI,
    IRQ,
    BRK,
    NULL
}

pub struct CPU<'a> {
    a: u8, // Accumulator (general purpose?)
    x: u8, // general purpose register x?
    y: u8, // general purpose register y?
    pub pc: u16, // Program counter
    s: u8, // Stack pointer (It indexes into a 256-byte stack at $0100-$01FF.)
    p: u8, // Status Register
    bus: BUS<'a>,
    pub cycle: usize,
    i: Interrupt,
    skip_cycles: u32,
}

impl<'a> CPU<'a> {

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
            i: NULL,
            skip_cycles: 0
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.read_address(RESET_VECTOR);
    }

    pub fn step(&mut self) {
        use Interrupt::*;
        for _ in 0..3 {
            let i = self.bus.ppu.step(); // PPU steps
            self.interrupt(i);
        }
        //log("cycles");
        //self.cycle += 1;
        if self.skip_cycles > 1 {
            self.skip_cycles -= 1;
            return
        }
          
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
            self.skip_cycles += cycle_len;
        }
        
    }

    // TODO
    fn skip_dma_cycles(&mut self) {
        self.skip_cycles += 513; // OAM DMA on its own takes 513 or 514 cycles, depending on whether alignment is needed. 
        self.skip_cycles += (self.cycle as u32) & 1; // alignment
    }

    fn interrupt(&mut self, i: Interrupt) {
        use Interrupt::*;
        
        if i == NULL { return; }
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
        
        let f = self.p & 0xEF | 0x20 | if i == BRK { 0x10 } else { 0x0 };
        self.push_stack(f);
        
        // "Side effects after pushing" - https://www.nesdev.org/wiki/Status_flags#The_B_flag
        self.p = (f & 0x04) | if i == BRK { 0x04 } else { 0x0 };
        
        // Writes to $4014 or $2004 should usually be done in an NMI routine, or otherwise within vertical blanking.
        
        self.pc = match i {
            BRK => { self.read_address(IRQ_VECTOR) },
            NMI => { self.read_address(NMI_VECTOR) },
            _ => { self.pc }
        };

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

        // check this.
        let status = [0x1 & self.p, (0x2 & self.p) >> 1, (0x8 & self.p) >> 3, (0x40 & self.p) >> 6 , (0x80 & self.p) >> 7];
        if (opcode & 0x1F) == 0x10 {
            let opcode = (opcode & 0x20) >> 5;
            if status.iter().any(|s| s == &opcode) {
                let offset = self.bus.read(self.pc);
                self.skip_cycles += 1;
                let new_pc = self.pc + offset as u16;
                self.set_page_crossed(self.pc, new_pc, 2);
                self.pc = new_pc;
            } else {
                // branch condition not met
                self.pc += 1;
            }
            true
        } else {
            false
        }
    }

    fn set_page_crossed(&mut self, a: u16, b: u16, inc: u32) {
        // https://wiki.osdev.org/Paging
        // The 6502 CPU groups memory together into 256 byte pages. 
        // Where one page begins and the other ends is commonly referred to as a page boundary.
        // If offset is in the same page, no additional instructions needed.
        if a & 0xFF00 != b & 0xFF00 {
            self.skip_cycles += inc;
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
            let mut peek = |a| self.bus.read(a) as u16 ;

            match ADDR_1[addr_mode as usize]  {
                IndirectX => {
                    let addr = peek(self.pc) + self.x as u16;
                    // wrapping with 0xFF because sum can be more than 0xFF
                    value = peek(addr & 0xFF) + peek((addr + 1) & 0xFF) * 0x100;
                    value = peek(value);
                    self.pc += 1;
                },
                Zeropage => {
                    // Data from the zero page (location on ram $0000-$00FF)
                    // https://www.nesdev.org/wiki/Sample_RAM_map 
                    value = peek(self.pc);
                    value = peek(value); // dont need to wrap with "0xFF" (no sum)
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
                    value = peek(addr) + peek((addr + 1) & 0xFF) * 0x100 + self.y as u16;
                    value = peek(value);
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
                    // why can't I use "peek(value)" here?
                    // https://doc.rust-lang.org/error_codes/E0502.html 
                    // because we have 2 match commands?
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
                    //self.set_c(sum);
                    self.set_c(((sum & 0x100) >> 8) as u8);
                    // http://www.c-jump.com/CIS77/CPU/Overflow/lecture.html (Overflow Condition (signed))
                    // When two signed 2's complement numbers are added, overflow is detected if:
                    //    both operands are positive and the result is negative, or
                    //    both operands are negative and the result is positive.
                    //let v = ((self.a as u16 ^ sum) & (value ^ sum) & 0x80) as u8 >> 1;
                    //self.p |= v;
                    //self.set_v(self.a as u16, value, sum);
                    self.set_v(((self.a as u16 ^ sum) & (value ^ sum) & 0x80) as u8 >> 1);
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
                    //self.p &= (((sum & 0x100) >> 8) ^ 0x1) as u8;
                    self.set_c((((sum & 0x100) >> 8) ^ 1) as u8);
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
                    //self.p &= (((sum & 0x100) >> 8) ^ 0x1) as u8;
                    self.set_c((((sum & 0x100) >> 8) ^ 1) as u8);

                    //self.p |= ((a as u16 ^ sum) & (!value ^ sum) & 0x80) as u8 >> 1;
                    self.set_v(((a as u16 ^ sum) & (!value ^ sum) & 0x80) as u8 >> 1);

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
        use Operation2::*;
        if opcode & OP_MASK == 1 {
            let mut value = 0;
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst_mode = match Operation2::from_int((opcode & INST_MODE_MASK) >> INST_MODE_SHIFT) {
                Ok(inst) => inst,
                _ => return false
            };

            match ADDR_2[addr_mode as usize]  {
                Immediate => {
                    value = self.pc;
                    self.pc += 1;
                },
                Zeropage => {
                    value = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                },
                Accumulator => {},
                Absolute => {
                    value = self.read_address(self.pc);
                    self.pc += 2;
                },
                ZeropageIndexed => {
                    value = self.bus.read(self.pc) as u16;
                    let mut index = 0;
                    if inst_mode == LDX || inst_mode == STX { 
                        index = self.y as u16;
                    } else {
                        index = self.x as u16;
                    };
                    value = (value + index) & 0xFF; // zero-page memory page
                },
                AbsoluteIndexed => {
                    value = self.read_address(self.pc);
                    self.pc += 2;
                    let mut index = 0;
                    if inst_mode == LDX || inst_mode == STX { 
                        index = self.y as u16;
                    } else {
                        index = self.x as u16;
                    };
                    self.set_page_crossed(value, value + index, 1);
                    value += index; // zero-page memory page
                },
                _ => { return false }
            }
            // can't declare "operand" here... "borrow" issue.
            // value here = address
            match inst_mode {
                ASL => {},               
                ROL => {
                    if ADDR_2[addr_mode as usize] == Accumulator {
                        let prev_c = self.p & 0x1; // prev_c = c
                        //self.p |= (self.a & 0x80) >> 7; // c
                        self.set_c((self.a & 0x80) >> 7);
                        self.a <<= 1;
                        self.a = self.a | prev_c;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let prev_c = self.p & 0x1; // prev_c = c
                        //self.p |= (operand & 0x80) >> 7; // c
                        self.set_c((operand & 0x80) >> 7); // c
                        operand = operand << 1 | prev_c;
                        self.set_zn(operand);
                        self.bus.write(value, operand);
                    }
                },
                LSR => {},
                ROR => {
                    if ADDR_2[addr_mode as usize] == Accumulator {
                        let prev_c = self.p & 1; // prev_c = c
                        //self.p |= self.a & 1; // c
                        self.set_c(self.a & 1);
                        self.a >>= 1;
                        self.a = self.a | prev_c << 7;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let prev_c = self.p & 1; // prev_c = c 
                        //self.p |= operand & 1; // c
                        self.set_c(operand & 1); // c
                        operand = operand >> 1 | prev_c << 7;
                        self.set_zn(operand);
                    }
                },
                STX => {
                    self.bus.write(value, self.x);
                },
                LDX => {
                    self.x = self.bus.read(value);
                    self.set_zn(self.x);
                },
                DEC => {
                    let mut peek = |a| { self.bus.read(a) as u16 };
                    let operand = ((peek(value) | 0x8000) - 1) as u8;
                    self.set_zn(operand);
                    self.bus.write(value, operand);
                },
                INC => {
                    let mut peek = |a| { self.bus.read(a) as u16 };
                    let operand = (peek(value) + 1) as u8;
                    self.set_zn(operand);
                    self.bus.write(value, operand);
                },
            }
            true
        } else {
            false
        }
    }

    fn operation0(&mut self, opcode: u8) -> bool {
        use Operation0::*;
        if opcode & OP_MASK == 1 {
            let mut value = 0;
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst_mode = match Operation0::from_int((opcode & INST_MODE_MASK) >> INST_MODE_SHIFT) {
                Ok(inst) => inst,
                _ => return false
            };
            match ADDR_2[addr_mode as usize]  {
                Immediate => {
                    value = self.pc;
                    self.pc += 1;
                },
                Zeropage => {
                    value = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                },
                Absolute => {
                    value = self.read_address(self.pc);
                    self.pc += 2;
                },
                ZeropageIndexed => {
                    value = self.bus.read(self.pc) as u16;
                    value = (value + self.x as u16) & 0xFF; // zero-page memory page
                },
                AbsoluteIndexed => {
                    value = self.read_address(self.pc);
                    self.pc += 2;
                    self.set_page_crossed(value, value + self.x as u16, 1);
                    value += self.x as u16; // zero-page memory page
                },
                _ => { return false }
            }
            match inst_mode {
                BIT => {
                    let operand = self.bus.read(value);
                    // assigning to P register is wrong! TODO
                    //self.p |= operand & 0x80 | operand & 0x40;
                    self.set_n(operand & 0x80);
                    self.set_v(operand & 0x40);
                    //self.p |= if operand & self.a == 0 { 0x02 } else { 0 };
                    self.set_z(if operand & self.a == 0 { 0x02 } else { 0 });
                },
                STY => {
                    self.bus.write(value, self.y);
                },
                LDY => {
                    self.y = self.bus.read(value);
                    self.set_zn(self.y)
                },
                CPY => {
                    let diff = ((self.y as u16) | 0x8000) - self.bus.read(value) as u16;
                    //self.p &= ((((diff & 0x100) >> 8) ^ 1) as u8);
                    self.set_c((((diff & 0x100) >> 8) ^ 1) as u8);
                    self.set_zn(diff as u8);
                },
                CPX => {
                    let diff = ((self.x as u16) | 0x8000) - self.bus.read(value) as u16;
                    //self.p &= (((diff & 0x100) >> 8) ^ 1) as u8;
                    self.set_c((((diff & 0x100) >> 8) ^ 1) as u8);
                    self.set_zn(diff as u8);
                }
            }
        }
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
                self.pc = self.pull_stack() as u16 | ((self.pull_stack() as u16) * 0x100);
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

    fn set_v(&mut self, v: u8) {
        self.p = (self.p & 0xBF) | v;
    }

    fn set_n(&mut self, n: u8) {
        self.p = (self.p & 0x7F) | n;
    }

    fn set_c(&mut self, c: u8) {
        self.p = (self.p & 0xFE) | c;
    }

    fn set_z(&mut self, z: u8) {
        self.p = (self.p & 0xFD) | z;
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

    fn read_address(&mut self, addr: u16) -> u16 {
        (self.bus.read(addr + 1) as u16) * 0x100 + self.bus.read(addr) as u16
    }

}