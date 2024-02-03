use super::Mapper;
use crate::mapper::Mirroring;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)] fn log(s: &str);
}

const PRG_BANK_SIZE: usize = 0x4000;
const CHR_BANK_SIZE: usize = 0x2000;

use Bank::*;

#[derive(Debug)]
enum Bank {
    Switch(usize),
    Fixed(usize),
}

// https://www.nesdev.org/wiki/MMC1
// iNES Mapper 001 is used to designate the SxROM boardset, all of which use Nintendo's MMC1. 
#[derive(Debug)]
pub struct MMC1 {
    sr: u8,
    prg_addr: (Bank, Option<Bank>), // (PRG bank 0, PRG bank 1); prg_mode = 0, second element ignored. 
    chr_addr: (Bank, Option<Bank>), // (CHR bank 0, CHR bank 1) 
    prg_ram: [u8; 0x2000],
    prg_rom: Vec<u8>, 
    chr: Vec<u8>, 
    mirroring: Mirroring
}

impl MMC1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        MMC1 {
            sr: 0x10,
            prg_addr: (Switch(0), None),
            chr_addr: (Fixed(0), None),
            mirroring,
            prg_ram: [0; 0x2000],
            prg_rom,
            chr: chr_rom,
        } 
    }

    fn set_value_register(&mut self, reg: u16, value: u8) {
        log(&format!("set_value_register(reg: {}, value: {})",reg, value));
        match reg {
            0 => {
                // Control register.
                match value & 3 {
                    0 => Mirroring::OneScreenLower,
                    1 => Mirroring::OneScreenUpper,
                    2 => Mirroring::Vertical,
                    3 => Mirroring::Horizontal,
                    _ => panic!()
                };

                // PRG ROM bank mode
                // 0, 1 = switch 32 KB at 0x8000. (Low bit ignored)
                // 2 = fix first bank at 0x8000 and switch 16 KB bank at 0xC000.
                // 3 = fix last bank at 0xC000 and switch 16 KB bank at 0x8000.
                match (value & 0b1100) >> 2 {
                    0 | 1 => { self.prg_addr = (Switch(0), None) },
                    2 => { self.prg_addr = (Fixed(0), Some(Switch(0))) },
                    3 => { 
                        let last_prg_bank: usize = self.prg_rom.len() - PRG_BANK_SIZE;
                        self.prg_addr = (Switch(0), Some(Fixed(last_prg_bank))) 
                    },
                    _ => { panic!() }
                }

                // CHR ROM bank mode
                // 0: switch 8 KB at a time.
                // 1: switch two separate 4 KB banks.
                match (value & 0b10000) >> 5 {
                    0 => { self.chr_addr = (Switch(0), None) },
                    1 => { self.chr_addr = (Switch(0), Some(Switch(0))) },
                    _ => { panic!() }
                }
            },
            1 => {
                // Select 4 KB or 8 KB CHR bank at PPU $0000 (low bit ignored in 8 KB mode)
                match self.prg_addr {
                    (Switch(_), None) => {
                        let bank = (value & 0xE) as usize;
                        self.prg_addr = (Switch(bank * CHR_BANK_SIZE), None);
                    },
                    (Switch(_), Some(Switch(x))) => {
                        let bank  = (value & 0x1F) as usize;
                        self.prg_addr = (Switch(bank * CHR_BANK_SIZE), Some(Switch(x)));
                    }
                    _ => { log("MMC1: CHR bank 0 register unknown state."); panic!(); }
                }
            },
            2 => {
                // Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
                match self.prg_addr {
                    (Switch(_), None) => (),
                    (Switch(x), Some(Switch(_))) => {
                        let bank  = (value & 0x1F) as usize;
                        self.prg_addr = (Switch(x), Some(Switch(bank * CHR_BANK_SIZE)));
                    }
                    _ => { log("MMC1: PRG bank 1 register unknown state."); panic!(); }
                }
            }, 
            3 => {
                match self.prg_addr {
                    (Switch(_), None) => {
                        let bank = (value & 0xE) as usize;
                        self.prg_addr = (Switch(bank * PRG_BANK_SIZE), None);
                    },
                    (Fixed(x), Some(Switch(_))) => {
                        let bank  = (value & 0xF) as usize;
                        self.prg_addr = (Fixed(x), Some(Switch(bank * PRG_BANK_SIZE)));
                    },
                    (Switch(_), Some(Fixed(x))) => {
                        let bank  = (value & 0xF) as usize;
                        self.prg_addr = (Switch(bank * PRG_BANK_SIZE), Some(Fixed(x)));
                    }
                    _ => { log("MMC1: PRG bank register unknown state."); panic!(); }
                }
            },
            _ => { log("MMC1: Unknown register."); panic!(); }
        };
    }

    fn update_sr(&mut self, value: u8, addr: u16) {
        // To change a register's value, the CPU writes five times with bit 7 clear and one bit of the desired value in bit 0. 
        // Otherwise, reset shift register.
        if (value & 0x7) != 0 { self.sr = 0x10; return; }

        // The 1 is used to detect when the SR has become full.
        if self.sr == 1 {
            self.sr >>= 1;
            self.sr |= (value & 0x1) << 5;
            let reg = (addr & 0x6000) >> 13;
            self.set_value_register(reg, self.sr);
            self.sr = 0x10;
        } else {
            // Check if  
            self.sr >>= 1;
            self.sr |= (value & 0x1) << 5;
            self.sr = 0x10;
        }
    }
}

