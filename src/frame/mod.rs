pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub frame: Box<[u8; 256*240]>
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            width: 256,
            height: 240,
            x: 0,
            y: 0,
            frame: Box::new([0; 256*240])
        }
    }

    pub fn set_pixel(&mut self, color: u8) {
        self.frame[self.height * self.y + self.x] = color;
        self.x += 1;
        if self.x == self.width - 1 {
            self.x = 0;
            if self.y == self.height - 1 {
                self.y = 0;
            } else {
                self.y += 1;
            }
        }
    }
}
