
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub index: usize,
    frame: [u8; 256*240],
}

impl Frame {
    pub fn new() -> Frame {
        // 32 x 30 tiles
        Frame {
            width: 256,
            height: 240,
            index: 0,
            frame: [0; 256*240],
        }
    }

    pub fn set_pixel(&mut self, color: u8) {
        self.frame[self.index] = color;
        self.increment();
    }

    pub fn increment(&mut self) {
        self.index += 1;
        if self.index == (self.height * self.width) {
            self.index = 0;
        }
    }

    pub fn get_pointer(&self) -> *const u8 {
        self.frame.as_ptr()
    }
}
