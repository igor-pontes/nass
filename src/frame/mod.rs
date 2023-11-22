use wasm_bindgen::JsValue;
pub struct Frame {
    pub width: u32,
    pub height: u32,
    x: u32,
    y: u32,
    frame: Vec<String>
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            width: 256,
            height: 240,
            x: 0,
            y: 0,
            // pixels: js_sys::Uint32Array::new_with_length(256*240),
            frame: Vec::new()
        }
    }
    // pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
    //     self.pixels.set(&JsValue::from_f64(color as f64), (self.x * 256) + self.y);
    //     self.x += 1;
    //     if self.x == self.width - 1 {
    //         self.x = 0;
    //         self.y += 1;
    //     }
    //     if self.y == self.height {
    //         self.y = 0;
    //     }
    // }
}
