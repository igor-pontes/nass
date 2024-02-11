use crate::cpu::CPU;

pub const NMI_VECTOR: u16 = 0xFFFA; 
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16 = 0xFFFE;

pub const CYCLE_MASK: usize = 0x7F;
// 0x80 = Intruction does not need to check if page is
// crossed if Address Mode requires it.
pub const CYCLE_PAGE_CROSS_MASK: u8 = 0x80; 

use AddrMode::*;

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

fn relative(cpu: &mut CPU, opcode: u8) { 
    let bits = cpu.status.bits();
    let status = [(0x80 & bits) >> 7, (0x40 & bits) >> 6, 0x01 & bits, (0x02 & bits) >> 1];
    if (opcode & 0x1F) == 0x10 {
        let inst = (opcode & 0xC0) >> 6;
        let cond = (opcode & 0x20) >> 5;
        if status[inst as usize] == cond {
            let offset = cpu.bus.read(cpu.pc) as i8;
            cpu.pc += 1;
            let old_pc = cpu.pc;
            let (new_pc, _) = old_pc.overflowing_add_signed(offset as i16);
            cpu.cycles_left += 1;
            cpu.set_page_crossed(old_pc, new_pc);
            cpu.pc = new_pc;
        } else { 
            cpu.pc += 1;
        }
    }
}

fn bpl(cpu: &mut CPU, _: u16) { relative(cpu, 0x10); }
fn bmi(cpu: &mut CPU, _: u16) { relative(cpu, 0x30); }
fn bvc(cpu: &mut CPU, _: u16) { relative(cpu, 0x50); }
fn bvs(cpu: &mut CPU, _: u16) { relative(cpu, 0x70); }
fn bcc(cpu: &mut CPU, _: u16) { relative(cpu, 0x90); }
fn bcs(cpu: &mut CPU, _: u16) { relative(cpu, 0xB0); }
fn bne(cpu: &mut CPU, _: u16) { relative(cpu, 0xD0); }
fn beq(cpu: &mut CPU, _: u16) { relative(cpu, 0xF0); }

fn jsr(cpu: &mut CPU, value: u16) { 
    let return_addr = cpu.pc - 1;
    cpu.push_stack(((return_addr  & 0xFF00) >> 8) as u8);
    cpu.push_stack((return_addr & 0x00FF) as u8);
    cpu.pc = value;
}

fn brk(cpu: &mut CPU, _: u16) {
    let return_addr = cpu.pc + 1;
    cpu.push_stack((return_addr & 0xFF00 >> 8) as u8);
    cpu.push_stack((return_addr & 0x00FF) as u8);
    cpu.push_stack(cpu.status.bits() | 0x10);
    cpu.status.set_interrupt(true); 
    cpu.pc = cpu.read_address(IRQ_VECTOR);
}


fn add(cpu: &mut CPU, value: u8) {
    let carry = cpu.status.bits() & 0x1 == 1;
    let (sum, carry) = cpu.a.carrying_add(value, carry);
    cpu.status.set_overflow(((cpu.a ^ sum) & (value ^ sum) & 0x80) != 0);
    cpu.a = sum;
    cpu.status.set_carry(carry);
    cpu.status.set_zn(cpu.a);
}

fn adc(cpu: &mut CPU, value: u16) {
    let value = cpu.bus.read(value);
    add(cpu, value);
}

fn rti(cpu: &mut CPU, _: u16) {
    let value = cpu.pull_stack();
    cpu.status.update(value);
    cpu.pc = (cpu.pull_stack() as u16) | ((cpu.pull_stack() as u16) * 0x100);
}

fn rts(cpu: &mut CPU, _: u16) {
    cpu.pc = ((cpu.pull_stack() as u16) | ((cpu.pull_stack() as u16 ) * 0x100))+1
}

fn ldy(cpu: &mut CPU, value: u16) {
    cpu.y = cpu.bus.read(value);
    cpu.status.set_zn(cpu.y);
}

