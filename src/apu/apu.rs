#[derive(Debug)]
pub struct APU {
    registers: [u8; 0x15]
}

impl APU {
    pub fn new() -> APU {
        APU {
            registers: [0; 0x15]
        }
    }
}