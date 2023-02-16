mod utils;
mod ppu;
mod cpu;
mod apu;
mod mapper;
use wasm_bindgen::prelude::*;
mod scene;
mod cartridge;
use crate::{cartridge::*, cpu::*};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn disassemble_and_run(file: String) {
    let cartridge = match Cartridge::disassemble(file.as_bytes()) {
        Ok(cartridge) => cartridge,
        Err(str) => return alert(str)
    };
    let cpu = CPU::new(BUS::new(cartridge));
    while true {
        
        return
    }
}