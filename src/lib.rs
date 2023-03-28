use js_sys::ArrayBuffer;

mod utils;
mod ppu;
mod cpu;
mod mapper;
mod scene;
mod cartridge;
mod emulator;
use { 
    core::cell::RefCell,
    crate::{ cartridge::*, cpu::*, ppu::*, scene::*, emulator::* }, 
    wasm_bindgen::prelude::*, 
    js_sys
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Sync trait = types for which it is safe to share references between threads.
// In Rust, static variables must be thread safe. Hence, the specified type must implement the sync trait.
// Setting a variable as static with a type that is not thread safe may lead to data race conditions.
// Static variables can only refer other static variables by refence only.
// To signal Rust that we are not working with multiple threads, we can use "thread_local"
// Since javascript only uses 1 thread, this is perfectly fine.
thread_local! { 
    static NES: RefCell<Emulator> = RefCell::new(Emulator::new());
}

#[wasm_bindgen]
pub fn disassemble(_rom: ArrayBuffer) {
    log("disassemble.");
}

#[wasm_bindgen]
pub fn step() {
    // https://badboi.dev/rust/2020/07/17/cell-refcell.html
    // Rust's borrow rules:
    // You can have one mutable reference. OR (exclusive; Either one or another, not both.)
    // You can have multiple immutable references.
    
    NES.with(|e| e.borrow_mut().step());
}
