mod utils;
mod ppu;
mod cpu;
mod apu;
mod mapper;
mod scene;
mod cartridge;
use crate::{cartridge::*, cpu::*};
use {wasm_bindgen::prelude::*, std::time::{Duration, Instant}};
use std::thread::sleep;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {

    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn disassemble(file: String) {

    let cartridge = match Cartridge::disassemble(file) {
        Ok(cartridge) => cartridge,
        Err(str) => return log(&str)
    };

    //alert(&format!("{:?}", cartridge.mirroring));
    //alert(&format!("{:?}", cartridge.mapper));
     
    let mut cpu = CPU::new(BUS::new(cartridge));
    cpu.reset();
    
    log(&format!("{:?}", cpu.pc));

    sleep(Duration::new(1, 0));
    //let now = Instant::now();
    //loop {
    //    while cpu.get_cycle() <= CLOCK_FREQUENCY {
    //        cpu.step();
    //    }
    //    sleep(Duration::new(0, 1000000000 - now.elapsed().as_nanos() as u32));
    //    break;
    //}
    //alert(&now.elapsed().as_secs().to_string())
    alert("end");
}