fn cpy(cpu: &mut CPU, value: u16) {
    let value = cpu.bus.read(value); 
    let diff = cpu.y.wrapping_sub(value);
    cpu.status.set_carry(cpu.y >= value);
    cpu.status.set_zn(diff);
}

fn cpx(cpu: &mut CPU, value: u16) {
    let value = cpu.bus.read(value); 
    let diff = cpu.x - value;
    cpu.status.set_carry(cpu.x >= value);
    cpu.status.set_zn(diff);
}

fn ora(cpu: &mut CPU, value: u16) {
    cpu.a |= cpu.bus.read(value);
    cpu.status.set_zn(cpu.a);
}

fn cmp(cpu: &mut CPU, value: u16) {
    let value = cpu.bus.read(value);
    let diff = cpu.a - value;
    cpu.status.set_carry(cpu.a >= value);
    cpu.status.set_zn(diff);
}

fn eor(cpu: &mut CPU, value: u16) {
    cpu.a ^= cpu.bus.read(value);
    cpu.status.set_zn(cpu.a);
}

fn sbc(cpu: &mut CPU, value: u16) {
    let value = cpu.bus.read(value);
    add(cpu, !value);
}

fn sta(cpu: &mut CPU, value: u16) {
    cpu.bus.write(value, cpu.a)
}

fn lda(cpu: &mut CPU, value: u16) {
    cpu.a = cpu.bus.read(value);
    cpu.status.set_zn(cpu.a);
}

fn jam(cpu: &mut CPU, _: u16) { cpu.pc -= 1 }

fn nop(_: &mut CPU, _: u16) { }

fn slo(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    cpu.status.set_carry((operand & 0x80) > 0);
    operand <<= 1;
    cpu.bus.write(value, operand);
    cpu.a |= operand;
    cpu.status.set_zn(cpu.a);
}

fn rla(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    let carry = cpu.status.bits() & 0x1;
    cpu.status.set_carry((operand & 0x80) > 0);
    operand = (operand << 1) | carry;
    cpu.bus.write(value, operand);
    cpu.a &= operand;
    cpu.status.set_zn(cpu.a);
}

fn sre(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    cpu.status.set_carry((operand & 0x1) == 1);
    operand >>= 1;
    cpu.bus.write(value, operand);
    cpu.a ^= operand;
    cpu.status.set_zn(cpu.a);
}

fn rra(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    let carry = cpu.status.bits() & 0x1;
    let carry_op = (operand & 0x1) == 1;
    operand = (operand >> 1) | carry << 7;
    cpu.bus.write(value, operand);
    let (sum, carry) = cpu.a.carrying_add(operand, carry_op);
    cpu.status.set_carry(carry);
    cpu.status.set_overflow(((cpu.a ^ sum) & (operand ^ sum) & 0x80) != 0);
    cpu.a = sum;
    cpu.status.set_zn(cpu.a);
}

fn sax(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value) & cpu.a;
    cpu.bus.write(value, operand);
}

fn lax(cpu: &mut CPU, value: u16) {
    cpu.a = cpu.bus.read(value);
    cpu.x = cpu.a;
    cpu.status.set_zn(cpu.x);
}

fn dcp(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value) - 1;
    cpu.bus.write(value, operand);
    let diff = cpu.a - operand;
    cpu.status.set_carry(cpu.a >= operand);
    cpu.status.set_zn(diff);
}

fn isc(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value) + 1;
    cpu.bus.write(value, operand);
    add(cpu, operand);
}

fn las(cpu: &mut CPU, value: u16) {
    cpu.s = cpu.bus.read(value) & cpu.s;
    cpu.a = cpu.s;
    cpu.x = cpu.s;
    cpu.status.set_zn(cpu.s);
}

fn asl_a(cpu: &mut CPU, _: u16) {
    cpu.status.set_carry((cpu.a & 0x80) > 0);
    cpu.a <<= 1;
    cpu.status.set_zn(cpu.a);
}
fn asl(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    cpu.status.set_carry((operand & 0x80) > 0);
    operand <<= 1;
    cpu.bus.write(value, operand);
    cpu.status.set_zn(operand);
}

