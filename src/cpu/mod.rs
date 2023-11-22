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
    bus: BUS,
    skip_cycles: usize
}

impl CPU {
    pub fn new(bus: BUS) -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            p: 0x34,
            bus,
            cycle: 0,
            skip_cycles: 0
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {
        if self.skip_cycles > 0 {
            self.cycle -= 1;
            return;
        }
        if (*interrupt) == Interrupt::NMI {
            self.execute_nmi();
        }
        let op = self.bus.read(self.pc);
        self.pc += 1;
        let cycle_len = OP_CYCLES[op as usize] as usize;
        if self.execute_implied(op) || self.execute_relative(op) || self.operation1(op) || self.operation2(op) || self.operation0(op) {
            self.cycle += cycle_len;
        }
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

    pub fn interrupt(&mut self, interrupt: &Interrupt) {
        unimplemented!();
    }

    fn execute_nmi(&mut self) {
        let value = self.bus.read(self.pc);
        self.bus.write(0x4014, value);
        if self.cycle % 2 == 0 {
            self.skip_cycles += 513;
        } else {
            self.skip_cycles += 514;
        }
    }

    fn execute_relative(&mut self, opcode: u8) -> bool { 
        let status = [0x1 & self.p, (0x2 & self.p) >> 1, (0x8 & self.p) >> 3, (0x40 & self.p) >> 6 , (0x80 & self.p) >> 7];
        if (opcode & 0x1F) == 0x10 {
            let opcode = (opcode & 0x20) >> 5;
            if status.iter().any(|s| s == &opcode) {
                let new_pc = self.pc + self.bus.read(self.pc) as u16;
                self.cycle += 1;
                self.set_page_crossed(self.pc, new_pc, 2);
                self.pc = new_pc;
            } else {
                self.pc += 1;
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

    fn get_address_mode(&mut self, addr_mode: &AddressingMode, inst: u8) -> u16 {
        match addr_mode {
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc += 1;
                addr
            },
            AddressingMode::Absolute => {
                let addr = self.read_address(self.pc);
                self.pc += 2;
                addr
            },
            AddressingMode::Zeropage => {
                let addr = self.bus.read(self.pc);
                self.pc += 1;
                addr as u16
            },
            AddressingMode::ZeropageX => {
                let addr = self.bus.read(self.pc).wrapping_add(self.x);
                self.pc += 1;
                addr as u16
            },
            AddressingMode::ZeropageY => {
                let addr = self.bus.read(self.pc).wrapping_add(self.y);
                self.pc += 1;
                addr as u16
            },
            AddressingMode::AbsoluteX => {
                let addr = self.read_address(self.pc);
                let addr_x = addr.wrapping_add(self.x as u16);
                // STA do not increment 1 cycle if page crossed.
                if inst != 0x99 { self.set_page_crossed(addr, addr_x, 1); }
                self.pc += 2;
                addr_x
            },
            AddressingMode::AbsoluteY => {
                let addr = self.read_address(self.pc);
                let addr_y = addr.wrapping_add(self.y as u16);
                if inst != 0x99 { self.set_page_crossed(addr, addr_y, 1); }
                self.pc += 2;
                addr_y
            },
            AddressingMode::IndirectX => {
                let addr = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                self.pc += 1;
                self.bus.read(addr & 0xFF) as u16 | (self.bus.read(addr.wrapping_add(1) & 0xFF) as u16) * 0x100
            },
            AddressingMode::IndirectY => {
                let addr = self.bus.read(self.pc) as u16;
                self.pc += 1;
                ( self.bus.read(addr) as u16 | self.bus.read(addr.wrapping_add(1) & 0xFF) as u16 * 0x100 ).wrapping_add(self.y as u16)
            },
            AddressingMode::ZeropageIndexed => {
                let addr = self.bus.read(self.pc);
                let index = if inst == 0xB6 || inst == 0x96 { self.y } else { self.x };
                self.pc += 1;
                addr.wrapping_add(index) as u16
            },
            AddressingMode::AbsoluteIndexed => {
                let addr = self.read_address(self.pc);
                let index = if inst == 0xB6 || inst == 0x96 { self.y as u16 } else { self.x as u16 };
                let value = addr.wrapping_add(index);
                self.set_page_crossed(addr, value, 1);
                self.pc += 2;
                addr.wrapping_add(index)
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
            match inst {
                ORA => {
                    log(&format!("opcode: ORA"));
                    self.a |= self.bus.read(value);
                    self.set_zn(self.a);
                },
                AND => {
                    log(&format!("opcode: AND"));
                    self.a &= self.bus.read(value);
                    self.set_zn(self.a);
                },
                EOR => {
                    log(&format!("opcode: EOR"));
                    self.a ^= self.bus.read(value);
                    self.set_zn(self.a);
                },
                ADC => {
                    log(&format!("opcode: ADC"));
                    let carry = self.p & 0x1;
                    let value = self.bus.read(value);
                    let (sum, c1) = self.a.overflowing_add(value);
                    let (sum, c2) = sum.overflowing_add(carry);
                    self.set_c((c1 || c2) as u8);
                    // http://www.c-jump.com/CIS77/CPU/Overflow/lecture.html (Overflow Condition (signed))
                    // Shift right to place on Status register
                    self.set_v(((self.a ^ sum) & (value ^ sum) & 0x80) >> 1);
                    self.a = sum;
                    self.set_zn(self.a);
                },
                STA => {
                    log(&format!("opcode: STA"));
                    self.bus.write(value, self.a)
                },
                LDA => {
                    log(&format!("opcode: LDA"));
                    self.a = self.bus.read(value);
                    self.set_zn(self.a);
                },
                CMP => {
                    log(&format!("opcode: CMP"));
                    let value = self.bus.read(value);
                    let sum = self.a.wrapping_sub(value);
                    self.set_c((self.a >= value) as u8);
                    self.set_zn(sum);
                },
                SBC => {
                    log(&format!("opcode: SBC"));
                    let value = self.bus.read(value);
                    let carry = self.p & 0x1 ^ 0x1; // NOT(c)
                    // https://github.com/rust-lang/rust/blob/cc946fcd326f7d85d4af096efdc73538622568e9/library/core/src/num/uint_macros.rs#L1538-L1544
                    let (sub, c1) = self.a.overflowing_sub(value);
                    let (sub, c2) = sub.overflowing_sub(carry);
                    self.set_c(!(c1 || c2) as u8);
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
        if opcode & OP_MASK == 1 {
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(&ADDR_2[addr_mode as usize], opcode);
            let inst = match Operation2::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            match inst {
                ASL => {
                    log(&format!("opcode: ASL"));
                    if opcode == 0x0A {
                        self.set_c((self.a & 0x80) >> 7);
                        self.a = self.a << 1;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.set_c((operand & 0x80) >> 7);
                        operand = operand << 1;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                ROL => {
                    log(&format!("opcode: ROL"));
                    if opcode == 0x2A {
                        let carry = self.p & 0x1;
                        self.set_c((self.a & 0x80) >> 7);
                        self.a = self.a << 1 | carry;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.p & 0x1;
                        self.set_c((operand & 0x80) >> 7);
                        operand = operand << 1 | carry;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                LSR => {
                    log(&format!("opcode: LSR"));
                    if opcode == 0x4A {
                        self.set_c(self.a & 0x1);
                        self.a = self.a >> 1;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.set_c(operand & 0x1);
                        operand = operand >> 1;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                ROR => {
                    log(&format!("opcode: ROR"));
                    if opcode == 0x6A {
                        let carry = self.p & 0x1;
                        self.set_c(self.a & 0x1);
                        self.a = self.a >> 1 | carry << 7;
                        self.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.p & 0x1;
                        self.set_c(operand & 0x1);
                        operand = operand >> 1 | carry << 7;
                        self.bus.write(value, operand);
                        self.set_zn(operand);
                    }
                },
                STX => {
                    log(&format!("opcode: STX"));
                    self.bus.write(value, self.x)
                },
                LDX => {
                    log(&format!("opcode: LDX"));
                    self.x = self.bus.read(value);
                    self.set_zn(self.x);
                },
                DEC => {
                    log(&format!("opcode: DEC"));
                    let operand = self.bus.read(value).wrapping_sub(1);
                    self.bus.write(value, operand);
                    self.set_zn(operand);
                },
                INC => {
                    log(&format!("opcode: INC"));
                    let operand = self.bus.read(value).wrapping_sub(1);
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
        if opcode & OP_MASK == 1 {
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(&ADDR_2[addr_mode as usize], opcode);
            let inst = match Operation0::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            match inst {
                BIT => {
                    log(&format!("opcode: BIT"));
                    let operand = self.bus.read(value).wrapping_sub(1);
                    let operand = self.bus.read(value);
                    self.set_n(operand & 0x80);
                    self.set_v(operand & 0x40);
                    self.set_z(if operand & self.a == 0 { 0x02 } else { 0 });
                },
                STY => {
                    log(&format!("opcode: STY"));
                    self.bus.write(value, self.y);
                },
                LDY => {
                    log(&format!("opcode: LDY"));
                    self.bus.write(value, self.y);
                    self.y = self.bus.read(value);
                    self.set_zn(self.y)
                },
                CPY => {
                    log(&format!("opcode: CPY"));
                    let value = self.bus.read(value); 
                    let diff = self.y.wrapping_sub(value);
                    self.set_c((self.y >= value) as u8);
                    self.set_zn(diff);
                },
                CPX => {
                    log(&format!("opcode: CPX"));
                    let value = self.bus.read(value); 
                    let diff = self.x.wrapping_sub(value);
                    self.set_c((self.x >= value) as u8);
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
        match implied {
            BRK => {
                log(&format!("opcode: BRK | cycle: {} | acc: {} | x: {} | s: {} | p: {} | pc: {} ", self.cycle, self.a, self.x, self.s, self.p, self.pc));
                // self.p |= 0x10;
                self.p |= 0x04;
            },
            TXA => {
                log(&format!("opcode: TXA"));
                self.a = self.x;
                self.set_zn(self.a);
            },
            TAX => {
                log(&format!("opcode: TAX"));
                self.x = self.a;
                self.set_zn(self.x);
            },
            TXS => {
                log(&format!("opcode: TXS"));
                self.s = self.x;
            },
            DEX => {
                log(&format!("opcode: DEX"));
                self.x -= 1;
                self.set_zn(self.x);
            },
            TSX => {
                log(&format!("opcode: TSX"));
                self.x = self.s;
                self.set_zn(self.x);
            },
            RTI => {
                log(&format!("opcode: RTI"));
                self.p = self.pull_stack() & 0xCF;
                self.pc = self.pull_stack() as u16 | ((self.pull_stack() as u16) * 0x100);
            },
            RTS => {
                log(&format!("opcode: RTS"));
                self.pc = self.pull_stack() as u16 | self.pull_stack() as u16 * 0x100;
                self.pc += 1;
            },
            PHP => {
                log(&format!("opcode: PHP"));
                self.push_stack(self.p | 0x30);
            },
            CLC => {
                log(&format!("opcode: CLC"));
                self.p &= 0xFE;
            },
            PLP => {
                log(&format!("opcode: PLP"));
                self.p = self.pull_stack();
            },
            SEC => {
                log(&format!("opcode: SEC"));
                self.p |= 0x01;
            },
            PHA => {
                log(&format!("opcode: PHA"));
                self.push_stack(self.a);
            },
            CLI => {
                log(&format!("opcode: CLI"));
                self.p &= 0xFB;
            },
            PLA => {
                log(&format!("opcode: PLA"));
                self.a = self.pull_stack();
                self.set_zn(self.a);
            },
            SEI => {
                log(&format!("opcode: SEI"));
                self.p |= 0x04;
            },
            DEY => {
                log(&format!("opcode: DEY"));
                self.y -= 1;
                self.set_zn(self.y);
            },
            TYA => {
                log(&format!("opcode: TYA"));
                self.a = self.y;
                self.set_zn(self.a);
            },
            TAY => {
                log(&format!("opcode: TAY"));
                self.y = self.a;
                self.set_zn(self.y);
            },
            CLV => {
                log(&format!("opcode: CLV"));
                self.p &= 0x40;
            },
            INY => {
                log(&format!("opcode: INY"));
                self.y += 1;
                self.set_zn(self.y)
            },
            CLD => {
                log(&format!("opcode: CLD"));
                self.p &= 0xF7;
            },
            INX => {
                log(&format!("opcode: INX"));
                self.x += 1;
                self.set_zn(self.x);
            },
            SED => {
                log(&format!("opcode: SED"));
                self.p |= 0x08;
            },
            NOP => {
                log(&format!("opcode: NOP"));
                todo!()
            },
        }
        true
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
