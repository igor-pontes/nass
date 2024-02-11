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

    fn relative(&mut self, opcode: u8) { 
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
                self.cycles_left += 1;
                self.set_page_crossed(old_pc, new_pc);
                self.pc = new_pc;
            } else { 
                self.pc += 1;
            }
        }
    }

    fn bpl(&mut self, _: u16) { self.relative(0x10); }
    fn bmi(&mut self, _: u16) { self.relative(0x30); }
    fn bvc(&mut self, _: u16) { self.relative(0x50); }
    fn bvs(&mut self, _: u16) { self.relative(0x70); }
    fn bcc(&mut self, _: u16) { self.relative(0x90); }
    fn bcs(&mut self, _: u16) { self.relative(0xB0); }
    fn bne(&mut self, _: u16) { self.relative(0xD0); }
    fn beq(&mut self, _: u16) { self.relative(0xF0); }

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

    pub const OPCODES: [(fn(&mut Self, u16) -> (), AddrMode) ; 0x100] = [
        (Self::brk, Impl(0x07)), (Self::ora,  IndX(0x06)), (Self::jam,      None), (Self::slo,  IndX(0x08)), (Self::nop, Zp(0x03)), (Self::ora,  Zp(0x03)), (Self::asl, Zp(0x05)), (Self::slo,  Zp(0x05)), 
        (Self::php, Impl(0x03)), (Self::ora,  Imm(0x02)), (Self::asl_a, Acc(0x02)), (Self::anc, Imm(0x02)), (Self::nop, Abs(0x04)), (Self::ora,  Abs(0x04)), (Self::asl, Abs(0x06)), (Self::slo,  Abs(0x06)),
        (Self::bpl,  Rel(0x02)), (Self::ora, IndrY(0x05)), (Self::jam,      None), (Self::slo, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::ora, ZpX(0x04)), (Self::asl, ZpX(0x06)), (Self::slo, ZpX(0x06)), 
        (Self::clc, Impl(0x02)), (Self::ora, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::slo, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::ora, AbsX(0x04)), (Self::asl, AbsX(0x87)), (Self::slo, AbsX(0x87)),
        (Self::jsr,  Abs(0x06)), (Self::and,  IndX(0x06)), (Self::jam,      None), (Self::rla, IndX(0x08)), (Self::bit, Zp(0x03)), (Self::and, Zp(0x03)), (Self::rol, Zp(0x05)), (Self::rla,  Zp(0x05)), 
        (Self::plp, Impl(0x04)), (Self::and,  Imm(0x02)), (Self::rol_a, Acc(0x02)), (Self::anc, Imm(0x02)), (Self::bit, Abs(0x04)), (Self::and, Abs(0x04)), (Self::rol, Abs(0x06)), (Self::rla,  Abs(0x06)),
        (Self::bmi,  Rel(0x02)), (Self::and,  IndX(0x05)), (Self::jam,      None), (Self::rla, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::and, ZpX(0x04)), (Self::rol, ZpX(0x06)), (Self::rla, ZpX(0x06)), 
        (Self::sec, Impl(0x02)), (Self::and, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::rla, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::and, AbsX(0x04)), (Self::rol, AbsX(0x87)), (Self::rla, AbsX(0x87)),
        (Self::rti, Impl(0x06)), (Self::eor,  IndX(0x06)), (Self::jam,      None), (Self::sre, IndX(0x08)), (Self::nop, Zp(0x03)), (Self::eor, Zp(0x03)), (Self::lsr, Zp(0x05)), (Self::sre,  Zp(0x05)), 
        (Self::pha, Impl(0x03)), (Self::eor,  Imm(0x02)), (Self::lsr_a, Acc(0x02)), (Self::alr, Imm(0x02)), (Self::jmp, Abs(0x03)), (Self::eor, Abs(0x03)), (Self::lsr, Abs(0x06)), (Self::sre,  Abs(0x06)),
        (Self::bvc,  Rel(0x02)), (Self::eor, IndrY(0x05)), (Self::jam,      None), (Self::sre, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::eor, ZpX(0x04)), (Self::lsr, ZpX(0x06)), (Self::sre, ZpX(0x06)), 
        (Self::cli, Impl(0x02)), (Self::eor, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::sre, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::eor, AbsX(0x04)), (Self::lsr, AbsX(0x87)), (Self::sre, AbsX(0x87)),
        (Self::rts, Impl(0x06)), (Self::adc,  IndX(0x06)), (Self::jam,      None), (Self::rra, IndX(0x08)), (Self::nop, Zp(0x03)), (Self::adc, Zp(0x03)), (Self::ror, Zp(0x05)), (Self::rra,  Zp(0x05)), 
        (Self::pla, Impl(0x04)), (Self::adc,  Imm(0x02)), (Self::ror_a, Acc(0x02)), (Self::arr, Imm(0x02)), (Self::jmp, Ind(0x05)), (Self::adc, Ind(0x05)), (Self::ror, Ind(0x06)), (Self::rra,  Ind(0x06)),
        (Self::bvs,  Rel(0x02)), (Self::adc, IndrY(0x05)), (Self::jam,      None), (Self::rra, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::adc, ZpX(0x04)), (Self::ror, ZpX(0x06)), (Self::rra, ZpX(0x06)), 
        (Self::sei, Impl(0x02)), (Self::adc, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::rra, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::adc, AbsX(0x04)), (Self::ror, AbsX(0x87)), (Self::rra, AbsX(0x87)),
        (Self::nop,  Imm(0x02)), (Self::sta,  IndX(0x06)), (Self::nop, Imm(0x02)), (Self::sax, IndX(0x06)), (Self::sty, Zp(0x03)), (Self::sta, Zp(0x03)), (Self::stx, Zp(0x03)), (Self::sax,  Zp(0x03)), 
        (Self::dey, Impl(0x02)), (Self::nop,  Imm(0x02)), (Self::txa, Impl(0x02)), (Self::nop, Imm(0x02)), (Self::sty,  Abs(0x04)), (Self::sta, Abs(0x84)), (Self::stx, Abs(0x04)), (Self::sax,  Abs(0x04)),
        (Self::bcc,  Rel(0x02)), (Self::sta, IndrY(0x86)), (Self::jam,      None), (Self::nop, IndrY(0x86)), (Self::sty,ZpX(0x04)), (Self::sta, ZpX(0x04)), (Self::stx, ZpY(0x04)), (Self::sax, ZpY(0x04)), 
        (Self::tya, Impl(0x02)), (Self::sta, AbsY(0x85)), (Self::txs, Impl(0x02)), (Self::nop, AbsY(0x85)), (Self::nop, AbsX(0x85)), (Self::sta, AbsX(0x85)), (Self::nop, AbsY(0x85)), (Self::nop, AbsY(0x85)),
        (Self::ldy,  Imm(0x02)), (Self::lda,  IndX(0x06)), (Self::ldx, Imm(0x02)), (Self::lax, IndX(0x06)), (Self::ldy, Zp(0x03)), (Self::lda, Zp(0x03)), (Self::ldx, Zp(0x03)), (Self::lax,  Zp(0x03)), 
        (Self::tay, Impl(0x02)), (Self::lda,  Imm(0x02)), (Self::tax, Impl(0x02)), (Self::nop, Imm(0x02)), (Self::ldy,  Abs(0x04)), (Self::lda, Abs(0x04)), (Self::ldx, Abs(0x04)), (Self::lax,  Abs(0x04)),
        (Self::bcs,  Rel(0x02)), (Self::lda, IndrY(0x05)), (Self::jam,      None), (Self::lax, IndrY(0x05)), (Self::ldy,ZpX(0x04)), (Self::lda, ZpX(0x04)), (Self::ldx, ZpY(0x04)), (Self::lax, ZpY(0x04)), 
        (Self::clv, Impl(0x02)), (Self::lda, AbsY(0x04)), (Self::tsx, Impl(0x02)), (Self::las, AbsY(0x04)), (Self::ldy, AbsX(0x04)), (Self::lda, AbsX(0x04)), (Self::ldx, AbsY(0x04)), (Self::lax, AbsY(0x04)),
        (Self::cpy,  Imm(0x02)), (Self::cmp,  IndX(0x06)), (Self::nop, Imm(0x02)), (Self::dcp, IndX(0x08)), (Self::cpy, Zp(0x03)), (Self::cmp, Zp(0x03)), (Self::dec, Zp(0x05)), (Self::dcp,  Zp(0x05)), 
        (Self::iny, Impl(0x02)), (Self::cmp,  Imm(0x02)), (Self::dex, Impl(0x02)), (Self::sbx, Imm(0x02)), (Self::cpy, Abs(0x04)), (Self::cmp, Abs(0x04)), (Self::dec, Abs(0x06)), (Self::dcp,  Abs(0x06)),
        (Self::bne,  Rel(0x02)), (Self::cmp, IndrY(0x05)), (Self::jam,      None), (Self::dcp, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::cmp, ZpX(0x04)), (Self::dec, ZpX(0x06)), (Self::dcp, ZpX(0x06)), 
        (Self::cld, Impl(0x02)), (Self::cmp, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::dcp, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::cmp, AbsX(0x04)), (Self::dec, AbsX(0x87)), (Self::dcp, AbsX(0x87)),
        (Self::cpx,  Imm(0x02)), (Self::sbc,  IndX(0x06)), (Self::nop, Imm(0x02)), (Self::isc, IndX(0x08)), (Self::cpx,  Zp(0x03)), (Self::sbc, Zp(0x03)), (Self::inc, Zp(0x05)), (Self::isc,  Zp(0x05)), 
        (Self::inx, Impl(0x02)), (Self::sbc,  Imm(0x02)), (Self::nop, Impl(0x02)), (Self::sbc, Imm(0x02)), (Self::cpx,  Abs(0x04)), (Self::sbc, Abs(0x04)), (Self::inc, Abs(0x06)), (Self::isc,  Abs(0x06)),
        (Self::beq,  Rel(0x02)), (Self::sbc,  IndX(0x05)), (Self::jam,      None), (Self::isc, IndrY(0x88)), (Self::nop, ZpX(0x04)), (Self::sbc, ZpX(0x04)), (Self::inc, ZpX(0x06)), (Self::isc, ZpX(0x06)), 
        (Self::sed, Impl(0x02)), (Self::sbc, AbsY(0x04)), (Self::nop, Impl(0x02)), (Self::isc, AbsY(0x87)), (Self::nop, AbsX(0x04)), (Self::sbc, AbsX(0x04)), (Self::inc, AbsX(0x87)), (Self::isc, AbsX(0x87)),
    ];
}


