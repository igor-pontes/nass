mod utils;
mod ppu;
mod cpu;
mod mapper;
mod scene;
mod cartridge;
use { 
    crate::{ cartridge::*, cpu::* }, 
    wasm_bindgen::prelude::*, 
    //js_sys
};

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

    // https://badboi.dev/rust/2020/07/17/cell-refcell.html
    // Rust's borrow rules:
    // You can have one mutable reference. OR (exclusive; Either one or another, not both.)
    // You can have multiple immutable references.

    let mapper = match Cartridge::disassemble(file) {
        Ok(m) => m,
        Err(str) => return log(&str)
    };
    let cpu = CPU::new(BUS::new(mapper));
    //cpu.reset();

    //loop {
    //    while cpu.cycle < CYCLES_PER_FRAME {
    //        cpu.step();
    //        cpu.cycle += 1;
    //    }
    //    break;
    //}
    
    alert("end");
}
