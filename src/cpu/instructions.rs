
pub const OPCODES: [OpCodes; 256] = [
    OpCodes { inst: Instructions::BRK, ad: AddressingMode::Implicit },      //0x00
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::IndirectX },     //0x01
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x02
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x03
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x04
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::ZeroPage },     //0x05
    OpCodes { inst: Instructions::ASL, ad: AddressingMode::ZeroPage },     //0x06
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x07
    OpCodes { inst: Instructions::PHP, ad: AddressingMode::Implicit },     //0x08
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::Immediate },     //0x09
    OpCodes { inst: Instructions::ASL, ad: AddressingMode::Accumulator },     //0x0A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x0B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x0C
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::Absolute },     //0x0D
    OpCodes { inst: Instructions::ASL, ad: AddressingMode::Absolute },     //0x0E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x0F
    OpCodes { inst: Instructions::BPL, ad: AddressingMode::Relative },     //0x10
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::IndirectY },     //0x11
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::Absolute },     //0x12
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::Absolute },     //0x13
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::Absolute },     //0x14
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::ZeroPageX },     //0x15
    OpCodes { inst: Instructions::ASL, ad: AddressingMode::ZeroPageX },     //0x16
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x17
    OpCodes { inst: Instructions::CLC, ad: AddressingMode::Implicit },     //0x18
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::AbsoluteY },     //0x19
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x1A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x1B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x1C
    OpCodes { inst: Instructions::ORA, ad: AddressingMode::AbsoluteX },     //0x1D
    OpCodes { inst: Instructions::ASL, ad: AddressingMode::AbsoluteX },     //0x1E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x1F
    OpCodes { inst: Instructions::JSR, ad: AddressingMode::Absolute },     //0x20
    OpCodes { inst: Instructions::AND, ad: AddressingMode::IndirectX },     //0x21
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x22
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x23
    OpCodes { inst: Instructions::BIT, ad: AddressingMode::ZeroPage },     //0x24
    OpCodes { inst: Instructions::AND, ad: AddressingMode::ZeroPage },     //0x25
    OpCodes { inst: Instructions::ROL, ad: AddressingMode::ZeroPage },     //0x26
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x27
    OpCodes { inst: Instructions::PLP, ad: AddressingMode::Implicit },     //0x28
    OpCodes { inst: Instructions::AND, ad: AddressingMode::Immediate },     //0x29
    OpCodes { inst: Instructions::ROL, ad: AddressingMode::Accumulator },     //0x2A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x2B
    OpCodes { inst: Instructions::BIT, ad: AddressingMode::Absolute },     //0x2C
    OpCodes { inst: Instructions::AND, ad: AddressingMode::Absolute },     //0x2D
    OpCodes { inst: Instructions::ROL, ad: AddressingMode::Absolute },     //0x2E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x2F
    OpCodes { inst: Instructions::BMI, ad: AddressingMode::Relative },      //0x30
    OpCodes { inst: Instructions::AND, ad: AddressingMode::IndirectY },     //0x31
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x32
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x33
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x34
    OpCodes { inst: Instructions::AND, ad: AddressingMode::ZeroPageX },     //0x35
    OpCodes { inst: Instructions::ROL, ad: AddressingMode::ZeroPageX },     //0x36
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x37
    OpCodes { inst: Instructions::SEC, ad: AddressingMode::Implicit },     //0x38
    OpCodes { inst: Instructions::AND, ad: AddressingMode::AbsoluteY },     //0x39
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x3A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x3B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x3C
    OpCodes { inst: Instructions::AND, ad: AddressingMode::AbsoluteX },     //0x3D
    OpCodes { inst: Instructions::ROL, ad: AddressingMode::AbsoluteX },     //0x3E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x3F
    OpCodes { inst: Instructions::RTI, ad: AddressingMode::Implicit },     //0x40
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::IndirectX },     //0x41
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x42
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x43
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x44
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::ZeroPage },     //0x45
    OpCodes { inst: Instructions::LSR, ad: AddressingMode::ZeroPage },     //0x46
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x47
    OpCodes { inst: Instructions::PHA, ad: AddressingMode::Implicit },     //0x48
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::Immediate },     //0x49
    OpCodes { inst: Instructions::LSR, ad: AddressingMode::Accumulator },     //0x4A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x4B
    OpCodes { inst: Instructions::JMP, ad: AddressingMode::Absolute },     //0x4C
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::Absolute },     //0x4D
    OpCodes { inst: Instructions::LSR, ad: AddressingMode::Absolute },     //0x4E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x4F
    OpCodes { inst: Instructions::BVC, ad: AddressingMode::Relative },     //0x50
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::IndirectY },     //0x51
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x52
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x53
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x54
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::ZeroPageX },     //0x55
    OpCodes { inst: Instructions::LSR, ad: AddressingMode::ZeroPageX },     //0x56
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x57
    OpCodes { inst: Instructions::CLI, ad: AddressingMode::Implicit },     //0x58
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::AbsoluteY },     //0x59
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x5A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x5B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x5C
    OpCodes { inst: Instructions::EOR, ad: AddressingMode::AbsoluteX },     //0x5D
    OpCodes { inst: Instructions::LSR, ad: AddressingMode::AbsoluteX },     //0x5E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x5F
    OpCodes { inst: Instructions::RTS, ad: AddressingMode::Implicit },      //0x60
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::IndirectX },     //0x61
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x62
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x63
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x64
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::ZeroPage },     //0x65
    OpCodes { inst: Instructions::ROR, ad: AddressingMode::ZeroPage },     //0x66
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x67
    OpCodes { inst: Instructions::PLA, ad: AddressingMode::Implicit },     //0x68
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::Immediate },     //0x69
    OpCodes { inst: Instructions::ROR, ad: AddressingMode::Accumulator },     //0x6A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x6B
    OpCodes { inst: Instructions::JMP, ad: AddressingMode::Indirect },     //0x6C
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::Absolute },     //0x6D
    OpCodes { inst: Instructions::ROR, ad: AddressingMode::Absolute },     //0x6E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x6F
    OpCodes { inst: Instructions::BVS, ad: AddressingMode::Relative },     //0x70
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::IndirectY },     //0x71
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x72
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x73
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x74
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::ZeroPageX },     //0x75
    OpCodes { inst: Instructions::ROR, ad: AddressingMode::ZeroPageX },     //0x76
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x77
    OpCodes { inst: Instructions::SEI, ad: AddressingMode::Implicit },     //0x78
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::AbsoluteY },     //0x79
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x7A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x7B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x7C
    OpCodes { inst: Instructions::ADC, ad: AddressingMode::AbsoluteX },     //0x7D
    OpCodes { inst: Instructions::ROR, ad: AddressingMode::AbsoluteX },     //0x7E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x7F
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x80
    OpCodes { inst: Instructions::STA, ad: AddressingMode::IndirectX },     //0x81
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x82
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x83
    OpCodes { inst: Instructions::STY, ad: AddressingMode::ZeroPage },     //0x84
    OpCodes { inst: Instructions::STA, ad: AddressingMode::ZeroPage },     //0x85
    OpCodes { inst: Instructions::STX, ad: AddressingMode::ZeroPage },     //0x86
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x87
    OpCodes { inst: Instructions::DEY, ad: AddressingMode::Implicit },     //0x88
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x89
    OpCodes { inst: Instructions::TXA, ad: AddressingMode::Implicit },     //0x8A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x8B
    OpCodes { inst: Instructions::STY, ad: AddressingMode::Absolute },     //0x8C
    OpCodes { inst: Instructions::STA, ad: AddressingMode::Absolute },     //0x8D
    OpCodes { inst: Instructions::STX, ad: AddressingMode::Absolute },     //0x8E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x8F
    OpCodes { inst: Instructions::BCC, ad: AddressingMode::Relative },      //0x90
    OpCodes { inst: Instructions::STA, ad: AddressingMode::IndirectY },     //0x91
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x92
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x93
    OpCodes { inst: Instructions::STY, ad: AddressingMode::ZeroPageX },     //0x94
    OpCodes { inst: Instructions::STA, ad: AddressingMode::ZeroPageX },     //0x95
    OpCodes { inst: Instructions::STX, ad: AddressingMode::ZeroPageY },     //0x96
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x97
    OpCodes { inst: Instructions::TYA, ad: AddressingMode::Implicit },     //0x98
    OpCodes { inst: Instructions::STA, ad: AddressingMode::AbsoluteY },     //0x99
    OpCodes { inst: Instructions::TXS, ad: AddressingMode::Implicit },     //0x9A
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x9B
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x9C
    OpCodes { inst: Instructions::STA, ad: AddressingMode::AbsoluteX },     //0x9D
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x9E
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0x9F
    OpCodes { inst: Instructions::LDY, ad: AddressingMode::Immediate },     //0xA0
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::IndirectX },     //0xA1
    OpCodes { inst: Instructions::LDX, ad: AddressingMode::Immediate },     //0xA2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xA3
    OpCodes { inst: Instructions::LDY, ad: AddressingMode::ZeroPage },     //0xA4
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::ZeroPage },     //0xA5
    OpCodes { inst: Instructions::LDX, ad: AddressingMode::ZeroPage },     //0xA6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xA7
    OpCodes { inst: Instructions::TAY, ad: AddressingMode::Implicit },     //0xA8
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::Immediate },     //0xA9
    OpCodes { inst: Instructions::TAX, ad: AddressingMode::Implicit },     //0xAA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xAB
    OpCodes { inst: Instructions::LDY, ad: AddressingMode::Absolute },     //0xAC
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::Absolute },     //0xAD
    OpCodes { inst: Instructions::LDX, ad: AddressingMode::Absolute },     //0xAE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xAF
    OpCodes { inst: Instructions::BCS, ad: AddressingMode::Relative },     //0xB0
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::IndirectY },     //0xB1
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xB2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xB3
    OpCodes { inst: Instructions::LDY, ad: AddressingMode::ZeroPageX },     //0xB4
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::ZeroPageX },     //0xB5
    OpCodes { inst: Instructions::LDX, ad: AddressingMode::ZeroPageY },     //0xB6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xB7
    OpCodes { inst: Instructions::CLV, ad: AddressingMode::Implicit },     //0xB8
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::AbsoluteY },     //0xB9
    OpCodes { inst: Instructions::TSX, ad: AddressingMode::Implicit },     //0xBA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xBB
    OpCodes { inst: Instructions::LDY, ad: AddressingMode::AbsoluteX },     //0xBC
    OpCodes { inst: Instructions::LDA, ad: AddressingMode::AbsoluteX },     //0xBD
    OpCodes { inst: Instructions::LDX, ad: AddressingMode::AbsoluteY },     //0xBE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xBF
    OpCodes { inst: Instructions::CPY, ad: AddressingMode::Immediate },      //0xC0
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::IndirectX },     //0xC1
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xC2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xC3
    OpCodes { inst: Instructions::CPY, ad: AddressingMode::ZeroPage },     //0xC4
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::ZeroPage },     //0xC5
    OpCodes { inst: Instructions::DEC, ad: AddressingMode::ZeroPage },     //0xC6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xC7
    OpCodes { inst: Instructions::INY, ad: AddressingMode::Implicit },     //0xC8
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::Immediate },     //0xC9
    OpCodes { inst: Instructions::DEX, ad: AddressingMode::Implicit },     //0xCA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xCB
    OpCodes { inst: Instructions::CPY, ad: AddressingMode::Absolute },     //0xCC
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::Absolute },     //0xCD
    OpCodes { inst: Instructions::DEC, ad: AddressingMode::Absolute },     //0xCE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xCF
    OpCodes { inst: Instructions::BNE, ad: AddressingMode::Relative },     //0xD0
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::IndirectY },     //0xD1
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xD2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xD3
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xD4
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::ZeroPageX },     //0xD5
    OpCodes { inst: Instructions::DEC, ad: AddressingMode::ZeroPageX },     //0xD6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xD7
    OpCodes { inst: Instructions::CLD, ad: AddressingMode::Implicit },     //0xD8
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::AbsoluteY },     //0xD9
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xDA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xDB
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xDC
    OpCodes { inst: Instructions::CMP, ad: AddressingMode::AbsoluteX },     //0xDD
    OpCodes { inst: Instructions::DEC, ad: AddressingMode::AbsoluteX },     //0xDE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xDF
    OpCodes { inst: Instructions::CPX, ad: AddressingMode::Immediate },     //0xE0
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::IndirectX },     //0xE1
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xE2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xE3
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::ZeroPage },     //0xE4
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::ZeroPage },      //0xE5
    OpCodes { inst: Instructions::INC, ad: AddressingMode::ZeroPage },      //0xE6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xE7
    OpCodes { inst: Instructions::INX, ad: AddressingMode::Implicit },      //0xE8
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::Immediate },      //0xE9
    OpCodes { inst: Instructions::NOP, ad: AddressingMode::Implicit },      //0xEA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xEB
    OpCodes { inst: Instructions::CPX, ad: AddressingMode::Absolute },      //0xEC
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::Absolute },      //0xED
    OpCodes { inst: Instructions::INC, ad: AddressingMode::Absolute },      //0xEE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xEF
    OpCodes { inst: Instructions::BEQ, ad: AddressingMode::Relative },      //0xF0
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::IndirectY },      //0xF1
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xF2
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xF3
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xF4
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::ZeroPageX },      //0xF5
    OpCodes { inst: Instructions::INC, ad: AddressingMode::ZeroPageX },      //0xF6
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },     //0xF7
    OpCodes { inst: Instructions::SED, ad: AddressingMode::Implicit },      //0xF8
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::AbsoluteY },     //0xF9
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },         //0xFA
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },         //0xFB
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },         //0xFC
    OpCodes { inst: Instructions::SBC, ad: AddressingMode::AbsoluteX },     //0xFD
    OpCodes { inst: Instructions::INC, ad: AddressingMode::AbsoluteX },     //0xFE
    OpCodes { inst: Instructions::NULL, ad: AddressingMode::NULL },         //0xFF
];

#[derive(Debug)]
pub enum Instructions {
    NULL,
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
    NULL,
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY
}
#[derive(Debug)]
pub struct OpCodes {
    pub inst: Instructions, // Instruction
    pub ad: AddressingMode,
}
