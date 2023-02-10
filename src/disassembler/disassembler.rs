use super::super::cpu::CPU;

pub fn disassemble(bytes: &[u8]) {
    let cpu = CPU::new();
    if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
        let prg_rom_size = bytes[4] as usize * 16384;
        let chr_rom_size = bytes[5] as usize * 8192;
        let mapper = (bytes[7] & 0xF0) | (bytes[6] & 0xF0) >> 4; // https://www.nesdev.org/wiki/Mapper
        let mirroring = bytes[6] & 0x1; // 0 = Horizontal / 1 = Vertical
        let contains_memory = bytes[6] & 0x2; // 1 = yes (PGA_RAM) TODO
        let contains_trainer = bytes[6] & 0x4; // 512-byte trainer at $7000-$71FF (stored before PRG data)
        let ignore_mirroring = bytes[6] & 0x8; // 1 = ignore mirroring
        let console_type = bytes[7] & 0x3;
        let is_nes_2 = bytes[7] & 0x12;
        let prg_ram_size = bytes[8]; // TODO
        if is_nes_2 == 2 {
            // NES 2.0 not supported(yet).
            return
        }
        let mut trainer = None;
        let mut offset = 16;
        if contains_trainer == 1 {
            trainer = Some(&bytes[offset..527]);
            offset = 528;
        }

        let prg_rom = &bytes[offset..offset + (prg_rom_size - 1)];

        if bytes[4] > 1 {
            cpu.bus.set_prg_rom(true, prg_rom);
        } else {
            cpu.bus.set_prg_rom(false, prg_rom);
        }
        offset += prg_rom_size;
        let mut chr_rom = None;
        if chr_rom_size != 0 {
            chr_rom = Some(&bytes[offset..offset + (chr_rom_size - 1)]);
            offset += chr_rom_size;
        }
        if console_type != 0 {
            // Console type not supported.
            return
        }
    } else {
        // Only NES files supported.
        return
    }
}
