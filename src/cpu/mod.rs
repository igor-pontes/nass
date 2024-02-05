mod bus;
mod instructions;
mod status_register;
pub use self::bus::BUS;
use status_register::*;
use crate::Interrupt;
use crate::cpu::instructions::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

const NMI_VECTOR: u16 = 0xFFFA; 
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

pub struct CPU {
    a: u8, // Accumulator
    x: u8, // register x
    y: u8, // register y
    pub pc: u16, // Program counter
    s: u8, // Stack pointer (256-byte stack at $0100-$01FF.)
    status: StatusRegister,
    pub cycle: usize,
    pub bus: BUS,
    pub odd_cycle: bool,
    pub debug: bool,
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
            status: StatusRegister::new(),
            bus,
            cycle: 0,
            odd_cycle: true,
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {

        self.odd_cycle = !self.odd_cycle;
        
        if self.cycle != 0 {
            self.cycle -= 1;
            return;
        }

        if (*interrupt) == Interrupt::NMI {
            self.execute_nmi();
            (*interrupt) = Interrupt::DISABLED; 
            self.cycle -= 1;
            return;
        }

        let op = self.bus.read(self.pc);

        // if op == 00 { self.debug = true; }

        if self.execute_implied(op) || self.execute_relative(op) || self.operation2(op) || self.operation1(op) || self.operation0(op) {
            self.cycle += OP_CYCLES[op as usize] as usize;
        }

        if self.debug { 
            let status = self.status.bits();
            log(&format!("pc: {:#06x} | cycle: {:#06x} | A: {:#06x} | X: {:#06x} | Y: {:#06x} | s: {:#06x} | p: {:#010b} ({:#04x})", self.pc, self.cycle, self.a, self.x, self.y, self.s, status, status)); 
        }

        self.cycle -= 1;
        self.pc += 1;
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.a = 0;
        self.s = 0xFD;
        self.cycle = 7;
        self.status = StatusRegister::new();
        self.cycle = 0;
        self.pc = self.read_address(RESET_VECTOR);
        self.odd_cycle = true;
    }

    fn execute_nmi(&mut self) {
        // self.debug = true;
        self.cycle = 7;
        self.push_stack(((self.pc & 0xFF00) >> 8) as u8);
        self.push_stack((self.pc & 0x00FF) as u8);
        self.push_stack(self.status.bits() & !0x10);
        self.pc = self.read_address(NMI_VECTOR); // Do not need to decrement by one
    }

    fn execute_relative(&mut self, opcode: u8) -> bool { 
        let bits = self.status.bits();
        let status = [(0x80 & bits) >> 7, (0x40 & bits) >> 6, 0x01 & bits, (0x02 & bits) >> 1];
        if (opcode & 0x1F) == 0x10 {
            let inst = (opcode & 0xC0) >> 6;
            let cond = (opcode & 0x20) >> 5;
            if status[inst as usize] == cond {
                if self.debug { log(&format!("[BRANCH ({opcode:#06x})] condition = true")); }
                let offset = self.bus.read(self.pc + 1) as i8;
                let old_pc = self.pc + 2;
                let (new_pc, _) = old_pc.overflowing_add_signed(offset as i16);
                self.cycle += 1;
                self.set_page_crossed(old_pc, new_pc);
                self.pc = new_pc - 1;
            } else { 
                if self.debug { log(&format!("[BRANCH ({opcode:#06x})] condition = false")); }
                self.pc += 1;
            }
            true
        } else {
            false
        }
    }

    fn set_page_crossed(&mut self, a: u16, b: u16) {
        if a & 0xFF00 != b & 0xFF00 { self.cycle += 1; }
    }

