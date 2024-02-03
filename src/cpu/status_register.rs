use bitflags::bitflags;

// https://www.nesdev.org/wiki/Status_flags
bitflags! {
    pub struct StatusRegister: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100; // When set, all interrupts except the NMI are inhibited.
        const DECIMAL           = 0b00001000;
        const BREAK             = 0b00010000; // Ignore
        const ONE               = 0b00100000; // Ignore
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_retain(0x34)
    }

    fn update(&mut self, value: u8) {
        *self = StatusRegister::from_bits_retain(value);
    }
    
    pub fn set_zero_negative(&mut self, value: u8) {
        self.set_zero(value == 0);
        self.set_negative((value & 0x80) > 0);
    }

    pub fn set_carry(&mut self, condition: bool) {
        // Increment and decrement instructions do not affect the carry flag.
        self.set(StatusRegister::CARRY, condition);
    }

    pub fn set_zero(&mut self, condition: bool) {
        self.set(StatusRegister::ZERO, condition);
    }
    
    pub fn set_interrupt_disable(&mut self, condition: bool) {
        self.set(StatusRegister::INTERRUPT_DISABLE, condition);
    }

    pub fn set_decimal(&mut self, condition: bool) {
        self.set(StatusRegister::DECIMAL, condition);
    }

    pub fn set_overflow(&mut self, condition: bool) {
        self.set(StatusRegister::OVERFLOW, condition);
    }

    pub fn set_negative(&mut self, condition: bool) {
        self.set(StatusRegister::NEGATIVE, condition);
    }

    pub fn set_break(&mut self, condition: bool) {
        self.set(StatusRegister::BREAK, condition);
    }

    pub fn irq_disabled(&self) -> bool {
        self.contains(StatusRegister::INTERRUPT_DISABLE)
    }

    pub fn set_effective(&mut self, value: u8) {
        // The two bits with no CPU effect are ignored when pulling flags from the stack; there are no corresponding registers for them in the CPU.
        // B is 0 when pushed by interrupts (/IRQ and /NMI) and 1 when pushed by instructions (BRK and PHP).
        self.update(value & !0x30);
    }
}
