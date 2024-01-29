pub struct Frame {
    pub width: usize,
    pub height: usize,
    // pub x: usize,
    // pub y: usize,
    pub index: usize,
    frame: Box<[u8; 256*240]>,
    temp: Box<[u8; 256*240]>
}

impl Frame {
    pub fn new() -> Frame {
        // 32 x 30 tiles
        Frame {
            width: 256,
            height: 240,
            // x: 0,
            // y: 0,
            index: 0,
            frame: Box::new([0; 256*240]),
            temp: Box::new([0; 256*240]),
        }
    }

    pub fn set_pixel(&mut self, color: u8) {
        self.frame[self.index] = color;
        self.index += 1;
        if self.index == (self.height * self.width) {
            self.index = 0;
        }
    }

    pub fn set_frame(&mut self) {
        self.temp.copy_from_slice(self.frame.as_slice());
    }

    pub fn get_pointer(&self) -> *const u8 {
        self.temp.as_ptr()
    }
}
