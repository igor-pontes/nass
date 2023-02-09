use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub pixels: Option<u8>,
}

#[wasm_bindgen]
impl Scene {
    pub fn new() -> Scene {
        Scene {
            width: 256,
            height: 240,
            pixels: None,
        }
    }
    pub fn getIndex() -> usize {
        // get pixel screen
        unimplemented!()
    }
}