fn rol_a(cpu: &mut CPU, _: u16) {
    let carry = cpu.status.bits() & 0x1;
    cpu.status.set_carry((cpu.a & 0x80) > 0);
    cpu.a = (cpu.a << 1) | carry;
    cpu.status.set_zn(cpu.a);
}
fn rol(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    let carry = cpu.status.bits() & 0x1;
    cpu.status.set_carry((operand & 0x80) > 0);
    operand = (operand << 1) | carry;
    cpu.status.set_zn(operand);
    cpu.bus.write(value, operand);
}

fn lsr_a(cpu: &mut CPU, _: u16) {
    cpu.status.set_carry((cpu.a & 0x1) == 1);
    cpu.a >>= 1;
    cpu.status.set_zn(cpu.a);
}
fn lsr(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    cpu.status.set_carry((operand & 0x1) == 1);
    operand >>= 1;
    cpu.status.set_zn(operand);
    cpu.bus.write(value, operand);
}

fn ror_a(cpu: &mut CPU, _: u16) {
    let carry = cpu.status.bits() & 0x1;
    cpu.status.set_carry((cpu.a & 0x1) == 1);
    cpu.a = (cpu.a >> 1) | carry << 7;
    cpu.status.set_zn(cpu.a);
}
fn ror(cpu: &mut CPU, value: u16) {
    let mut operand = cpu.bus.read(value);
    let carry = cpu.status.bits() & 0x1;
    cpu.status.set_carry((operand & 0x1) == 1);
    operand = (operand >> 1) | carry << 7;
    cpu.status.set_zn(operand);
    cpu.bus.write(value, operand);
}

fn stx(cpu: &mut CPU, value: u16) {
    cpu.bus.write(value, cpu.x)
}

fn ldx(cpu: &mut CPU, value: u16) {
    cpu.x = cpu.bus.read(value);
    cpu.status.set_zn(cpu.x);
}

fn and(cpu: &mut CPU, value: u16) {
    cpu.a &= cpu.bus.read(value);
    cpu.status.set_zn(cpu.a);
}

fn dec(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value) - 1;
    cpu.bus.write(value, operand);
    cpu.status.set_zn(operand);
}

fn inc(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value) + 1;
    cpu.status.set_zn(operand);
    cpu.bus.write(value, operand);
}

fn txa(cpu: &mut CPU, _: u16) {
    cpu.a = cpu.x;
    cpu.status.set_zn(cpu.a);
}

fn tax(cpu: &mut CPU, _: u16) {
    cpu.x = cpu.a;
    cpu.status.set_zn(cpu.x);
}

fn txs(cpu: &mut CPU, _: u16) {
    cpu.s = cpu.x
}
fn dex(cpu: &mut CPU, _: u16) {
    cpu.x -= 1;
    cpu.status.set_zn(cpu.x);
}

fn tsx(cpu: &mut CPU, _: u16) {
    cpu.x = cpu.s;
    cpu.status.set_zn(cpu.x);
}

fn php(cpu: &mut CPU, _: u16) { cpu.push_stack(cpu.status.bits() | 0x30) }
fn clc(cpu: &mut CPU, _: u16) { cpu.status.set_carry(false) }

fn plp(cpu: &mut CPU, _: u16) { 
    let value = cpu.pull_stack();
    cpu.status.update(value)
}

fn sec(cpu: &mut CPU, _: u16) { cpu.status.set_carry(true) }
fn pha(cpu: &mut CPU, _: u16) { cpu.push_stack(cpu.a) }
fn cli(cpu: &mut CPU, _: u16) { cpu.status.set_interrupt(false) }

fn pla(cpu: &mut CPU, _: u16) {
    cpu.a = cpu.pull_stack();
    cpu.status.set_zn(cpu.a);
}

