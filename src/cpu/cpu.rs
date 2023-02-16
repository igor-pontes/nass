use super::{
    bus::BUS,
    instructions::*,
};
use Instruction::*;
use AddressingMode::*;
//use bitvec::prelude::*;

// https://en.wikipedia.org/wiki/MOS_Technology_6502

pub const CLOCK_FREQUENCY: u32 = 1786830; // 1786830 per second
// (1/1786830) * 10^9 =~ 560 ns per cycle
// Every cycle on 6502 is either a read or a write cycle.

// The PPU renders 262 scanlines per frame. 
// Each scanline lasts for 341 PPU clock cycles (113.667 CPU clock cycles; 1 CPU cycle = 3 PPU cycles), 
// with each clock cycle producing one pixel.
// https://www.nesdev.org/wiki/PPU_rendering

// MOS 6502 implementation 
// It is common practice on a 6502 to initialize the stack pointer to $FF

//  The P register can be read by pushing it on the stack (with PHP or
//  by causing an interrupt). https://www.nesdev.org/6502_cpu.txt

const RESET_VECTOR: u16 = 0xFFFC; // INIT CODE
const NMI_VECTOR: u16 = 0xFFFC; 
const IRQ_VECTOR: u16 = 0xFFFC; // IRQ OR BRK

#[derive(Debug)]
pub struct CPU {
    a: u8, // Accumulator (general purpose?)
    x: u8, // general purpose register x?
    y: u8, // general purpose register y?
    pc: u16, // Program counter
    s: u8, // Stack pointer (It indexes into a 256-byte stack at $0100-$01FF.)
    p: u8, // Status Register
    bus: BUS, // RAM needs to live as long as both CPU and BUS structs.
    instructions: OpCodes,
}

impl CPU {
    // https://en.wikibooks.org/wiki/NES_Programming/Initializing_the_NES
    pub fn new(bus: BUS) -> CPU {
        let mut cpu = CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD, // 0x01FD (descending stack)
            p: 0x34,
            bus,
            instructions: OP_CODES,
        };
        cpu.pc = cpu.read_address(RESET_VECTOR);
        cpu
    }

    pub fn read_address(&self, addr: u16) -> u16 {
        self.bus.read(addr) as u16 | ((self.bus.read(addr + 1)) as u16) << 8
    }

    pub fn step(self, opcode: u8) {
        match &self.instructions[opcode as usize] {
            OpCode { inst: ADC, ad: ad_mode } => {
                match ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                    AbsoluteY =>,
                    IndirectX =>,
                    IndirectY =>,
                }
            },
            OpCode { inst: AND, ad: ad_mode } => {
                match ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                    AbsoluteY =>,
                    IndirectX =>,
                    IndirectY =>,
                }
            },
            OpCode { inst: ASL, ad: ad_mode } => {
                match ad_mode {
                    Accumulator =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                }
            },
            OpCode { inst: BCC, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BCS, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BEQ, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BIT, ad: ad_mode } => {
                match ad_mode {
                    Zeropage =>,
                    Absolute =>,
                }
            },
            OpCode { inst: BMI, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BNE, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BPL, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BRK, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: BVC, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: BVS, ad: ad_mode } => {
                match ad_mode {
                    Relative =>,
                }
            },
            OpCode { inst: CLC, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: CLD, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: CLI, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: CLV, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: CMP, ad: ad_mode } => {
                match ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                    AbsoluteY =>,
                    IndirectX =>,
                    IndirectY =>,
                }
            },
            OpCode { inst: CPX, ad: ad_mode } => {
                match ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    Absolute =>,
                }
            },
            OpCode { inst: CPY, ad: ad_mode } => {
                match ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    Absolute =>,
                }
            },
            OpCode { inst: DEC, ad: ad_mode } => {
                match ad_mode {
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                }
            },
            OpCode { inst: DEX, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: DEY, ad: ad_mode } => {
                match ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: EOR, ad: ad_mode } => {
                match &ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                    AbsoluteY =>,
                    IndirectX =>,
                    IndirectY =>,
                }
            },
            OpCode { inst: INC, ad: ad_mode } => {
                match &ad_mode {
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                }
            },
            OpCode { inst: INX, ad: ad_mode } => {
                match &ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: INY, ad: ad_mode } => {
                match &ad_mode {
                    Implicit =>,
                }
            },
            OpCode { inst: JMP, ad: ad_mode } => {
                match &ad_mode {
                    Absolute =>,
                    Indirect =>,
                }
            },
            OpCode { inst: JSR, ad: ad_mode } => {
                match &ad_mode {
                    Absolute =>,
                }
            },
            OpCode { inst: LDA, ad: ad_mode } => {
                match &ad_mode {
                    Immediate =>,
                    Zeropage =>,
                    ZeropageX =>,
                    Absolute =>,
                    AbsoluteX =>,
                    AbsoluteY =>,
                    IndirectX =>,
                    IndirectY =>,
                }
            },
            _ => ()
        }
    }
    
    
}