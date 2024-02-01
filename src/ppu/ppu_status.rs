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
        PPUStatus::from_bits_truncate(0b00000000)
    }
    pub fn update(&mut self, data: u8) {
        *self = PPUStatus::from_bits_truncate(data);
    }
    pub fn nmi_status(&self) -> bool {
        self.intersects(PPUStatus::VERTICAL_BLANK)
    }
    pub fn sprite_overflow(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_OVERFLOW)
    }

    pub fn sprite_hit(&self) -> bool {
        self.intersects(PPUStatus::SPRITE_HIT)
    }
}
