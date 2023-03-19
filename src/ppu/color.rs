use hex_string::u8_to_hex_string;

pub const COLORS: [u32; 0x40] = [
    0x666666ff, 0x002a88ff, 0x1412a7ff, 0x3b00a4ff, 0x5c007eff, 0x6e0040ff, 0x6c0600ff, 0x561d00ff,
    0x333500ff, 0x0b4800ff, 0x005200ff, 0x004f08ff, 0x00404dff, 0x000000ff, 0x000000ff, 0x000000ff,
    0xadadadff, 0x155fd9ff, 0x4240ffff, 0x7527feff, 0xa01accff, 0xb71e7bff, 0xb53120ff, 0x994e00ff,
    0x6b6d00ff, 0x388700ff, 0x0c9300ff, 0x008f32ff, 0x007c8dff, 0x000000ff, 0x000000ff, 0x000000ff,
    0xfffeffff, 0x64b0ffff, 0x9290ffff, 0xc676ffff, 0xf36affff, 0xfe6eccff, 0xfe8170ff, 0xea9e22ff,
    0xbcbe00ff, 0x88d800ff, 0x5ce430ff, 0x45e082ff, 0x48cddeff, 0x4f4f4fff, 0x000000ff, 0x000000ff,
    0xfffeffff, 0xc0dfffff, 0xd3d2ffff, 0xe8c8ffff, 0xfbc2ffff, 0xfec4eaff, 0xfeccc5ff, 0xf7d8a5ff,
    0xe4e594ff, 0xcfef96ff, 0xbdf4abff, 0xb3f3ccff, 0xb5ebf2ff, 0xb8b8b8ff, 0x000000ff, 0x000000ff,
];

#[derive(Clone, Copy)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    pub fn new() -> Color {
        Color { red: 0, green: 0, blue: 0, alpha: 0 }
    }
    pub fn decode(&mut self, color: u32) {
        self.red = ((color >> 24) & 0xFF) as u8;
        self.green = ((color >> 16) & 0xFF) as u8;
        self.blue = ((color >> 8) & 0xFF) as u8;
        self.alpha = (color & 0xFF) as u8;
    }

    pub fn to_hex(&self) -> String {
        // convert values to hexadecimal strings
        String::from(format!("#{}{}{}{}",
            u8_to_hex_string(&self.red).iter().collect::<String>(),
            u8_to_hex_string(&self.green).iter().collect::<String>(),
            u8_to_hex_string(&self.blue).iter().collect::<String>(),
            u8_to_hex_string(&self.alpha).iter().collect::<String>(),
        ))
    }
}

