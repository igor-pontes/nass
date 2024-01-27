mod bus;
mod instructions;
pub use self::bus::BUS;
use crate::Interrupt;
use crate::cpu::instructions::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const NMI_VECTOR: u16 = 0xFFFA; 
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

pub struct CPU {
    a: u8, // Accumulator
    x: u8, // register x
    y: u8, // register y
    pub pc: u16, // Program counter
    s: u8, // Stack pointer (It indexes into a 256-byte stack at $0100-$01FF.)
    p: u8, // Status Register (flags)
    pub cycle: usize,
    pub bus: BUS,
    pub odd_cycle: bool,
    // pub skip_cycles: usize,
    debug: bool
}

impl CPU {
    pub fn new(bus: BUS) -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            debug: false,
            s: 0xFD,
            p: 0x34, // 0011 0100 (IRQ disabled)
            bus,
            cycle: 0,
            odd_cycle: false,
            // skip_cycles: 0
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {

        if self.cycle % 2 == 0 && self.odd_cycle == true { 
            self.odd_cycle = true; 
        } else { 
            self.odd_cycle = false; 
        }

        if self.cycle > 0 {
            self.cycle -= 1;
            return;
        }

        if (*interrupt) == Interrupt::NMI {
            self.execute_nmi();
            (*interrupt) = Interrupt::DISABLED; 
        }

        let op = self.bus.read(self.pc);

        if self.jsr(op) || self.execute_implied(op) || self.execute_relative(op) || self.operation1(op) || self.operation2(op) || self.operation0(op) {
            self.cycle += OP_CYCLES[op as usize] as usize;
        }

        if self.debug { 
            log(&format!("pc: {:#06x} | cycle: {:#06x} | A: {:#06x} | X: {:#06x} | Y: {:#06x} | s: {:#06x} | p: {:#010b} ({:#04x})", self.pc, self.cycle, self.a, self.x, self.y, self.s, self.p, self.p)); 
        }

        self.pc += 1;

    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.p = 0x44;
        self.pc = 0;
        self.s = 0xFD;
        self.pc = self.read_address(RESET_VECTOR);
    }

    // TODO
    pub fn interrupt(&mut self, interrupt: &Interrupt) {
        unimplemented!();
    }

    fn execute_nmi(&mut self) {
        self.push_stack(self.p);
        self.push_stack((self.pc & 0x00FF) as u8);
        self.push_stack(((self.pc & 0xFF00) >> 8) as u8);
        self.pc = self.read_address(NMI_VECTOR);
    }

    fn execute_relative(&mut self, opcode: u8) -> bool { 
        // OBS: Subtract one from self.pc here because "step()" function already addds one to the
        // program counter.
        
        // 000 10000 - Branch on N=0 (self.p & 0x80 = 0)
        // 001 10000 - Branch on N=1 (self.p & 0x80 = 1)
        // 010 10000 - Branch on V=0 (self.p & 0x40 = 0)
        // 011 10000 - Branch on V=1 (self.p & 0x40 = 1)
        // 100 10000 - Branch on C=0 (self.p & 0x01 = 0)
        // 101 10000 - Branch on C=1 (self.p & 0x01 = 1)
        // 110 10000 - Branch on Z=0 (self.p & 0x02 = 0) [BNE] (inst = 3, cond = 0)
        // 111 10000 - Branch on Z=1 (self.p & 0x02 = 1) [BEQ] (inst = 3, cond = 1)
        
        let status = [(0x80 & self.p) >> 7, (0x40 & self.p) >> 6, 0x01 & self.p, (0x02 & self.p) >> 1];

        if (opcode & 0x1F) == 0x10 {
            let cond = (opcode & 0x20) >> 5;
            let inst = (opcode & 0xC0) >> 6;

            if status[inst as usize] == cond {
                if self.debug { log(&format!("[BRANCH_IN] inst: {} | cond: {} | status: {:#010b} | Y: {:#04x} | X: {:#04x} | cond == status: {}", inst, cond, self.p, self.y, self.x, status[inst as usize] == cond)); }
                let offset = self.bus.read(self.pc + 1) as i8;
                let old_pc = self.pc + 2;
                let (new_pc, _) = old_pc.overflowing_add_signed(offset as i16);
                self.cycle += 1;
                self.set_page_crossed(self.pc, new_pc, 1);
                self.pc = new_pc - 1;
            } else { 
                if self.debug { log(&format!("[BRANCH_OUT] inst: {} | cond: {} | status: {:#010b} | Y: {:#04x} | X: {:#04x} | cond == status: {}", inst, cond, self.p, self.y, self.x, status[inst as usize] == cond)); }
                self.pc += 2 - 1; // next instruction
            }
            true
        } else {
            false
        }
    }

