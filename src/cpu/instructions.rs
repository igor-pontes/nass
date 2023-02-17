use { Instruction::*, AddressingMode::* };

// https://www.nesdev.org/6502_cpu.txt
// https://www.nesdev.org/wiki/CPU_addressing_modes

pub const OP_MASK: u8 = 0x03; // Which column of instructions you want?
// ORA (x, ind) = 0b0000001
// - (INST_MASK += ...001 ) = Operation 1
// - 
// - (ADDR_MODE_MASK += ...001) = 0
// (ADDR_MODE_SHIFT += ...000 (>> 2) ) = 0 (which is X, ind)
pub const ADDR_MODE_MASK: u8 = 0x1C; // 0b11100
pub const ADDR_MODE_SHIFT: u8 = 0x02;

// Each 32, increases 1 digit:
// ORA = 0b00000000
// AND = 0b00100000
// EOR = 0b01000000
// ADC = 0b01100000
// STA = 0b10000000
// LDA = 0b10100000
// CMP = 0b11000000
// SBC = 0b11100000
// 8 Each type of operation, 8 possible outcomes: 2*2*2 = 8
pub const INST_MODE_MASK: u8 = 0xE0; // 0b11100000
pub const INST_MODE_SHIFT: u8 = 0x05; // We only care about first 3 digits

// Each column has a predetermined set of bits turned off, thats why this works.
// (https://www.masswerk.at/6502/6502_instruction_set.html#explanation)
// E.g. column 0x01 does not have bits 1-3(1111[000]1) active.
// E.g. column 0x02 does not have bits 0, 2, 3(1111[00]1[0]) active.

#[derive(Debug)]
pub enum Instruction {
    NULL, LAS, ISB, SBX, ISC,
    SLO, ANC, RLA, SRE, ASR, RRA, SAX, ARR,
    SHS, ANE, SHA, LAX, LXA, SHX, SHY, DCP,
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, 
    BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV, 
    CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, 
    JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA, 
    PHP, PLA, PLP, ROL, ROR, RTI, RTS, SBC, 
    SED, SEI, STA, STX, STY, TAX, TAY, TSX, 
    TXS, TYA, BNE, CMP, INY, PHA, SEC, TXA, 
}

#[derive(Debug)]
pub enum AddressingMode { 
    Implicit,
    Accumulator,
    Immediate,
    Zeropage,
    ZeropageX,
    ZeropageY,
    ZeropageIndexed,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    AbsoluteIndexed,
    Indirect,
    IndirectX,
    IndirectY,
    None
}

pub const OPERATION_0: [Instruction; 8] = [
    NULL,
    BIT,
    NULL,
    NULL,
    STY, // or SHY
    LDY,
    CPY,
    CPX,
];

pub const OPERATION_1: [Instruction; 8] = [
    ORA,
    AND,
    EOR,
    ADC,
    STA,
    LDA,
    CMP,
    SBC
];

pub const OPERATION_2: [Instruction; 8] = [
    ASL,
    ROL,
    LSR,
    ROR,
    STX,
    LDX,
    DEC,
    INC
];

pub const OPERATION_3: [Instruction; 8] = [
    SLO,
    RLA,
    SRE,
    RRA,
    SAX, // or SHA
    LAX,
    DCP,
    ISC
];

pub const ADDR_1: [AddressingMode; 8] = [
    IndirectX,
    Zeropage,
    Immediate,
    Absolute,
    IndirectY,
    ZeropageX,
    AbsoluteY,
    AbsoluteX
];

pub const ADDR_2: [AddressingMode; 8] = [
    Immediate,
    Zeropage,
    Accumulator,
    Absolute,
    None,
    ZeropageIndexed, // X or Y
    None,
    AbsoluteIndexed // X or Y
];

pub const ADDR_3: [AddressingMode; 8] = [
    IndirectX,
    Zeropage,
    None,
    Absolute,
    IndirectY,
    ZeropageIndexed, // X or Y
    None,
    AbsoluteIndexed // X or Y
];

// 0x100 = 0xFF + 1(zero) (total OPCODES)

pub const OP_CYCLES: [u8; 0x100] = [
    7, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    0, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    2, 6, 0, 6, 4, 4, 4, 4, 2, 5, 2, 5, 4, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    2, 5, 0, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];