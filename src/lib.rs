use js_sys::{ ArrayBuffer, Uint8Array };

mod ppu;
mod cpu;
mod mapper;
mod frame;
mod emulator;
use { 
    std::cell::RefCell,
    crate::emulator::*, 
    wasm_bindgen::prelude::*, 
    js_sys
};

thread_local!{ static EMULATOR: RefCell<Option<Emulator>> = RefCell::new(None) }

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn disassemble(rom: ArrayBuffer) {
    let array = Uint8Array::new_with_byte_offset(&rom, 0).to_vec();
    EMULATOR.set(Some(Emulator::new(&array)));
    EMULATOR.with_borrow_mut(|e| e.as_mut().and_then(|e| Some(e.reset())));
}

#[wasm_bindgen]
pub fn step() {
    EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.step(),
        None => { log("Emulator not initialized."); }
    });
}

#[wasm_bindgen]
pub fn get_frame_pointer() -> *const u8 {
    let pointer = EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.get_frame_pointer(),
        None => { log("Emulator not initialized."); panic!(); }
    });
    pointer
}

#[wasm_bindgen]
pub fn get_palette_pointer() -> *const u8 {
    let pointer = EMULATOR.with_borrow_mut(|e| match e {
        Some(e) => e.get_palette_pointer(),
        None => { log("Emulator not initialized."); panic!(); }
    });
    pointer
}
