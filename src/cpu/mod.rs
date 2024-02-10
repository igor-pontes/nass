mod bus;
mod instructions;
mod cpu_status;

pub use self::bus::*;
use cpu_status::*;
use crate::cpu::instructions::*;

// CPU is guaranteed to receive NMI every interrupt
const CYCLES_PER_FRAME: usize = 29780 * 2;
const NMI_VECTOR: u16 = 0xFFFA; 
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

pub struct CPU {
    a: u8, // Accumulator
    y: u8, // register y
    x: u8, // register x
    pc: u16, // Program counter
    s: u8, // Stack pointer (256-byte stack at $0100-$01FF.)
    status: CPUStatus,
    cycles: usize,
    pub bus: BUS,
    odd_cycle: bool,
}

impl CPU {
    pub fn new(bus: BUS) -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            status: CPUStatus::new(),
            bus,
            cycles: 0,
            odd_cycle: true,
        }
    }

    pub fn run_with_callback<F>(&mut self, mut _callback: F)
    where 
        F: FnMut(&mut CPU),
    {
        for _ in 0..CYCLES_PER_FRAME {
            self.odd_cycle = !self.odd_cycle;
            if self.cycles == 0 {
                let op = self.bus.read(self.pc);
                self.pc += 1;
                if  self.execute_implied(op) || 
                    self.execute_immediate(op) || 
                    self.execute_relative(op) || 
                    self.operation3(op) || 
                    self.operation2(op) || 
                    self.operation1(op) || 
                    self.operation0(op) 
                {
                    self.cycles += (CYCLES[op as usize] & CYCLES_MASK) as usize;
                }
                if let Some(Interrupt::Nmi) = self.bus.interrupt { 
                    self.bus.interrupt = None;
                    self.execute_nmi() 
                }
                if self.bus.suspend { 
                    if self.odd_cycle { self.cycles += 513; } else { self.cycles += 514; }
                    self.bus.suspend = false;
                }
            }
            self.cycles -= 1;
            self.bus.tick(self.cycles);
        }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.a = 0;
        self.s = 0xFD;
        self.cycles += 7;
        self.status = CPUStatus::new();
        self.pc = self.read_address(RESET_VECTOR);
        self.odd_cycle = true;
    }

    fn execute_nmi(&mut self) {
        self.cycles += 7;
        self.push_stack(((self.pc & 0xFF00) >> 8) as u8);
        self.push_stack((self.pc & 0x00FF) as u8);
        self.push_stack(self.status.bits() & !0x10);
        self.status.set_interrupt(true);
        self.pc = self.read_address(NMI_VECTOR); // Do not need to decrement by one
    }

    fn execute_relative(&mut self, opcode: u8) -> bool { 
        let bits = self.status.bits();
        let status = [(0x80 & bits) >> 7, (0x40 & bits) >> 6, 0x01 & bits, (0x02 & bits) >> 1];
        if (opcode & 0x1F) == 0x10 {
            let inst = (opcode & 0xC0) >> 6;
            let cond = (opcode & 0x20) >> 5;
            if status[inst as usize] == cond {
                let offset = self.bus.read(self.pc) as i8;
                self.pc += 1;
                let old_pc = self.pc;
                let (new_pc, _) = old_pc.overflowing_add_signed(offset as i16);
                self.cycles += 1;
                self.set_page_crossed(old_pc, new_pc);
                self.pc = new_pc;
            } else { 
                self.pc += 1;
            }
            return true
        }
        false
    }

    fn set_page_crossed(&mut self, a: u16, b: u16) {
        if a & 0xFF00 != b & 0xFF00 { self.cycles += 1; }
    }

    fn get_address_mode(&mut self, addr_mode: &AddrMode, inst: u8) -> u16 {
        let cross_page = (CYCLES[inst as usize] & CYCLES_CROSS_MASK) != CYCLES_CROSS_MASK;
        match addr_mode {
            AddrMode::Imm => {
                let operand = self.pc;
                self.pc += 1;
                operand
            },
            AddrMode::Ind => {
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = (self.bus.read((addr & 0xFF00) | ((addr + 1) & 0x00FF)) as u16) * 0x100 | self.bus.read(addr) as u16;
                operand
            },
            AddrMode::Abs => {
                let operand = self.read_address(self.pc);
                self.pc += 2;
                operand
            },
            AddrMode::Zp => { 
                let operand = self.bus.read(self.pc) as u16;
                self.pc += 1;
                operand
            },
            AddrMode::ZpX => {
                let operand = (self.bus.read(self.pc) + self.x) as u16; 
                self.pc += 1;
                operand
            },
            AddrMode::ZpY => {
                let operand = (self.bus.read(self.pc) + self.y) as u16;
                self.pc += 1;
                operand
            },
            AddrMode::AbsX => {
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = addr + self.x as u16;
                if cross_page { self.set_page_crossed(addr, operand) }
                operand
            },
            AddrMode::AbsY => {
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let operand = addr + self.y as u16;
                if cross_page { self.set_page_crossed(addr, operand) }
                operand
            },
            AddrMode::IndX => {
                let arg = (self.bus.read(self.pc) + self.x) as u16;
                self.pc += 1;
                self.bus.read(arg & 0xFF) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100
            },
            AddrMode::IndrY => {
                let arg = self.bus.read(self.pc) as u16;
                self.pc += 1;
                let addr = self.bus.read(arg) as u16 | (self.bus.read((arg + 1) & 0xFF) as u16) * 0x100;
                let operand = addr + self.y as u16;
                if cross_page { self.set_page_crossed(addr, operand) }
                operand
            },
            AddrMode::ZpInd => {
                let addr = self.bus.read(self.pc);
                self.pc += 1;
                let index = if inst == 0xB6 || inst == 0x96 || inst == 0x97 || inst == 0xB7 { self.y } else { self.x };
                (addr + index) as u16
            },
            AddrMode::AbsInd => {
                let addr = self.read_address(self.pc);
                self.pc += 2;
                let index = if inst == 0xBE || inst == 0x9E || inst == 0x9F || inst == 0xBF { self.y } else { self.x } as u16;
                let operand = addr + index;
                if cross_page { self.set_page_crossed(addr, operand) }
                operand
            },
            _ => 0
        }
    }

    fn add(&mut self, value: u8) {
        let carry = self.status.bits() & 0x1 == 1;
        let (sum, carry) = add(self.a, value, carry);
        self.status.set_carry(carry);
        self.status.set_overflow(((self.a ^ sum) & (value ^ sum) & 0x80) != 0);
        self.a = sum;
        self.status.set_zn(self.a);
    }

    fn operation1(&mut self, opcode: u8) -> bool {
        use Operation1::*;
        if opcode & OP_MASK == 1 {
            let addr_mode = &ADDR[((opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT) as usize];
            let inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let value = self.get_address_mode(addr_mode, opcode);
            let inst = match Operation1::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };

            match inst {
                ORA => {
                    self.a |= self.bus.read(value);
                    self.status.set_zn(self.a);
                },
                AND => {
                    self.a &= self.bus.read(value);
                    self.status.set_zn(self.a);
                },
                EOR => {
                    self.a ^= self.bus.read(value);
                    self.status.set_zn(self.a);
                },
                ADC => {
                    let value = self.bus.read(value);
                    self.add(value);
                },
                STA => self.bus.write(value, self.a),
                LDA => {
                    self.a = self.bus.read(value);
                    self.status.set_zn(self.a);
                },
                CMP => {
                    let value = self.bus.read(value);
                    let diff = self.a - value;
                    self.status.set_carry(self.a >= value);
                    self.status.set_zn(diff);
                },
                SBC => {
                    let value = self.bus.read(value);
                    self.add(!value);
                },
            };
            return true
        }
        false
    }

    fn operation2(&mut self, opcode: u8) -> bool {
        use Operation2::*;
        if opcode & OP_MASK == 2 {
            let mut addr_mode = &AddrMode::None;
            let oper_is_a = (opcode & 0x0F) == 0x0A;
            if !oper_is_a { addr_mode = &ADDR[((opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT) as usize]; }
            let value = self.get_address_mode(addr_mode, opcode);
            let mut inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            if opcode & 0x0F == 0x02 { inst = 0x8 }
            if opcode == 0x9E { inst = 0x9 }
            let inst = match Operation2::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            match inst {
                ASL => {
                    if oper_is_a {
                        self.status.set_carry((self.a & 0x80) > 0);
                        self.a <<= 1;
                        self.status.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.status.set_carry((operand & 0x80) > 0);
                        operand <<= 1;
                        self.status.set_zn(operand);
                        self.bus.write(value, operand);
                    }
                },
                ROL => {
                    if oper_is_a {
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((self.a & 0x80) > 0);
                        self.a = (self.a << 1) | carry;
                        self.status.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((operand & 0x80) > 0);
                        operand = (operand << 1) | carry;
                        self.status.set_zn(operand);
                        self.bus.write(value, operand);
                    }
                },
                LSR => {
                    if oper_is_a {
                        self.status.set_carry((self.a & 0x1) == 1);
                        self.a >>= 1;
                        self.status.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        self.status.set_carry((operand & 0x1) == 1);
                        operand >>= 1;
                        self.status.set_zn(operand);
                        self.bus.write(value, operand);
                    }
                },
                ROR => {
                    if oper_is_a {
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((self.a & 0x1) == 1);
                        self.a = (self.a >> 1) | carry << 7;
                        self.status.set_zn(self.a);
                    } else {
                        let mut operand = self.bus.read(value);
                        let carry = self.status.bits() & 0x1;
                        self.status.set_carry((operand & 0x1) == 1);
                        operand = (operand >> 1) | carry << 7;
                        self.status.set_zn(operand);
                        self.bus.write(value, operand);
                    }
                },
                STX => self.bus.write(value, self.x),
                LDX => {
                    self.x = self.bus.read(value);
                    self.status.set_zn(self.x);
                },
                DEC => {
                    let operand = self.bus.read(value) - 1;
                    self.bus.write(value, operand);
                    self.status.set_zn(operand);
                },
                INC => {
                    let operand = self.bus.read(value) + 1;
                    self.status.set_zn(operand);
                    self.bus.write(value, operand);
                },
                JAM => self.pc -= 1,
                SHX => ()
            }
            return true
        } 
        false
    }

    fn operation0(&mut self, opcode: u8) -> bool {
        use Operation0::*;
        if opcode & OP_MASK == 0 {
            let mut inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            let mut addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            if opcode == 0x20 { inst = 0; addr_mode = 3 }
            if opcode == 0x6C { addr_mode = 2 }
            if opcode == 0x9C { inst = 8 }
            let value = self.get_address_mode(&ADDR0[addr_mode as usize], opcode);
            let inst = match Operation0::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };

            match inst {
                JSR => {
                    let return_addr = self.pc - 1;
                    self.push_stack(((return_addr  & 0xFF00) >> 8) as u8);
                    self.push_stack((return_addr & 0x00FF) as u8);
                    self.pc = value;
                },
                BIT => {
                    let operand = self.bus.read(value);
                    self.status.set_negative(operand & 0x80 > 0);
                    self.status.set_overflow(operand & 0x40 > 0);
                    self.status.set_zero((operand & self.a) == 0);
                },
                JMP | _JMP => self.pc = value,
                STY => self.bus.write(value, self.y),
                LDY => {
                    self.y = self.bus.read(value);
                    self.status.set_zn(self.y);
                },
                CPY => {
                    let value = self.bus.read(value); 
                    let diff = self.y.wrapping_sub(value);
                    self.status.set_carry(self.y >= value);
                    self.status.set_zn(diff);
                },
                CPX => {
                    let value = self.bus.read(value); 
                    let diff = self.x - value;
                    self.status.set_carry(self.x >= value);
                    self.status.set_zn(diff);
                }
                SHY => ()
            }
            return true
        } 
        false
    }

    fn operation3(&mut self, opcode: u8) -> bool {
        use Operation3::*;
        if opcode & OP_MASK == 3 {
            let mut inst = (opcode & INST_MODE_MASK) >> INST_MODE_SHIFT;
            if opcode == 0xBB { inst = 0x8 }
            if opcode == 0x9F { inst = 0x9 }
            if opcode == 0x9B { inst = 0xA }
            let addr_mode = (opcode & ADDR_MODE_MASK) >> ADDR_MODE_SHIFT;
            let addr = self.get_address_mode(&ADDR[addr_mode as usize], opcode);
            let inst = match Operation3::try_from(inst) {
                Ok(op) => op,
                Err(_) => return false
            };
            match inst {
                SLO => {
                    let mut operand = self.bus.read(addr);
                    self.status.set_carry((operand & 0x80) > 0);
                    operand <<= 1;
                    self.bus.write(addr, operand);
                    self.a |= operand;
                    self.status.set_zn(self.a);
                },
                RLA => {
                    let mut operand = self.bus.read(addr);
                    let carry = self.status.bits() & 0x1;
                    self.status.set_carry((operand & 0x80) > 0);
                    operand = (operand << 1) | carry;
                    self.bus.write(addr, operand);
                    self.a &= operand;
                    self.status.set_zn(self.a);
                },
                SRE => {
                    let mut operand = self.bus.read(addr);
                    self.status.set_carry((operand & 0x1) == 1);
                    operand >>= 1;
                    self.bus.write(addr, operand);
                    self.a ^= operand;
                    self.status.set_zn(self.a);
                },
                RRA => {
                    let mut operand = self.bus.read(addr);
                    let carry = self.status.bits() & 0x1;
                    let carry_op = (operand & 0x1) == 1;
                    operand = (operand >> 1) | carry << 7;
                    self.bus.write(addr, operand);
                    let (sum, carry) = add(self.a, operand, carry_op);
                    self.status.set_carry(carry);
                    self.status.set_overflow(((self.a ^ sum) & (operand ^ sum) & 0x80) != 0);
                    self.a = sum;
                    self.status.set_zn(self.a);
                },
                SAX => {
                    let operand = self.bus.read(addr) & self.a;
                    self.bus.write(addr, operand);
                },
                LAX => {
                    self.a = self.bus.read(addr);
                    self.x = self.a;
                    self.status.set_zn(self.x);
                },
                DCP => {
                    let operand = self.bus.read(addr) - 1;
                    self.bus.write(addr, operand);
                    let diff = self.a - operand;
                    self.status.set_carry(self.a >= operand);
                    self.status.set_zn(diff);
                },
                ISC => {
                    let operand = self.bus.read(addr) + 1;
                    self.bus.write(addr, operand);
                    self.add(operand);
                },
                LAS => {
                    self.s = self.bus.read(addr) & self.s;
                    self.a = self.s;
                    self.x = self.s;
                    self.status.set_zn(self.s);
                },
                _ => () // Unstable
            }
            return true
        }
        false
    }

    fn execute_immediate(&mut self, opcode: u8) -> bool {
        use ImmediateOps::*;
        let inst = match ImmediateOps::try_from(opcode) {
            Ok(i) => i,
            _ => return false
        };
        let addr = self.get_address_mode(&AddrMode::Imm, opcode);
        let operand = self.bus.read(addr);
        match inst {
            LDX => {
                self.x = operand;
                self.status.set_zn(self.x);
            },
            ANC | ANC_ => {
                self.status.set_carry((operand & 0x80) > 0);
                self.status.set_zn(self.a & operand);
            },
            ALR => {
                self.a &= operand;
                self.status.set_carry((self.a & 0x1) == 1);
                self.a >>= 1;
                self.status.set_zn(self.a);
            },
            ARR => {
                let carry = self.status.bits() & 0x1 == 1;
                self.a &= operand;
                self.status.set_overflow(((self.a & 0x40) ^ ((self.a & 0x20) << 1)) > 0);
                self.status.set_carry((self.a & 0x40) > 0);
                self.status.set_zn(self.a);
                self.a = self.a >> 1 | (carry as u8) << 7;
            },
            SBX => {
                let value = self.x & self.a; 
                let (sum, carry) = add(!value, operand, false);
                self.x = sum;
                self.status.set_carry(carry);
                self.status.set_zn(self.x);
            },
            USBC => self.add(!operand), 
            _ => ()
        }
        true
    }

    fn execute_implied(&mut self, opcode: u8) -> bool {
        use ImplicitOps::*;
        let inst = match ImplicitOps::try_from(opcode) {
            Ok(i) => i,
            _ => return false
        };
        match inst {
            BRK => {
                let return_addr = self.pc + 1;
                self.push_stack((return_addr & 0xFF00 >> 8) as u8);
                self.push_stack((return_addr & 0x00FF) as u8);
                self.push_stack(self.status.bits() | 0x10);
                self.status.set_interrupt(true); 
                self.pc = self.read_address(IRQ_VECTOR);
            },
            TXA => {
                self.a = self.x;
                self.status.set_zn(self.a);
            },
            TAX => {
                self.x = self.a;
                self.status.set_zn(self.x);
            },
            TXS => self.s = self.x,
            DEX => {
                self.x -= 1;
                self.status.set_zn(self.x);
            },
            TSX => {
                self.x = self.s;
                self.status.set_zn(self.x);
            },
            RTI => {
                let value = self.pull_stack();
                self.status.update(value);
                self.pc = (self.pull_stack() as u16) | ((self.pull_stack() as u16) * 0x100);
            },
            RTS => self.pc = ((self.pull_stack() as u16) | ((self.pull_stack() as u16 ) * 0x100))+1,
            PHP => self.push_stack(self.status.bits() | 0x30),
            CLC => self.status.set_carry(false),
            PLP => { 
                let value = self.pull_stack();
                self.status.update(value)
            },
            SEC => self.status.set_carry(true),
            PHA => self.push_stack(self.a),
            CLI => self.status.set_interrupt(false),
            PLA => {
                self.a = self.pull_stack();
                self.status.set_zn(self.a);
            },
            SEI => self.status.set_interrupt(true),
            DEY => {
                self.y -= 1;
                self.status.set_zn(self.y);
            },
            TYA => {
                self.a = self.y;
                self.status.set_zn(self.a);
            },
            TAY => {
                self.y = self.a;
                self.status.set_zn(self.y);
            },
            CLV => self.status.set_overflow(false),
            INY => {
                self.y += 1;
                self.status.set_zn(self.y);
            },
            CLD => self.status.set_decimal(false),
            INX => {
                self.x += 1;
                self.status.set_zn(self.x);
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