fn sei(cpu: &mut CPU, _: u16) { cpu.status.set_interrupt(true) }

fn dey(cpu: &mut CPU, _: u16) {
    cpu.y -= 1;
    cpu.status.set_zn(cpu.y);
}

fn tya(cpu: &mut CPU, _: u16) {
    cpu.a = cpu.y;
    cpu.status.set_zn(cpu.a);
}

fn tay(cpu: &mut CPU, _: u16) {
    cpu.y = cpu.a;
    cpu.status.set_zn(cpu.y);
}

fn clv(cpu: &mut CPU, _: u16) { cpu.status.set_overflow(false) }

fn iny(cpu: &mut CPU, _: u16) {
    cpu.y += 1;
    cpu.status.set_zn(cpu.y);
}

fn cld(cpu: &mut CPU, _: u16) { cpu.status.set_decimal(false) }

fn inx(cpu: &mut CPU, _: u16) {
    cpu.x += 1;
    cpu.status.set_zn(cpu.x);
}

fn sed(cpu: &mut CPU, _: u16) { cpu.status.set_decimal(true) }


fn jmp(cpu: &mut CPU, value: u16) { cpu.pc = value }

fn sty(cpu: &mut CPU, value: u16) { cpu.bus.write(value, cpu.y) }

fn bit(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value);
    cpu.status.set_negative(operand & 0x80 > 0);
    cpu.status.set_overflow(operand & 0x40 > 0);
    cpu.status.set_zero((operand & cpu.a) == 0);
}


fn anc(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value);
    cpu.status.set_carry((operand & 0x80) > 0);
    cpu.status.set_zn(cpu.a & operand);
}

fn alr(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value);
    cpu.a &= operand;
    cpu.status.set_carry((cpu.a & 0x1) == 1);
    cpu.a >>= 1;
    cpu.status.set_zn(cpu.a);
}

fn arr(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value);
    let carry = cpu.status.bits() & 0x1 == 1;
    cpu.a &= operand;
    cpu.status.set_overflow(((cpu.a & 0x40) ^ ((cpu.a & 0x20) << 1)) > 0);
    cpu.status.set_carry((cpu.a & 0x40) > 0);
    cpu.status.set_zn(cpu.a);
    cpu.a = cpu.a >> 1 | (carry as u8) << 7;
}

fn sbx(cpu: &mut CPU, value: u16) {
    let operand = cpu.bus.read(value);
    let value = cpu.x & cpu.a; 
    let (sum, carry) = (!value).carrying_add(operand, false);
    cpu.x = sum;
    cpu.status.set_carry(carry);
    cpu.status.set_zn(cpu.x);
}



