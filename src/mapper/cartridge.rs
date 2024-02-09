use crate::mapper::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    OneScreenUpper,
    OneScreenLower,
    Vertical,
    Horizontal,
    FourScreen
}

pub fn new(bytes: &Vec<u8>) -> Result<Box<dyn Mapper>, String> {
    if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
        if bytes[7] & 0x12 == 2 { return Err("NES 2.0 not supported(yet).".to_string()) }

        let prg_rom_banks = bytes[4] as usize; // 16384
       // Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
        let chr_rom_banks = bytes[5] as usize; // 8192

        let four_screen = bytes[6] & 0x8 != 0;
        let vertical_mirroring = bytes[6] & 0x1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let skip_trainer = bytes[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_banks * 0x4000;

        let prg_rom = bytes[prg_rom_start..chr_rom_start].to_vec();
        let chr_rom = bytes[chr_rom_start..chr_rom_start + chr_rom_banks * 0x2000].to_vec();

        let mapper_id = (bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4;

        let mapper = match get_mapper(mapper_id, prg_rom, chr_rom, mirroring) {
            Ok(mapper) => mapper,
            Err(str) => return Err(str)
        };

        Ok(mapper)

    } else {
        Err("Only NES files supported.".to_string())
    }
}
