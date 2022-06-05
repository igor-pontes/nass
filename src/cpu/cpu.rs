use super::bus::BUS;
use super::instructions::Instructions;
use bitvec::prelude::*;


const CLOCK_FREQUENCY: u32 = 1789773; // 1789773 Hz (1.789773 MHz)
// (1/1789773) * 10^9 =~ 559 ns per cycle

// Every cycle on 6502 is either a read or a write cycle.

// MOS 6502 implementation 
#[derive(Debug)]
pub struct CPU<'a> {
    // Registers
    pub a: u8, // Accumulator (general purpose?)
    pub x: u8, // general purpose register x?
    pub y: u8, // general purpose register y?
    pub s: u8, // CPU status
    pub sp: u8, // Stack Pointer
    pub pc: u16, // Program counter
    // Flags
    pub f: BitArray<[u8; 1], Lsb0>, // CPU flags
    // BUS
    pub bus: &'a mut BUS, // RAM needs to live as long as both CPU and BUS structs.
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a mut BUS) -> CPU<'a> {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            sp: 0,
            pc: 0,
            f: bitarr!(u8, Lsb0; 0; 7),
            bus
        }
    }
}