pub const OPCODES: [(fn(&mut CPU, u16) -> (), AddrMode) ; 0x100] = [
    (brk, Impl(0x07)), (ora,  IndX(0x06)), (jam,      None), (slo,  IndX(0x08)), (nop,  Zp(0x03)), (ora,  Zp(0x03)), (asl,  Zp(0x05)), (slo,  Zp(0x05)), 
    (php, Impl(0x03)), (ora,  Imm(0x02)), (asl_a, Acc(0x02)), (anc,  Imm(0x02)), (nop,  Abs(0x04)), (ora,  Abs(0x04)), (asl,  Abs(0x06)), (slo,  Abs(0x06)),
    (bpl,  Rel(0x02)), (ora, IndrY(0x05)), (jam,      None), (slo, IndrY(0x88)), (nop, ZpX(0x04)), (ora, ZpX(0x04)), (asl, ZpX(0x06)), (slo, ZpX(0x06)), 
    (clc, Impl(0x02)), (ora, AbsY(0x04)), (nop, Impl(0x02)), (slo, AbsY(0x87)), (nop, AbsX(0x04)), (ora, AbsX(0x04)), (asl, AbsX(0x87)), (slo, AbsX(0x87)),
    (jsr,  Abs(0x06)), (and,  IndX(0x06)), (jam,      None), (rla,  IndX(0x08)), (bit,  Zp(0x03)), (and,  Zp(0x03)), (rol,  Zp(0x05)), (rla,  Zp(0x05)), 
    (plp, Impl(0x04)), (and,  Imm(0x02)), (rol_a, Acc(0x02)), (anc,  Imm(0x02)), (bit,  Abs(0x04)), (and,  Abs(0x04)), (rol,  Abs(0x06)), (rla,  Abs(0x06)),
    (bmi,  Rel(0x02)), (and,  IndX(0x05)), (jam,      None), (rla, IndrY(0x88)), (nop, ZpX(0x04)), (and, ZpX(0x04)), (rol, ZpX(0x06)), (rla, ZpX(0x06)), 
    (sec, Impl(0x02)), (and, AbsY(0x04)), (nop, Impl(0x02)), (rla, AbsY(0x87)), (nop, AbsX(0x04)), (and, AbsX(0x04)), (rol, AbsX(0x87)), (rla, AbsX(0x87)),
    (rti, Impl(0x06)), (eor,  IndX(0x06)), (jam,      None), (sre,  IndX(0x08)), (nop,  Zp(0x03)), (eor,  Zp(0x03)), (lsr,  Zp(0x05)), (sre,  Zp(0x05)), 
    (pha, Impl(0x03)), (eor,  Imm(0x02)), (lsr_a, Acc(0x02)), (alr,  Imm(0x02)), (jmp,  Abs(0x03)), (eor,  Abs(0x03)), (lsr,  Abs(0x06)), (sre,  Abs(0x06)),
    (bvc,  Rel(0x02)), (eor, IndrY(0x05)), (jam,      None), (sre, IndrY(0x88)), (nop, ZpX(0x04)), (eor, ZpX(0x04)), (lsr, ZpX(0x06)), (sre, ZpX(0x06)), 
    (cli, Impl(0x02)), (eor, AbsY(0x04)), (nop, Impl(0x02)), (sre, AbsY(0x87)), (nop, AbsX(0x04)), (eor, AbsX(0x04)), (lsr, AbsX(0x87)), (sre, AbsX(0x87)),
    (rts, Impl(0x06)), (adc,  IndX(0x06)), (jam,      None), (rra,  IndX(0x08)), (nop,  Zp(0x03)), (adc,  Zp(0x03)), (ror,  Zp(0x05)), (rra,  Zp(0x05)), 
    (pla, Impl(0x04)), (adc,  Imm(0x02)), (ror_a, Acc(0x02)), (arr,  Imm(0x02)), (jmp,  Ind(0x05)), (adc,  Ind(0x05)), (ror,  Ind(0x06)), (rra,  Ind(0x06)),
    (bvs,  Rel(0x02)), (adc, IndrY(0x05)), (jam,      None), (rra, IndrY(0x88)), (nop, ZpX(0x04)), (adc, ZpX(0x04)), (ror, ZpX(0x06)), (rra, ZpX(0x06)), 
    (sei, Impl(0x02)), (adc, AbsY(0x04)), (nop, Impl(0x02)), (rra, AbsY(0x87)), (nop, AbsX(0x04)), (adc, AbsX(0x04)), (ror, AbsX(0x87)), (rra, AbsX(0x87)),
    (nop,  Imm(0x02)), (sta,  IndX(0x06)), (nop, Imm(0x02)), (sax,  IndX(0x06)), (sty,  Zp(0x03)), (sta,  Zp(0x03)), (stx,  Zp(0x03)), (sax,  Zp(0x03)), 
    (dey, Impl(0x02)), (nop,  Imm(0x02)), (txa, Impl(0x02)), (nop,  Imm(0x02)), (sty,  Abs(0x04)), (sta,  Abs(0x84)), (stx,  Abs(0x04)), (sax,  Abs(0x04)),
    (bcc,  Rel(0x02)), (sta, IndrY(0x86)), (jam,      None), (nop, IndrY(0x86)), (sty, ZpX(0x04)), (sta, ZpX(0x04)), (stx, ZpY(0x04)), (sax, ZpY(0x04)), 
    (tya, Impl(0x02)), (sta, AbsY(0x85)), (txs, Impl(0x02)), (nop, AbsY(0x85)), (nop, AbsX(0x85)), (sta, AbsX(0x85)), (nop, AbsY(0x85)), (nop, AbsY(0x85)),
    (ldy,  Imm(0x02)), (lda,  IndX(0x06)), (ldx, Imm(0x02)), (lax,  IndX(0x06)), (ldy,  Zp(0x03)), (lda,  Zp(0x03)), (ldx,  Zp(0x03)), (lax,  Zp(0x03)), 
    (tay, Impl(0x02)), (lda,  Imm(0x02)), (tax, Impl(0x02)), (nop,  Imm(0x02)), (ldy,  Abs(0x04)), (lda,  Abs(0x04)), (ldx,  Abs(0x04)), (lax,  Abs(0x04)),
    (bcs,  Rel(0x02)), (lda, IndrY(0x05)), (jam,      None), (lax, IndrY(0x05)), (ldy, ZpX(0x04)), (lda, ZpX(0x04)), (ldx, ZpY(0x04)), (lax, ZpY(0x04)), 
    (clv, Impl(0x02)), (lda, AbsY(0x04)), (tsx, Impl(0x02)), (las, AbsY(0x04)), (ldy, AbsX(0x04)), (lda, AbsX(0x04)), (ldx, AbsY(0x04)), (lax, AbsY(0x04)),
    (cpy,  Imm(0x02)), (cmp,  IndX(0x06)), (nop, Imm(0x02)), (dcp,  IndX(0x08)), (cpy,  Zp(0x03)), (cmp,  Zp(0x03)), (dec,  Zp(0x05)), (dcp,  Zp(0x05)), 
    (iny, Impl(0x02)), (cmp,  Imm(0x02)), (dex, Impl(0x02)), (sbx,  Imm(0x02)), (cpy,  Abs(0x04)), (cmp,  Abs(0x04)), (dec,  Abs(0x06)), (dcp,  Abs(0x06)),
    (bne,  Rel(0x02)), (cmp, IndrY(0x05)), (jam,      None), (dcp, IndrY(0x88)), (nop, ZpX(0x04)), (cmp, ZpX(0x04)), (dec, ZpX(0x06)), (dcp, ZpX(0x06)), 
    (cld, Impl(0x02)), (cmp, AbsY(0x04)), (nop, Impl(0x02)), (dcp, AbsY(0x87)), (nop, AbsX(0x04)), (cmp, AbsX(0x04)), (dec, AbsX(0x87)), (dcp, AbsX(0x87)),
    (cpx,  Imm(0x02)), (sbc,  IndX(0x06)), (nop, Imm(0x02)), (isc,  IndX(0x08)), (cpx,  Zp(0x03)), (sbc,  Zp(0x03)), (inc,  Zp(0x05)), (isc,  Zp(0x05)), 
    (inx, Impl(0x02)), (sbc,  Imm(0x02)), (nop, Impl(0x02)), (sbc,  Imm(0x02)), (cpx,  Abs(0x04)), (sbc,  Abs(0x04)), (inc,  Abs(0x06)), (isc,  Abs(0x06)),
    (beq,  Rel(0x02)), (sbc,  IndX(0x05)), (jam,      None), (isc, IndrY(0x88)), (nop, ZpX(0x04)), (sbc, ZpX(0x04)), (inc, ZpX(0x06)), (isc, ZpX(0x06)), 
    (sed, Impl(0x02)), (sbc, AbsY(0x04)), (nop, Impl(0x02)), (isc, AbsY(0x87)), (nop, AbsX(0x04)), (sbc, AbsX(0x04)), (inc, AbsX(0x87)), (isc, AbsX(0x87)),
];

