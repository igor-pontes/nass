use bitflags::bitflags;

// https://www.nesdev.org/wiki/Status_flags
bitflags! {
    pub struct CPUStatus: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100; // When set, all interrupts except the NMI are inhibited.
        const DECIMAL           = 0b00001000;
        const BREAK             = 0b00010000;
        const ONE               = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

impl CPUStatus {
    pub fn new() -> Self {
        CPUStatus::from_bits_retain(0x34)
    }

    pub fn update(&mut self, value: u8) {
        *self = CPUStatus::from_bits_retain(value);
    }
    
    pub fn set_zn(&mut self, value: u8) {
        self.set_zero(value == 0);
        self.set_negative((value & 0x80) > 0);
    }

    pub fn set_carry(&mut self, condition: bool) {
        self.set(CPUStatus::CARRY, condition);
    }

    pub fn set_zero(&mut self, condition: bool) {
        self.set(CPUStatus::ZERO, condition);
    }
    
    pub fn set_interrupt(&mut self, condition: bool) {
        self.set(CPUStatus::INTERRUPT_DISABLE, condition);
    }

    pub fn set_decimal(&mut self, condition: bool) {
        self.set(CPUStatus::DECIMAL, condition);
    }

    pub fn set_overflow(&mut self, condition: bool) {
        self.set(CPUStatus::OVERFLOW, condition);
    }

    pub fn set_negative(&mut self, condition: bool) {
        self.set(CPUStatus::NEGATIVE, condition);
    }

    pub fn set_break(&mut self, condition: bool) {
        self.set(CPUStatus::BREAK, condition);
    }

    pub fn irq_disabled(&self) -> bool {
        self.intersects(CPUStatus::INTERRUPT_DISABLE)
    }
}
