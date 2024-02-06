use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct PPUStatus: u8 {
        //TODO: PPU open bus.
        const SPRITE_OVERFLOW   = 0b00100000;
        const SPRITE_HIT        = 0b01000000;
        const VERTICAL_BLANK    = 0b10000000;
    }
}

impl PPUStatus {
    pub fn new() -> Self {
        PPUStatus::empty()
    }

    pub fn update(&mut self, data: u8) {
        *self = PPUStatus::from_bits_truncate(data);
    }

    pub fn is_vblank(&self) -> bool {
        self.intersects(PPUStatus::VERTICAL_BLANK)
    }

    pub fn set_vblank(&mut self, cond: bool) {
        self.set(PPUStatus::VERTICAL_BLANK, cond);
    }

    pub fn set_sprite_hit(&mut self, cond: bool) {
        self.set(PPUStatus::SPRITE_HIT, cond);
    }

    pub fn set_overflow(&mut self, cond: bool) {
        self.set(PPUStatus::SPRITE_OVERFLOW, cond);
    }

    pub fn sprite_overflow(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_OVERFLOW)
    }

    pub fn sprite_hit(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_HIT)
    }
}
