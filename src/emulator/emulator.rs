use std::fmt::format;

use {
    //crate::{CPU, BUS, BUSPPU, PPU, Scene},
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Emulator {
    pub v: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator { v: 0 }
    }

    pub fn step(&mut self) {
        log(&format!("{}", self.v));
        self.v += 1;
    }
}