// Fix this.
impl Mapper for MMC1 {
    fn get_mirroring(&self) -> Mirroring { self.mirroring }

    fn read_chr(&self, addr: u16) -> u8 { 
        match self.chr_addr {
            (Switch(x), None) => { self.chr[x + addr as usize] },
            (Switch(x), Some(Switch(y))) => { 
                if addr >= 0x1000 { 
                    let diff = (addr - 0x1000) as usize;
                    self.chr[y + diff] 
                } else { 
                    self.chr[x + addr as usize] 
                }
            }
            _ => { log("MMC1: Unknown CHR read."); panic!(); }
        }
    }

    fn read_prg(&self, addr: u16) -> u8 { 
        if addr <= 0x7FFF { return self.prg_ram[(addr -  0x6000) as usize]; }
        match &self.prg_addr {
            (Switch(x), None) => { 
                let diff = (addr - 0x8000) as usize;
                self.prg_rom[x + diff] 
            },
            (Fixed(x), Some(Switch(y))) => {
                if addr >= 0xC000 { 
                    let diff = (addr - 0xC000) as usize;
                    self.prg_rom[y + diff] 
                } else { 
                    let diff = (addr - 0x8000) as usize;
                    self.prg_rom[x + diff] 
                } 
            }
            (Switch(x), Some(Fixed(y))) => { 
                if addr >= 0xC000 { 
                    let diff = (addr - 0xC000) as usize;
                    self.prg_rom[y + diff] 
                } else { 
                    let diff = (addr - 0x8000) as usize;
                    self.prg_rom[x + diff] 
                }
            }
            (b1, b2) => { 
                log(&format!("{:?}, {:?}", b1, b2)); 
                log("MMC1: Unknown PRG ROM read."); 
                panic!(); 
            }
        }
    }

    // CPU $6000-$7FFF: 8 KB PRG RAM bank.
    fn write_prg(&mut self, addr: u16, val: u8) { 
        self.update_sr(val, addr);
        self.prg_ram[addr as usize - 0x6000] = val; 
    }

    // MMC1 can do CHR banking in 4KB chunks. Known carts with CHR RAM have 8 KiB, so that makes 2 banks. RAM vs ROM doesn't make any difference for address lines.
    // fn write_chr(&mut self, addr: u16, val: u8) { self.chr[addr as usize] = val; }
    fn write_chr(&mut self, addr: u16, val: u8) { }
}