    fn get_address_mode(&mut self, addr_mode: &AddrMode, inst: u8) -> u16 {
        match addr_mode {
            AddrMode::Imm => {
                self.pc += 1;
                self.pc
            },
            AddrMode::Ind => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let operand = (self.bus.read((addr & 0xFF00) | ((addr + 1) & 0x00FF)) as u16) * 0x100 | self.bus.read(addr) as u16;
                operand
            },
            AddrMode::Abs => {
                self.pc += 1;
                let operand = self.read_address(self.pc);
                self.pc += 1;
                operand
            },
            AddrMode::Zp => {
                self.pc += 1;
                self.bus.read(self.pc) as u16
            },
            AddrMode::ZpX => {
                self.pc += 1;
                (self.bus.read(self.pc) + self.x) as u16
            },
            AddrMode::ZpY => {
                self.pc += 1;
                (self.bus.read(self.pc) + self.y) as u16
            },
            AddrMode::AbsX => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let operand = addr + self.x as u16;
                self.set_page_crossed(addr, operand);
                operand
            },
            AddrMode::AbsY => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let operand = addr + self.y as u16;
                if inst != 0x99 { self.set_page_crossed(addr, operand); }
                operand
            },
            AddrMode::IndX => {
                self.pc += 1;
                let arg = (self.bus.read(self.pc) + self.x) as u16;
                self.bus.read(arg & 0xFF) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100
            },
            AddrMode::IndrY => {
                self.pc += 1;
                let arg = self.bus.read(self.pc) as u16;
                let addr = self.bus.read(arg) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100;
                let operand = addr + self.y as u16;
                if inst != 0x91 { self.set_page_crossed(addr, operand); }
                operand
            },
            AddrMode::ZpInd => {
                self.pc += 1;
                let addr = self.bus.read(self.pc);
                let index = if inst == 0xB6 || inst == 0x96 { self.y } else { self.x };
                (addr + index) as u16
            },
            AddrMode::AbsInd => {
                self.pc += 1;
                let addr = self.read_address(self.pc);
                self.pc += 1;
                let index = if inst == 0xBE { self.y } else { self.x } as u16;
                let operand = addr + index;
                if (inst & 0x0E != 0x0E) && (inst != 0x9D) || (inst == 0xBE) { self.set_page_crossed(addr, operand); }
                operand
            },
            _ => 0
        }
    }

    fn operation1(&mut self, opcode: u8) -> bool {
        use Operation1::*;
        if opcode & OP_MASK == 1 {
            let temp = self.pc;
            let addr_mode = &ADDR[((opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT) as usize];
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(addr_mode, opcode);
            let inst = match Operation1::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };

            if self.debug { log(&format!("---- {inst:?}({opcode:#04x}) [{temp:#06x}] | Addr mode: {:?} | VALUE(addr): {value:#06x} | A: {:#04x} | X: {:#04x} | Y: {:#04x} | SP: {:#06x} | S {:#010b} ----", addr_mode, self.a, self.x, self.y, self.s, self.status.bits())); }

            let add = |value, carry| {
                let (sum, c1) = self.a.overflowing_add(value);
                let (sum, c2) = sum.overflowing_add(carry);
                (sum, c1 || c2)

            };
            match inst {
                ORA => {
                    self.a |= self.bus.read(value);
                    self.status.set_zero_negative(self.a);
                },
                AND => {
                    self.a &= self.bus.read(value);
                    self.status.set_zero_negative(self.a);
                },
                EOR => {
                    self.a ^= self.bus.read(value);
                    self.status.set_zero_negative(self.a);
                },
                ADC => {
                    let carry = self.status.bits() & 0x1;
                    let value = self.bus.read(value);
                    let (sum, carry) = add(value, carry);
                    self.status.set_carry(carry);
                    self.status.set_overflow(((self.a ^ sum) & (value ^ sum) & 0x80) != 0);
                    self.a = sum;
                    self.status.set_zero_negative(self.a);
                },
                STA => self.bus.write(value, self.a),
                LDA => {
                    self.a = self.bus.read(value);
                    self.status.set_zero_negative(self.a);
                },
                CMP => {
                    let value = self.bus.read(value);
                    let diff = self.a - value;
                    self.status.set_carry(self.a >= value);
                    self.status.set_zero_negative(diff);
                },
                SBC => {
                    let carry = self.status.bits() & 0x1;
                    let value = self.bus.read(value);
                    let (sum, carry) = add(!value, carry);
                    self.status.set_carry(carry);
                    self.status.set_overflow(((self.a ^ sum) & (value ^ sum) & 0x80) > 1);
                    self.a = sum;
                    self.status.set_zero_negative(self.a);
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
            let temp = self.pc;
            let mut addr_mode = &AddrMode::None;
            let oper_is_a = (opcode & 0x0F) == 0x0A;

            if !oper_is_a { addr_mode = &ADDR[((opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT) as usize]; }
            if opcode == 0xA2 { addr_mode = &AddrMode::Imm; }
            let value = self.get_address_mode(addr_mode, opcode);
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;

            let inst = match Operation2::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            if self.debug { log(&format!("---- {inst:?}({opcode:#04x}) [{temp:#06x}] | Addr mode: {:?} | VALUE(addr): {value:#06x} | A: {:#04x} | X: {:#04x} | Y: {:#04x} | SP: {:#06x} | S {:#010b} ----", addr_mode, self.a, self.x, self.y, self.s, self.status.bits())); }

            match inst {
                ASL => {
                    if oper_is_a {
                        self.status.set_carry((self.a & 0x80) > 0);
                        self.a <<= 1;
                        self.status.set_zero_negative(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.status.set_carry((operand & 0x80) > 0);
                        operand <<= 1;
                        self.status.set_zero_negative(operand);
                        self.bus.write(value, operand);
                    }
                },
                ROL => {
                    if oper_is_a {
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((self.a & 0x80) > 0);
                        self.a = self.a << 1 | carry;
                        self.status.set_zero_negative(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((operand & 0x80) > 0);
                        operand = operand << 1 | carry;
                        self.status.set_zero_negative(operand);
                        self.bus.write(value, operand);
                    }
                },
                LSR => {
                    if oper_is_a {
                        self.status.set_carry((self.a & 0x1) == 1);
                        self.a >>= 1;
                        self.status.set_zero_negative(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.status.set_carry((operand & 0x1) == 1);
                        operand >>= 1;
                        self.status.set_zero_negative(operand);
                        self.bus.write(value, operand);
                    }
                },
                ROR => {
                    if oper_is_a {
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((self.a & 0x1) == 1);
                        self.a = self.a >> 1 | carry << 7;
                        self.status.set_zero_negative(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((operand & 0x1) == 1);
                        operand = operand >> 1 | carry << 7;
                        self.status.set_zero_negative(operand);
                        self.bus.write(value, operand);
                    }
                },
                STX => self.bus.write(value, self.x),
                LDX => {
                    self.x = self.bus.read(value);
                    self.status.set_zero_negative(self.x);
                },
                DEC => {
                    let operand = self.bus.read(value) - 1;
                    self.bus.write(value, operand);
                    self.status.set_zero_negative(operand);
                },
                INC => {
                    let operand = self.bus.read(value) + 1;
                    self.status.set_zero_negative(operand);
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
        if opcode & OP_MASK == 0 {
            let temp_pc = self.pc;
            let mut inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let mut addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            if opcode == 0x20 { inst = 0; addr_mode = 3; }
            if opcode == 0x6C { addr_mode = 2; }
            let value = self.get_address_mode(&ADDR0[addr_mode as usize], opcode);
            let inst = match Operation0::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            if self.debug { log(&format!("---- {inst:?}({opcode:#04x}) [{temp_pc:#06x}] | Addr mode: {:?} | VALUE(addr): {value:#06x} | A: {:#04x} | X: {:#04x} | Y: {:#04x} | SP: {:#06x} | S {:#010b} ----", addr_mode, self.a, self.x, self.y, self.s, self.status.bits())); }

            match inst {
                JSR => {
                    let return_addr = temp_pc+2;
                    self.push_stack(((return_addr & 0xFF00) >> 8) as u8);
                    self.push_stack((return_addr & 0x00FF) as u8);
                    self.pc = value - 1;
                },
                BIT => {
                    let operand = self.bus.read(value);
                    self.status.set_negative(operand & 0x80 > 0);
                    self.status.set_overflow(operand & 0x40 > 0);
                    self.status.set_zero((operand & self.a) == 0);
                },
                JMP  => self.pc = value-1,
                _JMP => self.pc = value-1,
                STY => self.bus.write(value, self.y),
                LDY => {
                    self.y = self.bus.read(value);
                    self.status.set_zero_negative(self.y);
                },
                CPY => {
                    let value = self.bus.read(value); 
                    let diff = self.y.wrapping_sub(value);
                    self.status.set_carry(self.y >= value);
                    self.status.set_zero_negative(diff);
                },
                CPX => {
                    let value = self.bus.read(value); 
                    let diff = self.x - value;
                    self.status.set_carry(self.x >= value);
                    self.status.set_zero_negative(diff);
                }
            }
            true
        } else {
            false
        }
    }

    fn execute_implied(&mut self, opcode: u8) -> bool {
        let temp = self.pc;
        use ImplicitOps::*;
        let implied = match ImplicitOps::try_from(opcode) {
            Ok(i) => i,
            _ => return false
        };
        if self.debug { log(&format!("---- {implied:?} [{temp:#06x}] ----")); }

        match implied {
            BRK => {
                let ret_addr = self.pc+2;
                self.push_stack((ret_addr & 0xFF00 >> 8) as u8);
                self.push_stack((ret_addr & 0x00FF) as u8);
                self.status.set_break(true); 
                self.push_stack(self.status.bits() | 0x20);
                self.status.set_interrupt_disable(true); 
                self.pc = self.read_address(IRQ_VECTOR)-1;
            },
            TXA => {
                self.a = self.x;
                self.status.set_zero_negative(self.a);
            },
            TAX => {
                self.x = self.a;
                self.status.set_zero_negative(self.x);
            },
            TXS => self.s = self.x,
            DEX => {
                self.x -= 1;
                self.status.set_zero_negative(self.x);
            },
            TSX => {
                self.x = self.s;
                self.status.set_zero_negative(self.x);
            },
            RTI => {
                let value = self.pull_stack();
                self.status.set_effective(value);
                self.pc = ((self.pull_stack() as u16) | ((self.pull_stack() as u16) * 0x100))-1;
            },
            RTS => self.pc = (self.pull_stack() as u16) | ((self.pull_stack() as u16 ) * 0x100),
            PHP => self.push_stack(self.status.bits() | 0x30),
            CLC => self.status.set_carry(false),
            PLP => { 
                let value = self.pull_stack();
                self.status.set_effective(value)
            },
            SEC => self.status.set_carry(true),
            PHA => self.push_stack(self.a),
            CLI => self.status.set_interrupt_disable(false),
            PLA => {
                self.a = self.pull_stack();
                self.status.set_zero_negative(self.a);
            },
            SEI => self.status.set_interrupt_disable(true),
            DEY => {
                self.y -= 1;
                self.status.set_zero_negative(self.y);
            },
            TYA => {
                self.a = self.y;
                self.status.set_zero_negative(self.a);
            },
            TAY => {
                self.y = self.a;
                self.status.set_zero_negative(self.y);
            },
            CLV => self.status.set_overflow(false),
            INY => {
                self.y += 1;
                self.status.set_zero_negative(self.y);
            },
            CLD => self.status.set_decimal(false),
            INX => {
                self.x += 1;
                self.status.set_zero_negative(self.x);
            },
            SED => self.status.set_decimal(true),
            NOP => (), // Increments program counter.
        }
        true
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
