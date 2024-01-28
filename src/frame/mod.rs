pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    offset_x: usize,
    offset_y: usize,
    odd_tile: bool,
    pub frame: Box<[u8; 256*240]>
}

impl Frame {
    pub fn new() -> Frame {
        // 32 x 30 tiles
        Frame {
            width: 256,
            height: 240,
            x: 0,
            y: 0,
            offset_x: 0,
            offset_y: 0,
            frame: Box::new([0; 256*240]),
            odd_tile: false
        }
    }

    pub fn set_pixel(&mut self, color: u8) {
        self.frame[self.width * self.y + self.x] = color;
        self.x += 1;
        if self.x == self.width {
            self.x = 0;
            self.y += 1;
            if self.y == self.height {
                self.y = 0;
            }  
        }
        
        // for y in self.offset_y..(self.offset_y + 8) {
        //     for x in self.offset_x..(self.offset_x + 8) {
        //         // if self.odd_tile {
        //         //     self.frame[y*self.width + x] = 1;
        //         // } else {
        //         //     self.frame[y*self.width + x] = 8;
        //         // }
        //         self.frame[y*self.width + x] = color;
        //     }
        // }
        // self.offset_x += 8;
        // if self.offset_x == self.width {
        //     self.offset_x = 0;
        //     self.offset_y += 8;
        //     self.odd_tile = !self.odd_tile;
        //     if self.offset_y == self.height {
        //         self.offset_y = 0;
        //     } 
        // } 

        // self.odd_tile = !self.odd_tile;
    }
}
