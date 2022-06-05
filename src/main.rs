mod cpu;
mod ppu;
use crate::cpu::{CPU, instructions::OPCODES, bus::BUS};

fn main() {
    let mut b = BUS::new();
    let mut c = CPU::new(&mut b);
    let _op = &OPCODES[0xFF];
    println!("{}", c.f);
}
