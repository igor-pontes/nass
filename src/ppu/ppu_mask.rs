use bitflags::bitflags;

bitflags! {
    pub struct PPUMask: u8 {
        const GREYSCALE                = 0b00000001;
        const SHOW_BACKGROUND_LEFTMOST = 0b00000010;
        const SHOW_SPRITE_LEFTMOST     = 0b00000100;
        const SHOW_BACKGROUND          = 0b00001000;
        const SHOW_SPRITE              = 0b00010000;
        const EMPHASIZE_RED            = 0b00100000;
        const EMPHASIZE_GREEN          = 0b01000000;
        const EMPHASIZE_BLUE           = 0b10000000;
    }
}

impl PPUMask {
    pub fn new() -> Self {
        PPUMask::empty()
    }

    pub fn update(&mut self, data: u8) {
        *self = PPUMask::from_bits_truncate(data);
    }

    pub fn show_background_leftmost(&self) -> bool {
        self.intersects(PPUMask::SHOW_BACKGROUND_LEFTMOST)
    }

    pub fn show_sprite_leftmost(&self) -> bool {
        self.intersects(PPUMask::SHOW_SPRITE_LEFTMOST)
    }

    pub fn show_background(&self) -> bool {
        self.intersects(PPUMask::SHOW_BACKGROUND)
    }

    pub fn show_sprite(&self) -> bool {
        self.intersects(PPUMask::SHOW_SPRITE)
    }
}
