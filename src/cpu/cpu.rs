use super::{
    bus::BUS,
    instructions::{OpCodes, OP_CODES}
};
use bitvec::prelude::*;

// https://en.wikipedia.org/wiki/MOS_Technology_6502

const CLOCK_FREQUENCY: u32 = 1789773; // 1789773 Hz (1.789773 MHz)
// (1/1789773) * 10^9 =~ 559 ns per cycle

// Every cycle on 6502 is either a read or a write cycle.

// MOS 6502 implementation 
// It is common practice on a 6502 to initialize the stack pointer to $FF
#[derive(Debug)]
pub struct CPU {
    // Registers
    a: u8, // Accumulator (general purpose?)
    x: u8, // general purpose register x?
    y: u8, // general purpose register y?
    s: u8, // CPU status - stack pointer(descending stack)
    sp: u8, // Stack Pointer
    pc: u16, // Program counter
    // The low and high 8-bit halves of this register are called PCL and PCH, respectively.
    // Flags
    f: BitArray<[u8; 1], Lsb0>, // CPU flags
    // BUS
    pub bus: BUS, // RAM needs to live as long as both CPU and BUS structs.
    pub instructions: OpCodes
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            sp: 0xFF,
            pc: 0,
            f: bitarr!(u8, Lsb0; 0; 7),
            bus: BUS::new(),
            instructions: OP_CODES
        }
    }
}