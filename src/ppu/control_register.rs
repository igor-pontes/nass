use bitflags::bitflags;

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE1              = 0b00000001;
        const NAMETABLE2              = 0b00000010;
        const VRAM_ADD_INCREMENT      = 0b00000100;
        const SPRITE_PATTERN_ADDR     = 0b00001000;
        const BACKGROUND_PATTERN_ADDR = 0b00010000;
        const SPRITE_SIZE_16          = 0b00100000;
        const MASTER_SLAVE_SELECT     = 0b01000000;
        const GENERATE_NMI            = 0b10000000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::empty()
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if self.intersects(ControlRegister::VRAM_ADD_INCREMENT) { return 32; } 
        1
    }

    pub fn get_nametable(&self) -> u8 {
        (self.intersects(ControlRegister::NAMETABLE2) as u8) << 1 | 
        (self.intersects(ControlRegister::NAMETABLE1) as u8)
    }

    pub fn update(&mut self, data: u8, temp: &mut u16) {
        *self = ControlRegister::from_bits_retain(data);
        *temp = (( self.get_nametable() as u16 ) << 10) | (*temp & 0xF3FF);
    }

    pub fn is_sprite_size_16(&self) -> bool {
        self.intersects(ControlRegister::SPRITE_SIZE_16)
    }

    pub fn get_sprite_pattern_addr(&self) -> u16 {
        // ignored in 8x16 mode
        let right_table = self.intersects(ControlRegister::SPRITE_PATTERN_ADDR);
        if right_table { 0x1000 } else { 0 }
    }

    pub fn get_background_pattern_addr(&self) -> u16 {
        let right_table = self.intersects(ControlRegister::BACKGROUND_PATTERN_ADDR);
        if right_table { 0x1000 } else { 0 }
    }
    
    pub fn generate_nmi(&self) -> bool {
        self.intersects(ControlRegister::GENERATE_NMI)
    }
}
