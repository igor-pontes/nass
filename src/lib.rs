mod utils;
use std::fmt::format;

use wasm_bindgen::prelude::*;
use web_sys::Storage;
mod cpu;
use crate::cpu::*;
mod scene;
use crate::scene::Scene;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn disassemble(ls: Storage) {
    let file = ls.get_item("file").unwrap().unwrap();
    let bytes = file.as_bytes();
    if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
        let prg_rom_size = bytes[4] as usize;
        let chr_rom_size = bytes[5] as usize;
        alert(&format!("PGR:{} CHR:{}", prg_rom_size, chr_rom_size));
        let mirroring = bytes[6] & 0x1; // 0 = Horizontal / 1 = Vertical
        let contains_memory = bytes[6] & 0x2; // 1 = yes
        let contains_trainer = bytes[6] & 0x4; // 512-byte trainer at $7000-$71FF (stored before PRG data)
        let ignore_mirroring = bytes[6] & 0x8; // 1 = ignore
        let console_type = bytes[7] & 0x3;
        let is_nes_2 = bytes[7] & 0x12;
        if is_nes_2 == 2 {
            alert("NES 2.0 not supported(yet).");
            return
        }
        let mut trainer = None;
        let mut offset = 16;
        if contains_trainer == 1 {
            trainer = Some(&bytes[16..527]);
            offset = 528;
        }
        let prg_rom = &bytes[offset..(offset + 16384 * prg_rom_size)];
        offset += 16384 * prg_rom_size + 1;
        let mut chr_rom = None;
        if chr_rom_size != 0 {
            chr_rom = Some(&bytes[offset..(offset + 8192 * chr_rom_size)]);
            offset += 8192 * chr_rom_size + 1;
        }
        if console_type != 0 {
            alert("Console type not supported.");
            return
        }


    } else {
        alert("Only NES files supported.")
    }
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, nass!");
}