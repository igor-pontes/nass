mod utils;
mod ppu;
mod cpu;
mod apu;
mod mapper;
use std::fmt::format;
use wasm_bindgen::prelude::*;
use web_sys::Storage;
mod scene;
mod cartridge;
use crate::{scene::Scene, cartridge::Cartridge};

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
pub fn run(ls: Storage) {
    let file = ls.get_item("file").unwrap().unwrap();
    Cartridge::disassemble(file.as_bytes());
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, nass!");
}