    fn set_page_crossed(&mut self, a: u16, b: u16, inc: usize) {
        if a & 0xFF00 != b & 0xFF00 {
            self.cycle += inc;
        }
    }

    fn jsr(&mut self, opcode: u8) -> bool {
        if opcode == 0x20 {
            // let temp = self.pc;
            let return_addr = self.pc+2;
            self.push_stack((return_addr & 0x00FF) as u8);
            self.push_stack(((return_addr & 0xFF00) >> 8) as u8);
            let value = self.get_address_mode(&AddressingMode::Absolute, opcode);
            if self.debug { log(&format!("---- JSR | VALUE: {value:#06x} ----")); }
            self.pc = value-1;
            // if temp == 0x8051 { log(&format!("[JSR] new_pc: {:#04x} | status: {:#010b} | Y: {:#04x} | X: {:#04x} | SP: {:#04X}", self.pc + 1, self.p, self.y, self.x, self.s)); }
            // if temp == 0x8054 { log(&format!("[JSR] new_pc: {:#04x} | status: {:#010b} | Y: {:#04x} | X: {:#04x} | SP: {:#04X}", self.pc + 1, self.p, self.y, self.x, self.s)); panic!(); }
            true
        } else {
            false
        }
    }

    fn get_address_mode(&mut self, addr_mode: &AddressingMode, inst: u8) -> u16 {
        match addr_mode {
            AddressingMode::Immediate => {
                self.pc += 1;
                let addr = self.pc;
                addr
            },
            AddressingMode::Absolute => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                addr
            },
            AddressingMode::Zeropage => {
                self.pc += 1;
                let addr = self.bus.read(self.pc);
                addr as u16
            },
            AddressingMode::ZeropageX => {
                self.pc += 1;
                let addr = self.bus.read(self.pc).wrapping_add(self.x);
                addr as u16
            },
            AddressingMode::ZeropageY => {
                self.pc += 1;
                let addr = self.bus.read(self.pc).wrapping_add(self.y);
                addr as u16
            },
            AddressingMode::AbsoluteX => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let addr_x = addr.wrapping_add(self.x as u16);
                // STA do not increment 1 cycle if page crossed.
                if inst != 0x99 { self.set_page_crossed(addr, addr_x, 1); }
                addr_x
            },
            AddressingMode::AbsoluteY => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let addr_y = addr.wrapping_add(self.y as u16);
                if inst != 0x99 { self.set_page_crossed(addr, addr_y, 1); }
                addr_y
            },
            AddressingMode::IndirectX => {
                self.pc += 1;
                let addr = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                self.bus.read(addr & 0xFF) as u16 | (self.bus.read(addr.wrapping_add(1) & 0xFF) as u16) * 0x100
            },
            AddressingMode::IndirectY => {
                self.pc += 1;
                let addr = self.bus.read(self.pc) as u16;
                let addr_y = ( self.bus.read(addr) as u16 | self.bus.read(addr.wrapping_add(1) & 0xFF) as u16 * 0x100 ).wrapping_add(self.y as u16);
                if inst != 0x91 { self.set_page_crossed(addr, addr_y, 1); }
                addr_y
            },
            AddressingMode::ZeropageIndexed => {
                self.pc += 1;
                let addr = self.bus.read(self.pc);
                let index = if inst == 0xB6 || inst == 0x96 { self.y } else { self.x };
                addr.wrapping_add(index) as u16
            },
            AddressingMode::AbsoluteIndexed => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let index = if inst == 0xB6 || inst == 0x96 { self.y as u16 } else { self.x as u16 };
                let value = addr.wrapping_add(index);
                self.set_page_crossed(addr, value, 1);
                let addr = addr.wrapping_add(index);
                addr
            },
            _ => 0
        }
    }

    fn operation1(&mut self, opcode: u8) -> bool {
        use Operation1::*;
        if opcode & OP_MASK == 1 {
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(&ADDR_1[addr_mode as usize], opcode);
            let inst = match Operation1::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            if self.debug { log(&format!("---- {inst:?} | VALUE(addr): {value:#06x} ----")); }
            match inst {
                ORA => {
                    self.a |= self.bus.read(value);
                    self.set_zn(self.a);
                },
                AND => {
                    self.a &= self.bus.read(value);
                    self.set_zn(self.a);
                },
                EOR => {
                    self.a ^= self.bus.read(value);
                    self.set_zn(self.a);
                },
                ADC => {
                    let carry = self.p & 0x1;
                    let value = self.bus.read(value);
                    let (sum, c1) = self.a.overflowing_add(value);
                    let (sum, c2) = sum.overflowing_add(carry);
                    self.set_c(c1 || c2);
                    // http://www.c-jump.com/CIS77/CPU/Overflow/lecture.html (Overflow Condition (signed))
                    // Shift right to place on Status register
                    self.set_v(((self.a ^ sum) & (value ^ sum) & 0x80) >> 1);
                    self.a = sum;
                    self.set_zn(self.a);
                },
                STA => {
                    self.bus.write(value, self.a)
                },
                LDA => {
                    self.a = self.bus.read(value);
                    self.set_zn(self.a);
                },
                CMP => {
                    let value = self.bus.read(value);
                    let _value = value;
                    let sum = self.a.wrapping_sub(value);
                    self.set_c(self.a >= value);
                    self.set_z(self.a == value);
                    self.set_n(if sum & 0x80 > 0 { true } else { false });
                },
                SBC => {
                    let value = self.bus.read(value);
                    let carry = self.p & 0x1 ^ 0x1; // NOT(c)
                    // https://github.com/rust-lang/rust/blob/cc946fcd326f7d85d4af096efdc73538622568e9/library/core/src/num/uint_macros.rs#L1538-L1544
                    let (sub, c1) = self.a.overflowing_sub(value);
                    let (sub, c2) = sub.overflowing_sub(carry);
                    self.set_c(!(c1 || c2));
                    // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
                    // "!value" = One's complement of "value" 
                    self.set_v(((self.a ^ sub) & (!value ^ sub) & 0x80) as u8 >> 1);
                    self.a = sub;
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
        if opcode & OP_MASK == 2 {
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(&ADDR_2[addr_mode as usize], opcode);
            let inst = match Operation2::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            if self.debug { log(&format!("---- {inst:?} | VALUE: {value:#06x} ----")); }
            match inst {
                ASL => {
                    if opcode == 0x0A {
                        self.set_c(if ((self.a & 0x80) >> 7) > 0 { true } else { false });
                        self.a = self.a << 1;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.set_c(if ((operand & 0x80) >> 7) > 0 { true } else { false });
                        operand = operand << 1;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                ROL => {
                    if opcode == 0x2A {
                        let carry = self.p & 0x1;
                        self.set_c(if ((self.a & 0x80) >> 7) > 0 { true } else { false });
                        self.a = self.a << 1 | carry;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.p & 0x1;
                        self.set_c(if ((operand & 0x80) >> 7) > 0 { true } else { false });
                        operand = operand << 1 | carry;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                LSR => {
                    if opcode == 0x4A {
                        self.set_c(if self.a & 0x1 > 0 { true } else { false });
                        self.a = self.a >> 1;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.set_c(if operand & 0x1 > 0 { true } else { false });
                        operand = operand >> 1;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                ROR => {
                    if opcode == 0x6A {
                        let carry = self.p & 0x1;
                        self.set_c(if self.a & 0x1 > 0 { true } else { false });
                        self.a = self.a >> 1 | carry << 7;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.p & 0x1;
                        self.set_c(if operand & 0x1 > 0 { true } else { false });
                        operand = operand >> 1 | carry << 7;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                STX => {
                    self.bus.write(value, self.x)
                },
                LDX => {
                    self.x = self.bus.read(value);
                    self.set_zn(self.x);
                },
                DEC => {
                    let operand = self.bus.read(value).wrapping_sub(1);
                    self.bus.write(value, operand);
                    self.set_zn(operand);
                },
                INC => {
                    let operand = self.bus.read(value).wrapping_add(1);
                    self.bus.write(value, operand);
                    self.set_zn(operand);
                },
            }
            true
        } else {
            false
        }
    }

    fn operation0(&mut self, opcode: u8) -> bool {
        use Operation0::*;
        if opcode & OP_MASK == 0 {
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(&ADDR_2[addr_mode as usize], opcode);
            let inst = match Operation0::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            if self.debug { log(&format!("---- {inst:?} | VALUE: {value:#06x} ----")); }
            match inst {
                BIT => {
                    if value == 0x4d06 { log(&format!("---- {inst:?} | VALUE(addr): {value:#06x} ----")); panic!(""); }
                    let operand = self.bus.read(value);
                    self.set_n(if operand & 0x80 > 0 { true } else { false });
                    self.set_v(operand & 0x40);
                    self.set_z((operand & self.a) == 0);
                },
                JMP => {
                    // JMP == _JMP
                    // TODO: http://www.6502.org/users/obelisk/6502/reference.html#JMP
                    self.pc = value-1;
                }
                _JMP => {
                    self.pc = value-1;
                }
                STY => {
                    self.bus.write(value, self.y);
                },
                LDY => {
                    if value == 0x4d06 { log(&format!("---- {inst:?} | VALUE(addr): {value:#06x} ----")); panic!(""); }
                    self.y = self.bus.read(value);
                    self.set_zn(self.y)
                },
                CPY => {
                    if value == 0x4d06 { log(&format!("---- {inst:?} | VALUE(addr): {value:#06x} ----")); panic!(""); }
                    let value = self.bus.read(value); 
                    let diff = self.y.wrapping_sub(value);
                    self.set_c(self.y >= value);
                    self.set_zn(diff);
                },
                CPX => {
                    if value == 0x4d06 { log(&format!("---- {inst:?} | VALUE(addr): {value:#06x} ----")); panic!(""); }
                    let value = self.bus.read(value); 
                    let diff = self.x.wrapping_sub(value);
                    self.set_c(self.x >= value);
                    self.set_zn(diff);
                }
            }
            true
        } else {
            false
        }
    }

    fn execute_implied(&mut self, opcode: u8) -> bool {
        use ImplicitOps::*;
        let implied = match ImplicitOps::try_from(opcode) {
            Ok(i) => i,
            _ => return false
        };
        if self.debug { log(&format!("---- {implied:?} ----")); }
        match implied {
            BRK => {
                let ret_addr = self.pc + 2;
                self.push_stack(ret_addr as u8);
                self.push_stack((ret_addr >> 8) as u8);
                // The status register will be pushed to the stack with the break flag set to 1.
                self.push_stack(self.p | 0x10);
                self.pc = self.read_address(IRQ_VECTOR);
                self.p |= 0x04; 
            },
            TXA => {
                self.a = self.x;
                self.set_zn(self.a);
            },
            TAX => {
                self.x = self.a;
                self.set_zn(self.x);
            },
            TXS => {
                self.s = self.x;
            },
            DEX => {
                // if self.pc == 0x81bd { log(&format!("---- {implied:?} | X: {:#04x} | Y: {:#04x} | PC: {:#06x} | status: {:#010b} | sp: {:#06x} ----", self.x, self.y, self.pc, self.p, self.s)); }
                self.x -= 1;
                self.set_zn(self.x);
            },
            TSX => {
                self.x = self.s;
                self.set_zn(self.x);
            },
            RTI => {
                // if self.pc == 0x81a6 { log(&format!("---- SP: {:#06x} | VALUE(addr): {:#06x} ----", self.s, self.pc)); panic!(""); }
                self.p = self.pull_stack() & 0xCF;
                self.pc = (self.pull_stack() as u16 * 0x100 | ((self.pull_stack() as u16))) - 1;
            },
            RTS => {
                if self.debug { log(&format!("---- [{implied:?}] | X: {:#04x} | Y: {:#04x} | PC: {:#06x} | status: {:#010b} | sp: {:#06x} ----", self.x, self.y, self.pc, self.p, self.s)); panic!(); } 
                self.pc = self.pull_stack() as u16 * 0x100 | self.pull_stack() as u16;
            },
            PHP => {
                self.push_stack(self.p | 0x30);
            },
            CLC => {
                self.p &= 0xFE;
            },
            PLP => {
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
                // if self.pc == 0x81b1 { log(&format!("---- {implied:?} ----")); }
                self.y = self.a;
                self.set_zn(self.y);
            },
            CLV => {
                self.p &= 0x40;
            },
            INY => {
                // if self.pc == 0x81c6 { log(&format!("---- {implied:?} ----")); }
                self.y += 1;
                self.set_zn(self.y)
            },
            CLD => {
                // 11110111 
                self.p &= 0xF7;
            },
            INX => {
                self.x += 1;
                self.set_zn(self.x);
            },
            SED => {
                self.p |= 0x08;
            },
            NOP => (), // Increments program counter.
        }
        true
    }

    fn set_zn(&mut self, val: u8) {
        self.set_z(val == 0);
        self.set_n(if val & 0x80 > 0 { true } else { false });
    }

    fn set_v(&mut self, v: u8) {
        self.p = (self.p & 0xBF) | v;
    }

    fn set_n(&mut self, cond: bool) {
        self.p = (self.p & 0x7F) | (cond as u8) << 7;
    }

    fn set_c(&mut self, cond: bool) {
        self.p = (self.p & 0xFE) | (cond as u8);
    }

    fn set_z(&mut self, cond: bool) {
        self.p = (self.p & 0xFD) | if cond { 0x02 } else { 0 };
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
