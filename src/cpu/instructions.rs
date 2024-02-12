pub const NMI_VECTOR: u16 = 0xFFFA; 
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16 = 0xFFFE;

pub const CYCLE_MASK: usize = 0x7F;
// 0x80 = Intruction does not need to check if page is
// crossed if Address Mode requires it.
pub const CYCLE_PAGE_CROSS_MASK: u8 = 0x80; 

use AddrMode::*;
use crate::cpu::CPU;

#[derive(Clone, PartialEq)]
pub enum AddrMode { 
    Impl(usize),
    Acc(usize),
    Rel(usize),
    Imm(usize),
    Ind(usize),
    Zp(usize),
    ZpX(usize),
    ZpY(usize),
    Abs(usize),
    AbsX(usize),
    AbsY(usize),
    IndX(usize),
    IndrY(usize),
    None
}

impl CPU {

    fn relative(&mut self, cond: bool) { 
        if cond {
            let offset = self.bus.read(self.pc) as u16;
            self.pc += 1;
            let offset = if offset > 127 { 0xFFFF - (256 - offset) + 1 } else { offset };
            let new_pc = self.pc + offset;
            self.cycles_left += 1;
            self.set_page_crossed(self.pc, new_pc);
            self.pc = new_pc;
        } else { 
            self.pc += 1;
        }
    }

    fn bpl(&mut self, _: u16) { self.relative(!self.status.negative()); }
    fn bmi(&mut self, _: u16) { self.relative(self.status.negative()); }
    fn bvc(&mut self, _: u16) { self.relative(!self.status.overflow()); }
    fn bvs(&mut self, _: u16) { self.relative(self.status.overflow()); }
    fn bcc(&mut self, _: u16) { self.relative(!self.status.carry()); }
    fn bcs(&mut self, _: u16) { self.relative(self.status.carry()); }
    fn bne(&mut self, _: u16) { self.relative(!self.status.zero()); }
    fn beq(&mut self, _: u16) { self.relative(self.status.zero()); }

    fn jsr(&mut self, value: u16) { 
        let return_addr = self.pc - 1;
        self.push_stack(((return_addr  & 0xFF00) >> 8) as u8);
        self.push_stack((return_addr & 0x00FF) as u8);
        self.pc = value;
    }

    fn brk(&mut self, _: u16) {
        let return_addr = self.pc + 1;
        self.push_stack((return_addr & 0xFF00 >> 8) as u8);
        self.push_stack((return_addr & 0x00FF) as u8);
        self.push_stack(self.status.bits() | 0x10);
        self.status.set_interrupt(true); 
        self.pc = self.read_address(IRQ_VECTOR);
    }

    fn add(&mut self, value: u8) {
        let carry = self.status.bits() & 0x1 == 1;
        let (sum, carry) = self.a.carrying_add(value, carry);
        self.status.set_overflow(((self.a ^ sum) & (value ^ sum) & 0x80) != 0);
        self.a = sum;
        self.status.set_carry(carry);
        self.status.set_zn(self.a);
    }

    fn adc(&mut self, value: u16) {
        let value = self.bus.read(value);
        self.add(value);
    }

    fn rti(&mut self, _: u16) {
        let value = self.pull_stack();
        self.status.update(value);
        self.pc = (self.pull_stack() as u16) | ((self.pull_stack() as u16) * 0x100);
    }

    fn rts(&mut self, _: u16) {
        self.pc = ((self.pull_stack() as u16) | ((self.pull_stack() as u16 ) * 0x100))+1
    }

    fn ldy(&mut self, value: u16) {
        self.y = self.bus.read(value);
        self.status.set_zn(self.y);
    }

    fn cpy(&mut self, value: u16) {
        let value = self.bus.read(value); 
        let diff = self.y.wrapping_sub(value);
        self.status.set_carry(self.y >= value);
        self.status.set_zn(diff);
    }

    fn cpx(&mut self, value: u16) {
        let value = self.bus.read(value); 
        let diff = self.x - value;
        self.status.set_carry(self.x >= value);
        self.status.set_zn(diff);
    }

