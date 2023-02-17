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

const NMI_VECTOR: u16 = 0xFFFA; 
const RESET_VECTOR: u16 = 0xFFFC; // INIT CODE
const IRQ_VECTOR: u16 = 0xFFFE; // IRQ OR BRK

enum Interrupt {
    NMI,
    IRQ,
    BRK
}

#[derive(Debug)]
pub struct CPU {
    a: u8, // Accumulator (general purpose?)
    x: u8, // general purpose register x?
    y: u8, // general purpose register y?
    pc: u16, // Program counter
    s: u8, // Stack pointer (It indexes into a 256-byte stack at $0100-$01FF.)
    p: u8, // Status Register
    bus: BUS, // RAM needs to live as long as both CPU and BUS structs.
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
        };
        cpu.pc = cpu.read_address(RESET_VECTOR);
        cpu
    }

    pub fn read_address(&self, addr: u16) -> u16 {
        self.bus.read(addr) as u16 | ((self.bus.read(addr + 1)) as u16) << 8
    }

    pub fn step(self, opcode: u8) {
        unimplemented!()
    }
    
    
}