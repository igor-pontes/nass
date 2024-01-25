use num_enum::TryFromPrimitive;

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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ImplicitOps {
    BRK = 0x00,
    RTI = 0x04,
    RTS = 0x06,
    PHP = 0x08,
    CLC = 0x18,
    PLP = 0x28,
    SEC = 0x38,
    PHA = 0x48,
    CLI = 0x58,
    PLA = 0x68,
    SEI = 0x78,
    DEY = 0x88,
    TYA = 0x98,
    TAY = 0xA8,
    CLV = 0xB8,
    INY = 0xC8,
    CLD = 0xD8,
    INX = 0xE8,
    SED = 0xF8,
    TXA = 0x8A,
    TXS = 0x9A,
    TAX = 0xAA,
    TSX = 0xBA,
    DEX = 0xCA,
    NOP = 0xEA,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ImmediateOps {
    LDY = 0xA0,
    CPY = 0xC0,
    CPX = 0xE0,
    LDX = 0xA2,
    ALR = 0x4B,
    ARR = 0x6B,
    ANE = 0x8B,
    LXA = 0xAB,
    SBX = 0xCB,
    USBC = 0xEB, 
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum AbsoluteYOps  {
    LAS = 0xBB,
    TAS = 0x9B,
    SHA = 0x9F,
}

#[derive(Debug, PartialEq)]
pub enum AddressingMode { 
    Immediate,
    Zeropage,
    ZeropageX,
    ZeropageY,
    ZeropageIndexed,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    AbsoluteIndexed,
    Indirect,
    IndirectX,
    IndirectY,
    None
}

// Last to calculate (if not operation_0, it's a NOP zeropage or zeropageX)
// Blend NOP(Zpg, Zpgx), NOP(Abs, Ansx) with Operation0's Adressmode
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Operation0 {
    BIT = 0x1,
    JMP = 0x2,
    _JMP = 0x3,
    STY = 0x4, // or SHY
    LDY = 0x5,
    CPY = 0x6,
    CPX = 0x7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Operation1 {
    ORA = 0x0,
    AND = 0x1,
    EOR = 0x2,
    ADC = 0x3,
    STA = 0x4,
    LDA = 0x5,
    CMP = 0x6,
    SBC = 0x7
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Operation2 {
    ASL = 0x0,
    ROL = 0x1,
    LSR = 0x2,
    ROR = 0x3,
    STX = 0x4,
    LDX = 0x5,
    DEC = 0x6,
    INC = 0x7 
}

pub enum Operation3 {
    SLO = 0x0,
    RLA = 0x1,
    SRE = 0x2,
    RRA = 0x3,
    SAX = 0x4,
    LAX = 0x5,
    DCP = 0x6,
    ISC = 0x7
}

pub const ADDR_1: [AddressingMode; 8] = [
    AddressingMode::IndirectX,
    AddressingMode::Zeropage,
    AddressingMode::Immediate,
    AddressingMode::Absolute,
    AddressingMode::IndirectY,
    AddressingMode::ZeropageX,
    AddressingMode::AbsoluteY,
    AddressingMode::AbsoluteX
];

pub const ADDR_2: [AddressingMode; 8] = [
    AddressingMode::Immediate,
    AddressingMode::Zeropage,
    AddressingMode::None,
    AddressingMode::Absolute,
    AddressingMode::None,
    AddressingMode::ZeropageIndexed, // X or Y
    AddressingMode::None,
    AddressingMode::AbsoluteIndexed // X or Y
];

pub const ADDR_3: [AddressingMode; 8] = [
    AddressingMode::IndirectX,
    AddressingMode::Zeropage,
    AddressingMode::None,
    AddressingMode::Absolute,
    AddressingMode::IndirectY,
    AddressingMode::ZeropageIndexed, // X or Y
    AddressingMode::None,
    AddressingMode::AbsoluteIndexed // X or Y
];

pub const OP_CYCLES: [u8; 0x100] = [
    7, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 0, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    6, 6, 0, 0, 3, 3, 5, 0, 4, 2, 2, 0, 4, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    6, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 3, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    6, 6, 0, 0, 0, 3, 5, 0, 4, 2, 2, 0, 5, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    0, 6, 0, 0, 3, 3, 3, 0, 2, 0, 2, 0, 4, 4, 4, 0,
    2, 6, 0, 0, 4, 4, 4, 0, 2, 5, 2, 0, 0, 5, 0, 0,
    2, 6, 2, 0, 3, 3, 3, 0, 2, 2, 2, 0, 4, 4, 4, 0,
    2, 5, 0, 0, 4, 4, 4, 0, 2, 4, 2, 0, 4, 4, 4, 0,
    2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 0, 4, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 2, 4, 4, 6, 0,
    2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
];
