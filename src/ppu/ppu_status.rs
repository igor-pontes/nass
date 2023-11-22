use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct PPUStatus: u8 {
        //TODO: PPU open bus.
        const SPRITE_OVERFLOW = 0b00100000;
        const SPRITE_HIT = 0b01000000;
        const VERTICAL_BLANK = 0b10000000;
    }
}

impl PPUStatus {
    pub fn new() -> Self {
        PPUStatus::from_bits_truncate(0b00000000)
    }
    pub fn update(&mut self, data: u8) {
        *self = PPUStatus::from_bits_truncate(data);
    }
}