    fn ora(&mut self, value: u16) {
        self.a |= self.bus.read(value);
        self.status.set_zn(self.a);
    }

    fn cmp(&mut self, value: u16) {
        let value = self.bus.read(value);
        let diff = self.a - value;
        self.status.set_carry(self.a >= value);
        self.status.set_zn(diff);
    }

    fn eor(&mut self, value: u16) {
        self.a ^= self.bus.read(value);
        self.status.set_zn(self.a);
    }

    fn sbc(&mut self, value: u16) {
        let value = self.bus.read(value);
        self.add(!value);
    }

    fn sta(&mut self, value: u16) {
        self.bus.write(value, self.a)
    }

    fn lda(&mut self, value: u16) {
        self.a = self.bus.read(value);
        self.status.set_zn(self.a);
    }

    fn jam(&mut self, _: u16) { self.pc -= 1 }

    fn nop(&mut self, _: u16) { }

    fn slo(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        self.status.set_carry((operand & 0x80) > 0);
        operand <<= 1;
        self.bus.write(value, operand);
        self.a |= operand;
        self.status.set_zn(self.a);
    }

    fn rla(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        let carry = self.status.bits() & 0x1;
        self.status.set_carry((operand & 0x80) > 0);
        operand = (operand << 1) | carry;
        self.bus.write(value, operand);
        self.a &= operand;
        self.status.set_zn(self.a);
    }

    fn sre(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        self.status.set_carry((operand & 0x1) == 1);
        operand >>= 1;
        self.bus.write(value, operand);
        self.a ^= operand;
        self.status.set_zn(self.a);
    }

    fn rra(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        let carry = self.status.bits() & 0x1;
        let carry_op = (operand & 0x1) == 1;
        operand = (operand >> 1) | carry << 7;
        self.bus.write(value, operand);
        let (sum, carry) = self.a.carrying_add(operand, carry_op);
        self.status.set_carry(carry);
        self.status.set_overflow(((self.a ^ sum) & (operand ^ sum) & 0x80) != 0);
        self.a = sum;
        self.status.set_zn(self.a);
    }

    fn sax(&mut self, value: u16) {
        let operand = self.bus.read(value) & self.a;
        self.bus.write(value, operand);
    }

    fn lax(&mut self, value: u16) {
        self.a = self.bus.read(value);
        self.x = self.a;
        self.status.set_zn(self.x);
    }

    fn dcp(&mut self, value: u16) {
        let operand = self.bus.read(value) - 1;
        self.bus.write(value, operand);
        let diff = self.a - operand;
        self.status.set_carry(self.a >= operand);
        self.status.set_zn(diff);
    }

    fn isc(&mut self, value: u16) {
        let operand = self.bus.read(value) + 1;
        self.bus.write(value, operand);
        self.add(operand);
    }

    fn las(&mut self, value: u16) {
        self.s = self.bus.read(value) & self.s;
        self.a = self.s;
        self.x = self.s;
        self.status.set_zn(self.s);
    }

    fn asl_a(&mut self, _: u16) {
        self.status.set_carry((self.a & 0x80) > 0);
        self.a <<= 1;
        self.status.set_zn(self.a);
    }
    fn asl(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        self.status.set_carry((operand & 0x80) > 0);
        operand <<= 1;
        self.bus.write(value, operand);
        self.status.set_zn(operand);
    }

    fn rol_a(&mut self, _: u16) {
        let carry = self.status.bits() & 0x1;
        self.status.set_carry((self.a & 0x80) > 0);
        self.a = (self.a << 1) | carry;
        self.status.set_zn(self.a);
    }
    fn rol(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        let carry = self.status.bits() & 0x1;
        self.status.set_carry((operand & 0x80) > 0);
        operand = (operand << 1) | carry;
        self.status.set_zn(operand);
        self.bus.write(value, operand);
    }

