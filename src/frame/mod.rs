pub struct Frame {
    frame: [u32; Frame::WIDTH*Frame::HEIGHT],
    index: usize,
    frames: u8
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Frame {
        Frame { 
            frame: [0xFF; Frame::WIDTH*Frame::HEIGHT],
            index: 0,
            frames: 0
        }
    }

    pub fn set_pixel(&mut self, color: u32) {
        self.frame[self.index] = color;
        self.index += 1;
        if self.index == Frame::WIDTH * Frame::HEIGHT {
            self.index = 0;
            self.frames += 1;
        }
    }

    pub fn get_pointer(&self) -> *const u32 {
        self.frame.as_ptr()
    }

    pub fn even_frame(&self) -> bool { 
        self.frames & 1 == 0 
    }
}
