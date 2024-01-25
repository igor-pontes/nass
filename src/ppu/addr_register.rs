pub struct AddrRegister {
    value: (u8, u8),
    hi_ptr: bool
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0), // high - low
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8; // b11111111_00000000 -> b00000000_11111111
        self.value.1 = (data & 0xFF) as u8;
    }

    pub fn update(&mut self, data: u8, temp: &mut u16) {
        if self.hi_ptr {
            self.value.0 = data;
            *temp = ( data as u16 & 0b00111111 ) << 8 | *temp & 0x00FF;
        } else {
            self.value.1 = data;
            *temp = data as u16 & 0b11111111 | *temp & 0xFF00;
        } 

        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
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

    pub fn y_increment(&mut self) {
        if (self.value.0 & 0x70) != 0x70 { // if fine Y < 7
          self.value.0 += 0x10;
        } // increment fine Y
        else {
            self.value.0 &= !0x70; // fine Y = 0
            let mut y = (self.get() & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                self.value.0 ^= 0x08; // switch vertical nametable
            }
            else if y == 31 {
                y = 0  
            } // coarse Y = 0, nametable not switched
            else {
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
        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn toggle_latch(&mut self) {
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn latch(&self) -> bool {
        self.hi_ptr
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | ( self.value.1 as u16 )
    }
}
