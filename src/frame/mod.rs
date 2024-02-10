pub struct Frame {
    frame: [u32; 256*240],
    index: usize,
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Frame {
        Frame { 
            frame: [0xFF; 256*240], // 0x000000FF = Black
            index: 0
        }
    }

    pub fn set_pixel(&mut self, color: u32) {
        self.frame[self.index] = color;
        self.index += 1;
        if self.index == Frame::WIDTH * Frame::HEIGHT {
            self.index = 0;
        }
    }

    pub fn get_pointer(&self) -> *const u32 {
        self.frame.as_ptr()
    }
}
