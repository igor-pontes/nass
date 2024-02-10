mod ppu;
mod cpu;
mod mapper;
mod frame;
mod emulator;

use cfg_if::cfg_if;
use js_sys::{ ArrayBuffer, Uint8Array };
use { 
    std::cell::RefCell,
    crate::emulator::*, 
    wasm_bindgen::prelude::*, 
    js_sys
};

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

thread_local!{ static EMULATOR: RefCell<Option<Emulator>> = RefCell::new(None) }

#[wasm_bindgen]
pub fn disassemble(rom: ArrayBuffer) {
    set_panic_hook();
    let bytes = Uint8Array::new_with_byte_offset(&rom, 0).to_vec();
    EMULATOR.set(Some(Emulator::new(bytes)));
    EMULATOR.with_borrow_mut(|e| e.as_mut().and_then(|e| Some(e.reset())));
}

#[wasm_bindgen]
pub fn step() {
    EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.cpu.run_with_callback(|_| { }),
        None => { panic!("Emulator not initialized."); }
    });
}

#[wasm_bindgen]
pub fn get_frame_pointer() -> *const u32 {
    let pointer = EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.get_frame_pointer(),
        None => { panic!("Emulator not initialized."); }
    });
    pointer
}

#[wasm_bindgen]
pub fn get_color(index: usize) -> u32 {
    let pointer = EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.get_color(index),
        None => { panic!("Emulator not initialized."); }
    });
    pointer
}
