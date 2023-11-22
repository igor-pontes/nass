use js_sys::{ ArrayBuffer, Uint8Array };
use wasm_bindgen::{closure::Closure, JsValue};

mod ppu;
mod cpu;
mod mapper;
mod frame;
mod emulator;
use { 
    // core::cell::RefCell,
    crate::emulator::*, 
    wasm_bindgen::prelude::*, 
    js_sys
};

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn disassemble(rom: ArrayBuffer) -> JsValue {
    let array = Uint8Array::new_with_byte_offset(&rom, 0).to_vec();
    let mut emulator = Emulator::new(&array);
    emulator.reset();
    let cb = Closure::wrap(Box::new(move || { emulator.step(); }) as Box<dyn FnMut()>);
    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}
