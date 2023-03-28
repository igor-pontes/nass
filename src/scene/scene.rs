pub struct Scene {
    pub width: u32,
    pub height: u32,
    pixels: js_sys::Array,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            width: 256,
            height: 240,
            pixels: js_sys::Array::new_with_length(256*240),
        }
    }
    // pub fn set_pixel(&mut self, x: u32, y: u32, value: &str) {
    //     self.pixels.set((x * 256) + y, JsValue::from_str(value))
    // }
}