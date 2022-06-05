use crate::cpu::bus::BUS;

pub struct PPU<'a>{
    pub registers: &'a mut [u8],
    
}

impl<'a> PPU<'a> {
    pub fn new(bus: &'a mut BUS) -> PPU<'a>{
        PPU {
            registers: &mut bus.ram[0x2000..0x2007]
        }
    }
}