    fn lsr_a(&mut self, _: u16) {
        self.status.set_carry((self.a & 0x1) == 1);
        self.a >>= 1;
        self.status.set_zn(self.a);
    }
    fn lsr(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        self.status.set_carry((operand & 0x1) == 1);
        operand >>= 1;
        self.status.set_zn(operand);
        self.bus.write(value, operand);
    }

    fn ror_a(&mut self, _: u16) {
        let carry = self.status.bits() & 0x1;
        self.status.set_carry((self.a & 0x1) == 1);
        self.a = (self.a >> 1) | carry << 7;
        self.status.set_zn(self.a);
    }
    fn ror(&mut self, value: u16) {
        let mut operand = self.bus.read(value);
        let carry = self.status.bits() & 0x1;
        self.status.set_carry((operand & 0x1) == 1);
        operand = (operand >> 1) | carry << 7;
        self.status.set_zn(operand);
        self.bus.write(value, operand);
    }

    fn stx(&mut self, value: u16) {
        self.bus.write(value, self.x)
    }

    fn ldx(&mut self, value: u16) {
        self.x = self.bus.read(value);
        self.status.set_zn(self.x);
    }

    fn and(&mut self, value: u16) {
        self.a &= self.bus.read(value);
        self.status.set_zn(self.a);
    }

    fn dec(&mut self, value: u16) {
        let operand = self.bus.read(value) - 1;
        self.bus.write(value, operand);
        self.status.set_zn(operand);
    }

    fn inc(&mut self, value: u16) {
        let operand = self.bus.read(value) + 1;
        self.status.set_zn(operand);
        self.bus.write(value, operand);
    }

    fn txa(&mut self, _: u16) {
        self.a = self.x;
        self.status.set_zn(self.a);
    }

    fn tax(&mut self, _: u16) {
        self.x = self.a;
        self.status.set_zn(self.x);
    }

    fn txs(&mut self, _: u16) {
        self.s = self.x
    }
    fn dex(&mut self, _: u16) {
        self.x -= 1;
        self.status.set_zn(self.x);
    }

    fn tsx(&mut self, _: u16) {
        self.x = self.s;
        self.status.set_zn(self.x);
    }

    fn php(&mut self, _: u16) { self.push_stack(self.status.bits() | 0x30) }
    fn clc(&mut self, _: u16) { self.status.set_carry(false) }

    fn plp(&mut self, _: u16) { 
        let value = self.pull_stack();
        self.status.update(value)
    }

    fn sec(&mut self, _: u16) { self.status.set_carry(true) }
    fn pha(&mut self, _: u16) { self.push_stack(self.a) }
    fn cli(&mut self, _: u16) { self.status.set_interrupt(false) }

    fn pla(&mut self, _: u16) {
        self.a = self.pull_stack();
        self.status.set_zn(self.a);
    }

    fn sei(&mut self, _: u16) { self.status.set_interrupt(true) }

    fn dey(&mut self, _: u16) {
        self.y -= 1;
        self.status.set_zn(self.y);
    }

    fn tya(&mut self, _: u16) {
        self.a = self.y;
        self.status.set_zn(self.a);
    }

    fn tay(&mut self, _: u16) {
        self.y = self.a;
        self.status.set_zn(self.y);
    }

    fn clv(&mut self, _: u16) { self.status.set_overflow(false) }

    fn iny(&mut self, _: u16) {
        self.y += 1;
        self.status.set_zn(self.y);
    }

    fn cld(&mut self, _: u16) { self.status.set_decimal(false) }

    fn inx(&mut self, _: u16) {
        self.x += 1;
        self.status.set_zn(self.x);
    }

    fn sed(&mut self, _: u16) { self.status.set_decimal(true) }

    fn jmp(&mut self, value: u16) { self.pc = value }

    fn sty(&mut self, value: u16) { self.bus.write(value, self.y) }

    fn bit(&mut self, value: u16) {
        let operand = self.bus.read(value);
        self.status.set_negative(operand & 0x80 > 0);
        self.status.set_overflow(operand & 0x40 > 0);
        self.status.set_zero((operand & self.a) == 0);
    }

