pub struct AddrRegister {
    value: (u8, u8),
    latch: bool
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0), // high - low
            latch: false,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0x00FF) as u8;
    }

    pub fn set_horizontal(&mut self, t: u16) {
        // X coarse
        let (x_high, x_low) = (((t & 0x0400) >> 8) as u8, (t & 0x001F) as u8);
        self.value.0 = (self.value.0 & !0x04) | x_high;
        self.value.1 = (self.value.1 & !0x1F) | x_low;
    }

    pub fn set_vertical(&mut self, t: u16) {
        // Y coarse
        // let (y_high, y_low) = (((t & 0x7B00) >> 8) as u8, (t & 0x00E0) as u8);
        let (y_high, y_low) = (((t & 0x3B00) >> 8) as u8, (t & 0x00E0) as u8);
        self.value.0 = (self.value.0 & !0x7B) | y_high; 
        self.value.1 = (self.value.1 & !0xE0) | y_low;
    }

    pub fn update(&mut self, data: u8, temp: &mut u16) {
        if !self.latch {
            *temp = (((data as u16) & 0x3F ) << 8) | (*temp & 0xFF);
        } else {
            *temp = ((data as u16) & 0x00FF) | (*temp & 0xFF00);
            self.value.0 = ((*temp & 0xFF00) >> 8) as u8;
            self.value.1 = (*temp & 0x00FF) as u8;
        } 
        self.toggle_latch();
    }

    pub fn coarse_x_increment(&mut self) {
        if (self.value.1 & 0x1F) == 31 {
            self.value.1 &= !0x1F; // coarse X = 0
            self.value.0 ^= 0x04; // switch horizontal nametable
        } // if coarse X == 31
        else {
            self.value.1 += 1; // increment coarse X
        }
    }

    pub fn coarse_y_increment(&mut self) {
        if (self.value.0 & 0x70) != 0x70 { // if fine Y < 7
          self.value.0 += 0x10; // increment fine Y
        }
        else {
            self.value.0 &= !0x70; // fine Y = 0
            let mut y = (self.get() & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0;
                self.value.0 ^= 0x08; // switch vertical nametable
            } else if y > 31 {
                y = 0  // coarse Y = 0, nametable not switched
            } else {
                y += 1; // increment coarse Y
            }
            self.set((self.get() & !0x03E0) | (y << 5)); // put coarse Y back into addr
        }
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }

    pub fn toggle_latch(&mut self) {
        self.latch = !self.latch;
    }

    pub fn latch(&self) -> bool {
        self.latch
    }

    pub fn get(&self) -> u16 {
        ( ( self.value.0 as u16 ) << 8 ) | ( self.value.1 as u16 )
    }
}
