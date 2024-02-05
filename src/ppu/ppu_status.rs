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

    pub fn set_vblank(&mut self) {
        self.update(self.bits() | 0x80);
    }

    pub fn clear_vblank(&mut self) {
        self.update(self.bits() | 0x80);
    }

    pub fn set_sprite_hit(&mut self) {
        self.update(self.bits() | 0x40);
    }

    pub fn clear_sprite_hit(&mut self) {
        self.update(self.bits() & !0x40);
    }

    pub fn set_overflow(&mut self) {
        self.update(self.bits() | 0x20);
    }

    pub fn clear_overflow(&mut self) {
        self.update(self.bits() & !0x20);
    }

    pub fn sprite_overflow(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_OVERFLOW)
    }

    pub fn sprite_hit(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_HIT)
    }
}