    fn anc(&mut self, value: u16) {
        let operand = self.bus.read(value);
        self.status.set_carry((operand & 0x80) > 0);
        self.status.set_zn(self.a & operand);
    }

    fn alr(&mut self, value: u16) {
        let operand = self.bus.read(value);
        self.a &= operand;
        self.status.set_carry((self.a & 0x1) == 1);
        self.a >>= 1;
        self.status.set_zn(self.a);
    }

    fn arr(&mut self, value: u16) {
        let operand = self.bus.read(value);
        let carry = self.status.bits() & 0x1 == 1;
        self.a &= operand;
        self.status.set_overflow(((self.a & 0x40) ^ ((self.a & 0x20) << 1)) > 0);
        self.status.set_carry((self.a & 0x40) > 0);
        self.status.set_zn(self.a);
        self.a = self.a >> 1 | (carry as u8) << 7;
    }

    fn sbx(&mut self, value: u16) {
        let operand = self.bus.read(value);
        let value = self.x & self.a; 
        let (sum, carry) = (!value).carrying_add(operand, false);
        self.x = sum;
        self.status.set_carry(carry);
        self.status.set_zn(self.x);
    }

    pub const OPCODES: [(fn(&mut CPU, u16) -> (), AddrMode) ; 0x100] = [
        (CPU::brk, Impl(0x07)), (CPU::ora,  IndX(0x06)), (CPU::jam,        None), (CPU::slo,  IndX(0x08)), (CPU::nop,   Zp(0x03)), (CPU::ora,   Zp(0x03)), (CPU::asl,   Zp(0x05)), (CPU::slo,   Zp(0x05)), 
        (CPU::php, Impl(0x03)), (CPU::ora,   Imm(0x02)), (CPU::asl_a, Acc(0x02)), (CPU::anc,   Imm(0x02)), (CPU::nop,  Abs(0x04)), (CPU::ora,  Abs(0x04)), (CPU::asl,  Abs(0x06)), (CPU::slo,  Abs(0x06)),
        (CPU::bpl,  Rel(0x02)), (CPU::ora, IndrY(0x05)), (CPU::jam,        None), (CPU::slo, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::ora,  ZpX(0x04)), (CPU::asl,  ZpX(0x06)), (CPU::slo,  ZpX(0x06)), 
        (CPU::clc, Impl(0x02)), (CPU::ora,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::slo,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::ora, AbsX(0x04)), (CPU::asl, AbsX(0x87)), (CPU::slo, AbsX(0x87)),
        (CPU::jsr,  Abs(0x06)), (CPU::and,  IndX(0x06)), (CPU::jam,        None), (CPU::rla,  IndX(0x08)), (CPU::bit,   Zp(0x03)), (CPU::and,   Zp(0x03)), (CPU::rol,   Zp(0x05)), (CPU::rla,   Zp(0x05)), 
        (CPU::plp, Impl(0x04)), (CPU::and,   Imm(0x02)), (CPU::rol_a, Acc(0x02)), (CPU::anc,   Imm(0x02)), (CPU::bit,  Abs(0x04)), (CPU::and,  Abs(0x04)), (CPU::rol,  Abs(0x06)), (CPU::rla,  Abs(0x06)),
        (CPU::bmi,  Rel(0x02)), (CPU::and, IndrY(0x05)), (CPU::jam,        None), (CPU::rla, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::and,  ZpX(0x04)), (CPU::rol,  ZpX(0x06)), (CPU::rla,  ZpX(0x06)), 
        (CPU::sec, Impl(0x02)), (CPU::and,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::rla,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::and, AbsX(0x04)), (CPU::rol, AbsX(0x87)), (CPU::rla, AbsX(0x87)),
        (CPU::rti, Impl(0x06)), (CPU::eor,  IndX(0x06)), (CPU::jam,        None), (CPU::sre,  IndX(0x08)), (CPU::nop,   Zp(0x03)), (CPU::eor,   Zp(0x03)), (CPU::lsr,   Zp(0x05)), (CPU::sre,   Zp(0x05)), 
        (CPU::pha, Impl(0x03)), (CPU::eor,   Imm(0x02)), (CPU::lsr_a, Acc(0x02)), (CPU::alr,   Imm(0x02)), (CPU::jmp,  Abs(0x03)), (CPU::eor,  Abs(0x04)), (CPU::lsr,  Abs(0x06)), (CPU::sre,  Abs(0x06)),
        (CPU::bvc,  Rel(0x02)), (CPU::eor, IndrY(0x05)), (CPU::jam,        None), (CPU::sre, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::eor,  ZpX(0x04)), (CPU::lsr,  ZpX(0x06)), (CPU::sre,  ZpX(0x06)), 
        (CPU::cli, Impl(0x02)), (CPU::eor,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::sre,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::eor, AbsX(0x04)), (CPU::lsr, AbsX(0x87)), (CPU::sre, AbsX(0x87)),
        (CPU::rts, Impl(0x06)), (CPU::adc,  IndX(0x06)), (CPU::jam,        None), (CPU::rra,  IndX(0x08)), (CPU::nop,   Zp(0x03)), (CPU::adc,   Zp(0x03)), (CPU::ror,   Zp(0x05)), (CPU::rra,   Zp(0x05)), 
        (CPU::pla, Impl(0x04)), (CPU::adc,   Imm(0x02)), (CPU::ror_a, Acc(0x02)), (CPU::arr,   Imm(0x02)), (CPU::jmp,  Ind(0x05)), (CPU::adc,  Abs(0x04)), (CPU::ror,  Abs(0x06)), (CPU::rra,  Ind(0x06)),
        (CPU::bvs,  Rel(0x02)), (CPU::adc, IndrY(0x05)), (CPU::jam,        None), (CPU::rra, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::adc,  ZpX(0x04)), (CPU::ror,  ZpX(0x06)), (CPU::rra,  ZpX(0x06)), 
        (CPU::sei, Impl(0x02)), (CPU::adc,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::rra,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::adc, AbsX(0x04)), (CPU::ror, AbsX(0x87)), (CPU::rra, AbsX(0x87)),
        (CPU::nop,  Imm(0x02)), (CPU::sta,  IndX(0x06)), (CPU::nop,   Imm(0x02)), (CPU::sax,  IndX(0x06)), (CPU::sty,   Zp(0x03)), (CPU::sta,   Zp(0x03)), (CPU::stx,   Zp(0x03)), (CPU::sax,   Zp(0x03)), 
        (CPU::dey, Impl(0x02)), (CPU::nop,   Imm(0x02)), (CPU::txa,  Impl(0x02)), (CPU::nop,   Imm(0x02)), (CPU::sty,  Abs(0x04)), (CPU::sta,  Abs(0x84)), (CPU::stx,  Abs(0x04)), (CPU::sax,  Abs(0x04)),
        (CPU::bcc,  Rel(0x02)), (CPU::sta, IndrY(0x86)), (CPU::jam,        None), (CPU::nop, IndrY(0x86)), (CPU::sty,  ZpX(0x04)), (CPU::sta,  ZpX(0x04)), (CPU::stx,  ZpY(0x04)), (CPU::sax,  ZpY(0x04)), 
        (CPU::tya, Impl(0x02)), (CPU::sta,  AbsY(0x85)), (CPU::txs,  Impl(0x02)), (CPU::nop,  AbsY(0x85)), (CPU::nop, AbsX(0x85)), (CPU::sta, AbsX(0x85)), (CPU::nop, AbsY(0x85)), (CPU::nop, AbsY(0x85)),
        (CPU::ldy,  Imm(0x02)), (CPU::lda,  IndX(0x06)), (CPU::ldx,   Imm(0x02)), (CPU::lax,  IndX(0x06)), (CPU::ldy,   Zp(0x03)), (CPU::lda,   Zp(0x03)), (CPU::ldx,   Zp(0x03)), (CPU::lax,   Zp(0x03)), 
        (CPU::tay, Impl(0x02)), (CPU::lda,   Imm(0x02)), (CPU::tax,  Impl(0x02)), (CPU::nop,   Imm(0x02)), (CPU::ldy,  Abs(0x04)), (CPU::lda,  Abs(0x04)), (CPU::ldx,  Abs(0x04)), (CPU::lax,  Abs(0x04)),
        (CPU::bcs,  Rel(0x02)), (CPU::lda, IndrY(0x05)), (CPU::jam,        None), (CPU::lax, IndrY(0x05)), (CPU::ldy,  ZpX(0x04)), (CPU::lda,  ZpX(0x04)), (CPU::ldx,  ZpY(0x04)), (CPU::lax,  ZpY(0x04)), 
        (CPU::clv, Impl(0x02)), (CPU::lda,  AbsY(0x04)), (CPU::tsx,  Impl(0x02)), (CPU::las,  AbsY(0x04)), (CPU::ldy, AbsX(0x04)), (CPU::lda, AbsX(0x04)), (CPU::ldx, AbsY(0x04)), (CPU::lax, AbsY(0x04)),
        (CPU::cpy,  Imm(0x02)), (CPU::cmp,  IndX(0x06)), (CPU::nop,   Imm(0x02)), (CPU::dcp,  IndX(0x08)), (CPU::cpy,   Zp(0x03)), (CPU::cmp,   Zp(0x03)), (CPU::dec,   Zp(0x05)), (CPU::dcp,   Zp(0x05)), 
        (CPU::iny, Impl(0x02)), (CPU::cmp,   Imm(0x02)), (CPU::dex,  Impl(0x02)), (CPU::sbx,   Imm(0x02)), (CPU::cpy,  Abs(0x04)), (CPU::cmp,  Abs(0x04)), (CPU::dec,  Abs(0x06)), (CPU::dcp,  Abs(0x06)),
        (CPU::bne,  Rel(0x02)), (CPU::cmp, IndrY(0x05)), (CPU::jam,        None), (CPU::dcp, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::cmp,  ZpX(0x04)), (CPU::dec,  ZpX(0x06)), (CPU::dcp,  ZpX(0x06)), 
        (CPU::cld, Impl(0x02)), (CPU::cmp,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::dcp,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::cmp, AbsX(0x04)), (CPU::dec, AbsX(0x87)), (CPU::dcp, AbsX(0x87)),
        (CPU::cpx,  Imm(0x02)), (CPU::sbc,  IndX(0x06)), (CPU::nop,   Imm(0x02)), (CPU::isc,  IndX(0x08)), (CPU::cpx,   Zp(0x03)), (CPU::sbc,   Zp(0x03)), (CPU::inc,   Zp(0x05)), (CPU::isc,   Zp(0x05)), 
        (CPU::inx, Impl(0x02)), (CPU::sbc,   Imm(0x02)), (CPU::nop,  Impl(0x02)), (CPU::sbc,   Imm(0x02)), (CPU::cpx,  Abs(0x04)), (CPU::sbc,  Abs(0x04)), (CPU::inc,  Abs(0x06)), (CPU::isc,  Abs(0x06)),
        (CPU::beq,  Rel(0x02)), (CPU::sbc, IndrY(0x05)), (CPU::jam,        None), (CPU::isc, IndrY(0x88)), (CPU::nop,  ZpX(0x04)), (CPU::sbc,  ZpX(0x04)), (CPU::inc,  ZpX(0x06)), (CPU::isc,  ZpX(0x06)), 
        (CPU::sed, Impl(0x02)), (CPU::sbc,  AbsY(0x04)), (CPU::nop,  Impl(0x02)), (CPU::isc,  AbsY(0x87)), (CPU::nop, AbsX(0x04)), (CPU::sbc, AbsX(0x04)), (CPU::inc, AbsX(0x87)), (CPU::isc, AbsX(0x87)),
    ];
}
