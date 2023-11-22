use bitflags::bitflags;

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE1              = 0b00000001;
        const NAMETABLE2              = 0b00000010;
        const VRAM_ADD_INCREMENT      = 0b00000100;
        const SPRITE_PATTERN_ADDR     = 0b00001000;
        const BACKGROUND_PATTERN_ADDR = 0b00010000;
        const SPRITE_SIZE             = 0b00100000;
        const MASTER_SLAVE_SELECT     = 0b01000000;
        const GENERATE_NMI            = 0b10000000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b00000000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8, temp: &mut u16) {
        *self = ControlRegister::from_bits_truncate(data);
        *temp = ( data as u16 & 0b00000011 ) << 10 | *temp & 0b111001111111111;
    }

    pub fn get_background_pattern_addr(&self) -> bool {
        self.intersects(ControlRegister::BACKGROUND_PATTERN_ADDR)
    }
    
    pub fn generate_nmi(&self) -> bool {
        self.intersects(ControlRegister::GENERATE_NMI)
    }
}
